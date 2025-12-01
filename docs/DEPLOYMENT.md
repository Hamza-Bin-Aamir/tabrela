# Tabrela - Automated Deployment CI/CD for Railway and cPanel
Hybrid deployment setup:
- Frontend: cPanel shared hosting (Apache)
- Backend: Railway.app (Docker)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         INTERNET                                 │
└─────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
    ┌───────────────────────────┐   ┌───────────────────────────┐
    │   cPanel Shared Hosting   │   │       Railway.app         │
    │   (Frontend - React SPA)  │   │   (Backend - Docker)      │
    │                           │   │                           │
    │   tabrela.yourdomain.com  │   │   tabrela-api.up.railway  │
    │                           │   │   .app                    │
    │   ┌───────────────────┐   │   │   ┌───────────────────┐   │
    │   │   public_html/    │   │   │   │   Auth (8081)     │   │
    │   │   - index.html    │   │   │   │   Attendance(8082)│   │
    │   │   - assets/       │   │   │   │   Merit (8083)    │   │
    │   │   - .htaccess     │   │   │   │   Email (5000)    │   │
    │   └───────────────────┘   │   │   └───────────────────┘   │
    └───────────────────────────┘   │                           │
                                    │   ┌───────────────────┐   │
                                    │   │   PostgreSQL      │   │
                                    │   │   (Railway DB)    │   │
                                    │   └───────────────────┘   │
                                    └───────────────────────────┘
```

---

## Part 1: Railway Setup (Backend)

### 1.1 Create Railway Account & Project

1. Go to [railway.app](https://railway.app) and sign up with GitHub
2. Click **"New Project"** → **"Deploy from GitHub repo"**
3. Select your `tabrela` repository
4. Railway will detect the `railway.toml` and use `docker/Dockerfile.backend`

### 1.2 Add PostgreSQL Database

1. In your Railway project, click **"+ New"** → **"Database"** → **"PostgreSQL"**
2. Railway will automatically provision a PostgreSQL instance
3. Click on the PostgreSQL service to see connection details

### 1.3 Configure Environment Variables

Click on your backend service → **"Variables"** → Add these:

| Variable | Description | Example |
|----------|-------------|---------|
| `PORT` | Railway injects this; must be `8081` for auth service health check | `8081` |
| `DATABASE_URL` | Auto-linked from PostgreSQL | `${{Postgres.DATABASE_URL}}` |
| `JWT_SECRET` | Random 32+ char string | `your-super-secret-jwt-key` |
| `JWT_ACCESS_TOKEN_EXPIRY` | Token expiry in seconds | `3600` |
| `JWT_REFRESH_TOKEN_EXPIRY` | Refresh token expiry | `604800` |
| `RESEND_API_KEY` | From resend.com | `re_xxxxx` |
| `EMAIL_SERVICE_API_KEY` | API key for email service (must match `SERVICE_API_KEY` in email service) | `re_xxxxx` |
| `FROM_EMAIL` | Sender email | `noreply@yourdomain.com` |
| `RUST_LOG` | Log level | `info` |
| `ALLOWED_ORIGINS` | Frontend URL | `https://tabrela.yourdomain.com` |
| `PASSWORD_PEPPER` | Extra secret for password hashing | `b7f3c8e2a1d4f6e9c0b2a8d7e5f1c3a4b6d8e0f2c4a6b8d0e2f4c6a8b0d2e4f6` |
| `SERVICE_API_KEY` | API key for inter-service authentication | `service_xxxxx` |
| `FRONTEND_URL` | Public URL of the frontend (used in email links) | `https://tabrela.yourdomain.com` |

### 1.4 Get Railway Deploy Token

1. Go to **Account Settings** → **Tokens**
2. Create a new token called `github-actions`
3. Copy the token (you'll add it to GitHub secrets)

### 1.5 Note Your Railway URL

After first deployment, Railway will give you a URL like:
```
https://tabrela-backend-production.up.railway.app
```

This is your `VITE_API_BASE_URL` for the frontend.

---

## Part 2: cPanel Setup (Frontend)

### 2.1 Enable SSH Access

1. Log in to cPanel
2. Go to **Security** → **SSH Access** or **Manage SSH Keys**
3. Generate a new SSH key pair OR import your existing public key
4. Authorize the key for SSH access

### 2.2 Get SSH Connection Details

From cPanel, note:
- **Host**: Usually your server hostname 
- **Username**: Your cPanel username
- **Port**: Usually `22` 
- **Path**: `~/public_html` or `~/public_html/subdomain_folder`

### 2.3 Test SSH Connection (Optional)

From your local machine:
```bash
ssh -p 22 your-username@your-server-ip
```

### 2.4 Ensure .htaccess is in Place

The `.htaccess` file in `web/public/` will be copied during build.
Verify it exists after first deployment:
```bash
cat ~/public_html/.htaccess
```

It should contain:
```apache
<IfModule mod_rewrite.c>
    RewriteEngine On
    RewriteBase /
    RewriteRule ^index\.html$ - [L]
    RewriteCond %{REQUEST_FILENAME} !-f
    RewriteCond %{REQUEST_FILENAME} !-d
    RewriteRule . /index.html [L]
</IfModule>
```

---

## Part 3: GitHub Secrets Configuration

Go to your GitHub repo → **Settings** → **Secrets and variables** → **Actions**

### Frontend (cPanel) Secrets:

| Secret | Description | Example |
|--------|-------------|---------|
| `CPANEL_SSH_KEY` | Private SSH key | Contents of `~/.ssh/id_ed25519` |
| `CPANEL_HOST` | Server hostname/IP | `209.42.24.4` |
| `CPANEL_USER` | cPanel username | `gikidtco` |
| `CPANEL_PORT` | SSH port (optional, defaults to 22) | `22` |
| `CPANEL_PUBLIC_HTML` | Deployment path | `~/public_html` |
| `VITE_API_BASE_URL` | Railway backend URL | `https://tabrela-api.up.railway.app` |

### Backend (Railway) Secrets:

| Secret | Description | Example |
|--------|-------------|---------|
| `RAILWAY_TOKEN` | Railway deploy token | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |

---

## Part 4: Trigger Deployments

### Automatic Deployment
Push to `main` branch:
- Changes in `web/` → Frontend deploys to cPanel
- Changes in `services/` → Backend deploys to Railway

### Manual Deployment
1. Go to **Actions** tab in GitHub
2. Select workflow
3. Click **"Run workflow"**

---

## Part 5: Verify Deployment

### Check Frontend
Visit your domain: `https://tabrela.yourdomain.com`

### Check Backend
```bash
# Health check
curl https://tabrela-api.up.railway.app/health

# Auth service
curl https://tabrela-api.up.railway.app/health
```

### Railway Dashboard
- View logs in Railway dashboard
- Check deployment status
- Monitor resource usage

---

## Troubleshooting

### Frontend Issues

**404 errors on page refresh:**
- Check `.htaccess` exists in `public_html`
- Verify Apache `mod_rewrite` is enabled (should be by default on cPanel)

**CORS errors:**
- Add your frontend domain to `ALLOWED_ORIGINS` in Railway

**SSH connection failed:**
- Verify SSH key is authorized in cPanel
- Check correct port (might not be 22)
- Some cPanel hosts use different SSH ports

### Backend Issues

**Railway build fails:**
- Check Railway build logs
- Ensure `docker/Dockerfile.backend` syntax is correct
- Check all service source files are present

**Database connection errors:**
- Verify `DATABASE_URL` is using Railway's variable reference: `${{Postgres.DATABASE_URL}}`
- Run migrations (Railway can do this automatically or add a start command)

**Services not starting:**
- Check Railway logs for each service
- Verify environment variables are set

---

## Cost Estimate

### cPanel Hosting
- You already have this: **$0/month** (included in your existing plan)

### Railway.app
- **Free Tier**: $5 credit/month (enough for light usage)
- **Starter**: ~$5-20/month depending on usage
- PostgreSQL: Included in usage

### Total: ~$5-20/month
(Much cheaper than a dedicated VPS + more reliable)

---

## Updating the Application

### Frontend Update
```bash
# Make changes to web/ folder
git add .
git commit -m "Update frontend"
git push origin main
# GitHub Actions auto-deploys to cPanel
```

### Backend Update
```bash
# Make changes to services/ folder
git add .
git commit -m "Update backend"
git push origin main
# GitHub Actions auto-deploys to Railway
```

---

## Database Migrations

Railway doesn't run migrations automatically. Options:

### Option 1: Run via Railway CLI
```bash
# Install Railway CLI
npm install -g @railway/cli

# Login
railway login

# Link to project
railway link

# Run migration command
railway run sqlx migrate run
```

### Option 2: Add to Dockerfile
Add migration step to `docker/Dockerfile.backend` (already handled by supervisord starting the services).

---

## Backup

### Database Backup (Railway)
```bash
# Using Railway CLI
railway run pg_dump > backup_$(date +%Y%m%d).sql
```

### Frontend Backup
Not needed - source is in Git, deployed files can be regenerated.
