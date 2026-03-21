#!/usr/bin/env bash
#
# One-time VM setup for properties.mason-wheeler.com
# Run as azureuser with sudo privileges.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="/home/azureuser/app"
DATA_DIR="${APP_DIR}/data"
DOMAIN="properties.mason-wheeler.com"

echo "=== Mason Wheeler Properties - VM Setup ==="

# Create application directories
echo "Creating application directories..."
mkdir -p "$APP_DIR" "$DATA_DIR" "${APP_DIR}/site"

# Install Azure CLI if not present
if ! command -v az &>/dev/null; then
    echo "Installing Azure CLI..."
    curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash
else
    echo "Azure CLI already installed."
fi

# Login using the VM's managed identity
echo "Logging in with managed identity..."
az login --identity

# Pull ALL secrets from Key Vault and write .env
echo "Pulling secrets from Key Vault..."
STRIPE_SECRET_KEY=$(az keyvault secret show --vault-name mason-wheeler-kv --name stripe-secret-key --query value -o tsv)
STRIPE_PUBLIC_KEY=$(az keyvault secret show --vault-name mason-wheeler-kv --name stripe-publishable-key --query value -o tsv)
SESSION_SECRET=$(az keyvault secret show --vault-name mason-wheeler-kv --name session-secret --query value -o tsv)
STRIPE_WEBHOOK_SECRET=$(az keyvault secret show --vault-name mason-wheeler-kv --name stripe-webhook-secret --query value -o tsv)
ADMIN_EMAIL=$(az keyvault secret show --vault-name mason-wheeler-kv --name admin-email --query value -o tsv)
ADMIN_PASSWORD=$(az keyvault secret show --vault-name mason-wheeler-kv --name admin-password --query value -o tsv)
ADMIN_NAME=$(az keyvault secret show --vault-name mason-wheeler-kv --name admin-name --query value -o tsv)

cat > "${APP_DIR}/.env" <<EOF
STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
STRIPE_PUBLIC_KEY=${STRIPE_PUBLIC_KEY}
SESSION_SECRET=${SESSION_SECRET}
STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}
DATABASE_PATH=${DATA_DIR}/app.db
LEPTOS_SITE_ROOT=${APP_DIR}/site
LEPTOS_SITE_ADDR=127.0.0.1:3000
ADMIN_EMAIL=${ADMIN_EMAIL}
ADMIN_PASSWORD=${ADMIN_PASSWORD}
ADMIN_NAME=${ADMIN_NAME}
EOF

chmod 600 "${APP_DIR}/.env"
echo "Wrote secrets to ${APP_DIR}/.env"

# Install sqlite3 if not present
if ! command -v sqlite3 &>/dev/null; then
    echo "Installing sqlite3..."
    sudo apt-get update -y
    sudo apt-get install -y sqlite3
else
    echo "sqlite3 already installed."
fi

# Install backup script and crontab
echo "Installing backup script..."
cp "${SCRIPT_DIR}/backup.sh" "${APP_DIR}/backup.sh"
chmod +x "${APP_DIR}/backup.sh"
mkdir -p "${DATA_DIR}/backups"

echo "Installing backup crontab..."
crontab "${SCRIPT_DIR}/crontab.txt"

# Install systemd service
echo "Installing systemd service..."
sudo cp "${SCRIPT_DIR}/mason-wheeler.service" /etc/systemd/system/mason-wheeler-app.service
sudo systemctl daemon-reload
sudo systemctl enable mason-wheeler-app

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
sudo cp "${SCRIPT_DIR}/nginx.conf" "/etc/nginx/sites-available/${DOMAIN}"
sudo ln -sf "/etc/nginx/sites-available/${DOMAIN}" "/etc/nginx/sites-enabled/${DOMAIN}"

# Test and reload Nginx
sudo nginx -t
sudo systemctl reload nginx
sudo systemctl enable nginx

# Obtain SSL certificate
echo "Obtaining SSL certificate..."
sudo certbot --nginx -d "$DOMAIN" \
    --non-interactive --agree-tos --email "masonwheeler@fieldflow.us"

# Reload Nginx with SSL
sudo nginx -t
sudo systemctl reload nginx

echo "=== Setup complete ==="
echo "Push to master to trigger a deploy via GitHub Actions."
