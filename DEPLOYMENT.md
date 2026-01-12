# Deploying to Hetzner Server

This guide covers deploying the MAPF backend to your Hetzner dedicated server.

## Prerequisites

- Hetzner server with Ubuntu 22.04+ or Debian 11+
- SSH access
- Domain name pointing to server (optional but recommended)

## Server Setup

### 1. Install Dependencies

```bash
# Connect to server
ssh user@your-server.com

# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo apt install docker-compose-plugin

# Logout and login for docker group to take effect
exit
ssh user@your-server.com
```

### 2. Create Application Directory

```bash
sudo mkdir -p /opt/mapf
sudo chown $USER:$USER /opt/mapf
cd /opt/mapf
```

### 3. Transfer Files

From your local machine:

```bash
# Build Docker image locally
task docker:build

# Save image to file
docker save mapf-server:latest | gzip > mapf-server.tar.gz

# Transfer to server
scp mapf-server.tar.gz user@your-server.com:/opt/mapf/
scp backend/docker-compose.yml user@your-server.com:/opt/mapf/
scp backend/migrations user@your-server.com:/opt/mapf/ -r
```

On the server:

```bash
cd /opt/mapf

# Load Docker image
docker load < mapf-server.tar.gz

# Create .env file
cat > .env << 'EOF'
DATABASE_URL=postgres://mapf:CHANGE_ME_PASSWORD@postgres/mapf_arena
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info,mapf_server=debug
CORS_ALLOWED_ORIGINS=https://mapf.dev,https://your-domain.com
MAX_WASM_SIZE_MB=10
SOLVER_TIMEOUT_SECS=30
SOLVER_INSTRUCTION_LIMIT=10000000000
EOF

# Edit .env and change the database password
nano .env
```

### 4. Start Services

```bash
cd /opt/mapf

# Start PostgreSQL and backend
docker-compose up -d

# Check logs
docker-compose logs -f

# Verify health
curl http://localhost:3000/health
```

## Nginx Reverse Proxy (Recommended)

### 1. Install Nginx

```bash
sudo apt install nginx certbot python3-certbot-nginx
```

### 2. Configure Nginx

```bash
sudo nano /etc/nginx/sites-available/mapf-api
```

Add configuration:

```nginx
server {
    listen 80;
    server_name api.mapf.dev;  # Change to your domain

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Increase timeout for long-running solvers
        proxy_read_timeout 60s;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
    }
}
```

Enable site:

```bash
sudo ln -s /etc/nginx/sites-available/mapf-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 3. Enable HTTPS

```bash
sudo certbot --nginx -d api.mapf.dev
```

## Firewall Configuration

```bash
# Allow SSH, HTTP, HTTPS
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable
```

## Systemd Service (Alternative to Docker Compose)

If you prefer running as a systemd service instead of Docker Compose:

```bash
sudo nano /etc/systemd/system/mapf-server.service
```

```ini
[Unit]
Description=MAPF Arena Backend
After=network.target postgresql.service

[Service]
Type=simple
User=mapf
Group=mapf
WorkingDirectory=/opt/mapf
Environment="DATABASE_URL=postgres://mapf:password@localhost/mapf_arena"
Environment="SERVER_PORT=3000"
Environment="RUST_LOG=info,mapf_server=debug"
ExecStart=/opt/mapf/mapf-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable mapf-server
sudo systemctl start mapf-server
sudo systemctl status mapf-server
```

## Monitoring

### Check Logs

```bash
# Docker Compose
docker-compose logs -f backend

# Systemd
sudo journalctl -u mapf-server -f
```

### Database Backup

```bash
# Backup
docker-compose exec postgres pg_dump -U mapf mapf_arena > backup.sql

# Restore
docker-compose exec -T postgres psql -U mapf mapf_arena < backup.sql
```

## Updating

```bash
cd /opt/mapf

# Pull new image
scp mapf-server.tar.gz user@your-server.com:/opt/mapf/
docker load < mapf-server.tar.gz

# Restart services
docker-compose down
docker-compose up -d

# Check logs
docker-compose logs -f
```

## Frontend Configuration

Update your frontend [.env](../frontend/.env):

```env
VITE_BACKEND_URL=https://api.mapf.dev
```

Rebuild and redeploy frontend to Cloudflare Pages.

## Security Checklist

- [x] Change default PostgreSQL password
- [x] Enable firewall (ufw)
- [x] Set up HTTPS with certbot
- [x] Configure CORS for your domain only
- [x] Use strong API keys for submissions
- [x] Keep Docker images updated
- [x] Regular database backups
- [x] Monitor logs for suspicious activity

## Troubleshooting

### Backend not starting

```bash
# Check logs
docker-compose logs backend

# Check database connection
docker-compose exec backend curl http://localhost:3000/health
```

### Database connection errors

```bash
# Check PostgreSQL is running
docker-compose ps

# Test connection
docker-compose exec postgres psql -U mapf -d mapf_arena -c "SELECT 1;"
```

### High memory usage

```bash
# Check container stats
docker stats

# Adjust solver limits in .env
# Reduce SOLVER_INSTRUCTION_LIMIT or SOLVER_TIMEOUT_SECS
```

## Performance Tuning

### PostgreSQL

Edit `backend/docker-compose.yml` and add under `postgres` service:

```yaml
command:
  - postgres
  - -c
  - shared_buffers=256MB
  - -c
  - effective_cache_size=1GB
  - -c
  - maintenance_work_mem=128MB
```

### Backend

Adjust limits in `.env`:

```env
# Reduce if memory constrained
SOLVER_INSTRUCTION_LIMIT=5000000000

# Reduce timeout for faster feedback
SOLVER_TIMEOUT_SECS=15

# Limit WASM size
MAX_WASM_SIZE_MB=5
```

## Support

For issues, check:
- Backend logs: `docker-compose logs -f`
- Database logs: `docker-compose logs postgres`
- Nginx logs: `/var/log/nginx/error.log`

Report bugs at: https://github.com/your-org/mapf/issues
