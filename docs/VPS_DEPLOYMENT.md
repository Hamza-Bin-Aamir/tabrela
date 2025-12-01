# ==============================================================================
# Tabrela - VPS Deployment Guide
# ==============================================================================
# This guide explains how to set up your VPS for Tabrela deployment
# ==============================================================================

## Prerequisites

- A VPS with at least 1GB RAM (2GB recommended)
- Ubuntu 22.04 LTS or Debian 12 (Bookworm)
- SSH access to the VPS
- A domain name pointed to your VPS IP

## 1. Initial VPS Setup

### Update System
```bash
sudo apt update && sudo apt upgrade -y
```

### Install Docker
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group (so you don't need sudo)
sudo usermod -aG docker $USER

# Install Docker Compose plugin
sudo apt install docker-compose-plugin -y

# Verify installation
docker --version
docker compose version
```

### Install Required Packages
```bash
sudo apt install -y \
    git \
    curl \
    nginx \
    certbot \
    python3-certbot-nginx \
    rsync
```

## 2. Create Deployment User

For security, create a dedicated user for deployments:

```bash
# Create user
sudo useradd -m -s /bin/bash deploy

# Add to docker group
sudo usermod -aG docker deploy

# Set up SSH keys (from your local machine)
# On your local machine: ssh-keygen -t ed25519 -C "tabrela-deploy"
# Then copy the public key to the server

sudo mkdir -p /home/deploy/.ssh
sudo nano /home/deploy/.ssh/authorized_keys
# Paste your public key

sudo chown -R deploy:deploy /home/deploy/.ssh
sudo chmod 700 /home/deploy/.ssh
sudo chmod 600 /home/deploy/.ssh/authorized_keys
```

## 3. Create Deployment Directories

```bash
# Backend deployment directory
sudo mkdir -p /opt/tabrela
sudo chown deploy:deploy /opt/tabrela

# Frontend directory (Apache/Nginx web root)
sudo mkdir -p /var/www/tabrela
sudo chown deploy:deploy /var/www/tabrela
```

## 4. Configure Docker to Pull from GHCR

```bash
# Login to GitHub Container Registry
# You'll need a Personal Access Token with read:packages permission
docker login ghcr.io -u YOUR_GITHUB_USERNAME

# Enter your PAT when prompted
```

## 5. Configure Nginx (Reverse Proxy)

Create Nginx configuration for both frontend and backend:

```bash
sudo nano /etc/nginx/sites-available/tabrela
```

Add the following configuration:

```nginx
# ==============================================================================
# Tabrela - Nginx Configuration
# ==============================================================================

# Frontend (React SPA)
server {
    listen 80;
    server_name tabrela.yourdomain.com;

    root /var/www/tabrela;
    index index.html;

    # Handle SPA routing (React Router)
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}

# Backend API (Docker services)
server {
    listen 80;
    server_name api.tabrela.yourdomain.com;

    # Auth Service
    location /auth/ {
        proxy_pass http://127.0.0.1:8081/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Attendance Service
    location /attendance/ {
        proxy_pass http://127.0.0.1:8082/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Merit Service
    location /merit/ {
        proxy_pass http://127.0.0.1:8083/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Email Service (internal only - don't expose publicly)
    # location /email/ {
    #     proxy_pass http://127.0.0.1:5000/;
    # }
}
```

Enable the site:
```bash
sudo ln -s /etc/nginx/sites-available/tabrela /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## 6. Set Up SSL with Let's Encrypt

```bash
sudo certbot --nginx -d tabrela.yourdomain.com -d api.tabrela.yourdomain.com
```

## 7. Copy Docker Compose File to VPS

```bash
# On VPS
cd /opt/tabrela

# Copy docker-compose.yml and .env file
# You can do this via git clone or scp
```

Create the `.env` file:
```bash
nano /opt/tabrela/.env
```

Fill in the values from `.env.example`.

## 8. Start the Backend Services

```bash
cd /opt/tabrela
docker compose up -d
```

## 9. Configure GitHub Repository Secrets

In your GitHub repository, go to **Settings → Secrets and variables → Actions** and add:

### Frontend Deployment Secrets:
| Secret Name | Description | Example |
|-------------|-------------|---------|
| `VPS_SSH_KEY` | SSH private key for deploy user | Contents of `~/.ssh/tabrela_deploy` |
| `VPS_HOST` | VPS hostname or IP | `123.45.67.89` |
| `VPS_USER` | SSH username | `deploy` |
| `VPS_PORT` | SSH port | `22` |
| `VPS_FRONTEND_PATH` | Frontend deployment path | `/var/www/tabrela` |
| `VITE_API_BASE_URL` | Backend API URL | `https://api.tabrela.yourdomain.com` |

### Backend Deployment Secrets:
| Secret Name | Description | Example |
|-------------|-------------|---------|
| `VPS_DEPLOY_PATH` | Backend deployment path | `/opt/tabrela` |

## 10. Test the Deployment

### Manually Trigger Workflows:
1. Go to **Actions** tab in GitHub
2. Select the workflow you want to run
3. Click "Run workflow"

### Verify Services:
```bash
# Check Docker containers
docker compose ps

# View logs
docker compose logs -f

# Test auth service health
curl http://localhost:8081/health
```

## Troubleshooting

### Docker Won't Start
```bash
# Check Docker service
sudo systemctl status docker

# View container logs
docker compose logs backend
```

### Permission Denied
```bash
# Re-add user to docker group and re-login
sudo usermod -aG docker $USER
newgrp docker
```

### Port Already in Use
```bash
# Find process using the port
sudo lsof -i :8081
# Kill if needed
sudo kill -9 PID
```

### Database Connection Issues
```bash
# Check if PostgreSQL container is running
docker compose ps db

# Connect to database
docker compose exec db psql -U tabrela -d tabrela
```

## Maintenance

### View Logs
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
```

### Update to Latest Version
```bash
cd /opt/tabrela
docker compose pull
docker compose up -d
```

### Backup Database
```bash
docker compose exec db pg_dump -U tabrela tabrela > backup_$(date +%Y%m%d).sql
```

### Clean Up Docker Resources
```bash
# Remove unused images
docker image prune -f

# Remove all unused resources
docker system prune -f
```
