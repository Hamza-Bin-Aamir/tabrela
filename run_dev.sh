#!/bin/bash

# Tabrela Development Environment Startup Script
# This script starts all microservices and the frontend application

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
port_in_use() {
    lsof -i:$1 >/dev/null 2>&1
}

# Check required tools
print_info "Checking required tools..."

if ! command_exists cargo; then
    print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

if ! command_exists python3; then
    print_error "Python3 is not installed. Please install Python3."
    exit 1
fi

if ! command_exists npm; then
    print_error "Node.js/npm is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi

print_success "All required tools are installed"

# Check for .env files
print_info "Checking environment files..."

if [ ! -f "services/auth/.env" ]; then
    print_warning "services/auth/.env not found. Creating from .env.example..."
    cp services/auth/.env.example services/auth/.env
    print_warning "Please edit services/auth/.env with your configuration"
fi

if [ ! -f "services/email/.env" ]; then
    print_warning "services/email/.env not found. Creating from .env.example..."
    cp services/email/.env.example services/email/.env
    print_warning "Please edit services/email/.env with your configuration"
fi

if [ ! -f "services/attendance/.env" ]; then
    print_warning "services/attendance/.env not found. Creating from .env.example..."
    cp services/attendance/.env.example services/attendance/.env
    print_warning "Please edit services/attendance/.env with your configuration"
fi

if [ ! -f "web/.env" ]; then
    print_warning "web/.env not found. Creating from .env.example..."
    if [ -f "web/.env.example" ]; then
        cp web/.env.example web/.env
    else
        echo "VITE_API_URL=http://localhost:8081" > web/.env
    fi
fi

# Check if ports are available
print_info "Checking port availability..."

if port_in_use 8081; then
    print_error "Port 8081 (Auth Service) is already in use"
    print_info "Please stop the process using: lsof -ti:8081 | xargs kill -9"
    exit 1
fi

if port_in_use 8082; then
    print_error "Port 8082 (Attendance Service) is already in use"
    print_info "Please stop the process using: lsof -ti:8082 | xargs kill -9"
    exit 1
fi

if port_in_use 5000; then
    print_error "Port 5000 (Email Service) is already in use"
    print_info "Please stop the process using: lsof -ti:5000 | xargs kill -9"
    exit 1
fi

if port_in_use 5173; then
    print_error "Port 5173 (Frontend) is already in use"
    print_info "Please stop the process using: lsof -ti:5173 | xargs kill -9"
    exit 1
fi

print_success "All ports are available"

# Create logs directory
mkdir -p logs

# Install dependencies if needed
print_info "Checking dependencies..."

# Check Python dependencies for email service
if [ ! -d "services/email/venv" ]; then
    print_info "Creating Python virtual environment for email service..."
    cd services/email
    python3 -m venv venv
    source venv/bin/activate
    pip install --quiet -r requirements.txt
    deactivate
    cd ../..
    print_success "Email service dependencies installed"
else
    print_info "Email service virtual environment already exists"
fi

# Check frontend dependencies
if [ ! -d "web/node_modules" ]; then
    print_info "Installing frontend dependencies..."
    cd web
    npm install
    cd ..
    print_success "Frontend dependencies installed"
else
    print_info "Frontend dependencies already installed"
fi

print_success "All dependencies are ready"

# Start services
print_info "Starting microservices..."

# Start Email Service
print_info "Starting Email Service on port 5000..."
cd services/email
source venv/bin/activate
nohup python app.py > ../../logs/email-service.log 2>&1 &
EMAIL_PID=$!
deactivate
cd ../..
echo $EMAIL_PID > logs/email-service.pid
print_success "Email Service started (PID: $EMAIL_PID)"

# Wait for email service to be ready
sleep 2

# Start Auth Service
print_info "Starting Auth Service on port 8081..."
cd services/auth
nohup cargo run --release > ../../logs/auth-service.log 2>&1 &
AUTH_PID=$!
cd ../..
echo $AUTH_PID > logs/auth-service.pid
print_success "Auth Service started (PID: $AUTH_PID)"

# Start Attendance Service
print_info "Starting Attendance Service on port 8082..."
cd services/attendance
nohup cargo run --release > ../../logs/attendance-service.log 2>&1 &
ATTENDANCE_PID=$!
cd ../..
echo $ATTENDANCE_PID > logs/attendance-service.pid
print_success "Attendance Service started (PID: $ATTENDANCE_PID)"

# Wait for auth service to be ready
print_info "Waiting for services to initialize..."
sleep 5

# Start Frontend
print_info "Starting Frontend on port 5173..."
cd web
nohup npm run dev -- --host > ../logs/frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..
echo $FRONTEND_PID > logs/frontend.pid
print_success "Frontend started (PID: $FRONTEND_PID)"

# Wait a moment for everything to start
sleep 3

# Display status
echo ""
echo "======================================"
print_success "🚀 All services started successfully!"
echo "======================================"
echo ""
echo -e "${GREEN}Services:${NC}"
echo "  📧 Email Service:       http://localhost:5000 (PID: $EMAIL_PID)"
echo "  🔐 Auth Service:        http://localhost:8081 (PID: $AUTH_PID)"
echo "  📋 Attendance Service:  http://localhost:8082 (PID: $ATTENDANCE_PID)"
echo "  🌐 Frontend:            http://localhost:5173 (PID: $FRONTEND_PID)"
echo ""
echo -e "${BLUE}Logs:${NC}"
echo "  Email Service:       tail -f logs/email-service.log"
echo "  Auth Service:        tail -f logs/auth-service.log"
echo "  Attendance Service:  tail -f logs/attendance-service.log"
echo "  Frontend:            tail -f logs/frontend.log"
echo ""
echo -e "${YELLOW}To stop all services:${NC}"
echo "  ./stop_dev.sh"
echo ""
echo "======================================"

# Function to cleanup on exit
cleanup() {
    print_info "Stopping all services..."
    if [ -f logs/email-service.pid ]; then
        kill $(cat logs/email-service.pid) 2>/dev/null || true
    fi
    if [ -f logs/auth-service.pid ]; then
        kill $(cat logs/auth-service.pid) 2>/dev/null || true
    fi
    if [ -f logs/attendance-service.pid ]; then
        kill $(cat logs/attendance-service.pid) 2>/dev/null || true
    fi
    if [ -f logs/frontend.pid ]; then
        kill $(cat logs/frontend.pid) 2>/dev/null || true
    fi
    print_success "All services stopped"
}

# Register cleanup function to run on script exit
trap cleanup EXIT

# Keep script running and follow logs
print_info "Following logs (Ctrl+C to stop all services)..."
echo ""
tail -f logs/*.log 2>/dev/null || sleep infinity
