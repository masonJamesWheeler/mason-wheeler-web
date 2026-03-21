#!/usr/bin/env bash
#
# One-time VM setup for mason-wheeler.com
# Run as azureuser with sudo privileges.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="/home/azureuser/app"
DATA_DIR="${APP_DIR}/data"
DOMAIN="mason-wheeler.com"

echo "=== Mason Wheeler Web - VM Setup ==="

# Create application directories
echo "Creating application directories..."
mkdir -p "$APP_DIR" "$DATA_DIR" "${APP_DIR}/site"

# Create a placeholder .env if it doesn't exist
if [ ! -f "${APP_DIR}/.env" ]; then
    touch "${APP_DIR}/.env"
    echo "# Add environment variables here" > "${APP_DIR}/.env"
    echo "Created empty ${APP_DIR}/.env (add secrets here)"
fi

# Install systemd service
echo "Installing systemd service..."
sudo cp "${SCRIPT_DIR}/mason-wheeler.service" /etc/systemd/system/mason-wheeler.service
sudo systemctl daemon-reload
sudo systemctl enable mason-wheeler

# Install Nginx if not present
if ! command -v nginx &>/dev/null; then
    echo "Installing Nginx..."
    sudo apt-get update -y
    sudo apt-get install -y nginx
else
    echo "Nginx already installed."
fi

# Install Certbot if not present
if ! command -v certbot &>/dev/null; then
    echo "Installing Certbot..."
    sudo apt-get update -y
    sudo apt-get install -y certbot python3-certbot-nginx
else
    echo "Certbot already installed."
fi

# Copy Nginx config
echo "Copying Nginx configuration..."
sudo cp "${SCRIPT_DIR}/nginx.conf" /etc/nginx/sites-available/mason-wheeler.conf
sudo ln -sf /etc/nginx/sites-available/mason-wheeler.conf /etc/nginx/sites-enabled/mason-wheeler.conf
sudo rm -f /etc/nginx/sites-enabled/default

# Test and reload Nginx
sudo nginx -t
sudo systemctl reload nginx
sudo systemctl enable nginx

# Obtain SSL certificate
echo "Obtaining SSL certificate..."
sudo certbot --nginx -d "$DOMAIN" -d "www.${DOMAIN}" \
    --non-interactive --agree-tos --email "admin@${DOMAIN}"

# Reload Nginx with SSL
sudo nginx -t
sudo systemctl reload nginx

echo "=== Setup complete ==="
echo "The application service is enabled but not started (no binary yet)."
echo "Push to main to trigger a deploy via GitHub Actions."
