#!/usr/bin/env bash
# SQLite backup script — copies DB to Azure Blob Storage
set -euo pipefail

DB_PATH="/home/azureuser/app/data/app.db"
BACKUP_DIR="/home/azureuser/app/data/backups"
CONTAINER="mason-wheeler-backups"
STORAGE_ACCOUNT="masonwheelerstorage"
DATE=$(date +%Y-%m-%d_%H%M)

mkdir -p "$BACKUP_DIR"

# Use SQLite's backup API (safe for WAL mode)
sqlite3 "$DB_PATH" ".backup '$BACKUP_DIR/app-$DATE.db'"

# Compress
gzip "$BACKUP_DIR/app-$DATE.db"

# Upload to Azure Blob (if az cli available and logged in)
if command -v az &>/dev/null; then
    az storage blob upload \
        --account-name "$STORAGE_ACCOUNT" \
        --container-name "$CONTAINER" \
        --name "backups/app-$DATE.db.gz" \
        --file "$BACKUP_DIR/app-$DATE.db.gz" \
        --auth-mode login 2>/dev/null || echo "Azure upload skipped"
fi

# Keep only last 7 local backups
ls -t "$BACKUP_DIR"/*.gz 2>/dev/null | tail -n +8 | xargs rm -f 2>/dev/null || true

echo "Backup complete: app-$DATE.db.gz"
