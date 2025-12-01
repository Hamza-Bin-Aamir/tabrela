#!/bin/bash
# ==============================================================================
# Tabrela - VPS Initial Setup Script
# ==============================================================================
# Run this script on a fresh VPS to prepare it for Tabrela deployment
# 
# Usage: curl -fsSL https://raw.githubusercontent.com/Hamza-Bin-Aamir/tabrela/main/scripts/vps-setup.sh | bash
# Or: wget -qO- https://raw.githubusercontent.com/Hamza-Bin-Aamir/tabrela/main/scripts/vps-setup.sh | bash
# ==============================================================================

set -e

echo "========================================"
echo "Tabrela VPS Setup Script"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root (use sudo)${NC}"
    exit 1
fi

# ==============================================================================
# 1. Update System
# ==============================================================================
echo -e "${YELLOW}[1/7] Updating system...${NC}"
apt update && apt upgrade -y

# ==============================================================================
# 2. Install Docker
# ==============================================================================
echo -e "${YELLOW}[2/7] Installing Docker...${NC}"
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    rm get-docker.sh
    
    # Install Docker Compose plugin
    apt install -y docker-compose-plugin
else
    echo "Docker already installed, skipping..."
fi

# ==============================================================================
# 3. Install Required Packages
# ==============================================================================
echo -e "${YELLOW}[3/7] Installing required packages...${NC}"
apt install -y \
    git \
    curl \
    nginx \
    certbot \
    python3-certbot-nginx \
    rsync \
    ufw

# ==============================================================================
# 4. Create Deploy User
# ==============================================================================
echo -e "${YELLOW}[4/7] Creating deploy user...${NC}"
if ! id "deploy" &>/dev/null; then
    useradd -m -s /bin/bash deploy
    usermod -aG docker deploy
    
    # Set up SSH directory
    mkdir -p /home/deploy/.ssh
    chmod 700 /home/deploy/.ssh
    touch /home/deploy/.ssh/authorized_keys
    chmod 600 /home/deploy/.ssh/authorized_keys
    chown -R deploy:deploy /home/deploy/.ssh
    
    echo -e "${GREEN}Deploy user created. Add your SSH public key to:${NC}"
    echo "/home/deploy/.ssh/authorized_keys"
else
    echo "Deploy user already exists, skipping..."
fi

# ==============================================================================
# 5. Create Deployment Directories
# ==============================================================================
echo -e "${YELLOW}[5/7] Creating deployment directories...${NC}"
mkdir -p /opt/tabrela
mkdir -p /var/www/tabrela
chown deploy:deploy /opt/tabrela
chown deploy:deploy /var/www/tabrela

# ==============================================================================
# 6. Configure Firewall
# ==============================================================================
echo -e "${YELLOW}[6/7] Configuring firewall...${NC}"
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow http
ufw allow https
ufw --force enable

# ==============================================================================
# 7. Start Services
# ==============================================================================
echo -e "${YELLOW}[7/7] Starting services...${NC}"
systemctl enable docker
systemctl start docker
systemctl enable nginx
systemctl start nginx

# ==============================================================================
# Summary
# ==============================================================================
echo ""
echo "========================================"
echo -e "${GREEN}VPS Setup Complete!${NC}"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Add your SSH public key to /home/deploy/.ssh/authorized_keys"
echo "2. Copy docker-compose.yml to /opt/tabrela/"
echo "3. Create /opt/tabrela/.env from .env.example"
echo "4. Configure Nginx in /etc/nginx/sites-available/tabrela"
echo "5. Set up SSL with: certbot --nginx -d yourdomain.com"
echo "6. Configure GitHub repository secrets"
echo ""
echo "See docs/VPS_DEPLOYMENT.md for detailed instructions"
echo ""
