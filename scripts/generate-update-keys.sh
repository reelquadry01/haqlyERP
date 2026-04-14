#!/usr/bin/env bash
# Author: Quadri Atharu
# Generate Ed25519 keypair for Tauri auto-update signing
# Run from project root: bash scripts/generate-update-keys.sh

set -euo pipefail

KEYS_DIR="./keys"
PRIVATE_KEY="$KEYS_DIR/update-private.pem"
PUBLIC_KEY="$KEYS_DIR/update-public.pem"
TAURI_CONF="./src-tauri/tauri.conf.json"

mkdir -p "$KEYS_DIR"

if [ -f "$PRIVATE_KEY" ]; then
    echo "WARNING: $PRIVATE_KEY already exists. Backing up..."
    mv "$PRIVATE_KEY" "${PRIVATE_KEY}.bak.$(date +%s)"
fi

echo "Generating Ed25519 keypair for Tauri updater signing..."

openssl genpkey -algorithm Ed25519 -out "$PRIVATE_KEY"

openssl pkey -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY"

PUBLIC_KEY_BASE64=$(openssl pkey -in "$PRIVATE_KEY" -pubout | grep -v "-----" | tr -d '\n')

echo ""
echo "=== Keys Generated ==="
echo "Private key: $PRIVATE_KEY  (KEEP SECRET - never commit to git)"
echo "Public key:  $PUBLIC_KEY"
echo ""
echo "Public key (base64, for tauri.conf.json):"
echo "$PUBLIC_KEY_BASE64"
echo ""
echo "Update tauri.conf.json plugins.updater.pubkey with the value above."
echo "Set UPDATES_PRIVATE_KEY_PATH=$PRIVATE_KEY in your .env"
echo ""

if [ -f ".gitignore" ]; then
    if ! grep -q "keys/" .gitignore 2>/dev/null; then
        echo "keys/" >> .gitignore
        echo "Added keys/ to .gitignore"
    fi
fi

echo "Done."
