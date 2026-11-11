#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="/opt/sipesantren/backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

sqlite3 /opt/sipesantren/data/sipesantren.db ".backup '$BACKUP_DIR/sipesantren.db'"
rsync -a /opt/sipesantren/data/private_photos/ "$BACKUP_DIR/private_photos/"

find /opt/sipesantren/backups -mindepth 1 -maxdepth 1 -type d -mtime +30 -exec rm -rf {} +
