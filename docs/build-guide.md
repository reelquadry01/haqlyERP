# HAQLY ERP Build Guide
# Author: Quadri Atharu

## Prerequisites

- Rust 1.77+ with `cargo`
- Node.js 18+ with `npm`
- PostgreSQL 15+ running locally
- Tauri CLI: `cargo install tauri-cli`
- Windows SDK (for Windows builds)

## Windows Code Signing Setup

### 1. Obtain a Code Signing Certificate

Purchase an Authenticode code signing certificate from a trusted Certificate Authority:
- **DigiCert** — https://www.digicert.com/signing/code-signing-certificates
- **Sectigo** — https://sectigo.com/ssl-certificates-tls/code-signing
- **GlobalSign** — https://www.globalsign.com/en/code-signing-certificate

For EV (Extended Validation) certificates, a USB token is typically required.

### 2. Install the Certificate

1. Import the certificate into the Windows Certificate Store:
   - Double-click the `.pfx` file
   - Follow the wizard, selecting "Local Machine" store
   - Select "Automatically select the certificate store"
2. Find the certificate thumbprint:
   ```powershell
   certutil -store My | findstr -i "sha1"
   ```
3. Set the thumbprint in your `.env`:
   ```
   WINDOWS_CERTIFICATE_THUMBPRINT=<your-thumbprint>
   ```

### 3. Configure tauri.conf.json

The Windows signing configuration is in `src-tauri/tauri.conf.json` under `bundle.windows`:
```json
{
  "certificateThumbprint": "<thumbprint-or-env-var>",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

Set `certificateThumbprint` to your certificate thumbprint or use the environment variable.

## Auto-Update Signing Keys

### 1. Generate Ed25519 Keypair

```bash
bash scripts/generate-update-keys.sh
```

This creates:
- `keys/update-private.pem` — Private key (NEVER commit this)
- `keys/update-public.pem` — Public key

### 2. Configure tauri.conf.json

Copy the base64 public key output and set it in `src-tauri/tauri.conf.json`:
```json
{
  "plugins": {
    "updater": {
      "pubkey": "<base64-public-key-here>"
    }
  }
}
```

### 3. Set Environment Variable

In your `.env`:
```
UPDATES_PRIVATE_KEY_PATH=./keys/update-private.pem
```

When running `tauri build`, the Tauri CLI reads this env var to sign the update bundle.

## Building for Production

### Development Build

```bash
cd src-tauri
cargo tauri dev
```

### Production Build (Windows .msi / .exe)

```bash
cd src-tauri
cargo tauri build
```

Output artifacts are in `src-tauri/target/release/bundle/`:
- `msi/HAQLY ERP_0.1.0_x64_en-US.msi`
- `nsis/HAQLY ERP_0.1.0_x64-setup.exe`

### Build with Signing

Ensure environment variables are set before building:
```powershell
$env:UPDATES_PRIVATE_KEY_PATH = "./keys/update-private.pem"
$env:WINDOWS_CERTIFICATE_THUMBPRINT = "<your-thumbprint>"
cargo tauri build
```

## Creating a Release with Auto-Update

### 1. Build and Sign

```bash
cargo tauri build
```

The Tauri CLI automatically:
- Signs the `.msi`/`.exe` with your Windows code signing certificate
- Signs the update bundle (`.tar.gz` + `.sig`) with your Ed25519 private key

### 2. Upload Artifacts

Upload the following to your release server (`https://releases.haqly.com/erp/desktop/`):
- The `.msi` or `.exe` installer
- The update bundle: `haqly-erp_x64.tar.gz`
- The signature file: `haqly-erp_x64.tar.gz.sig`

### 3. Update the JSON Manifest

Create/update the release manifest at the endpoint matching your `tauri.conf.json` `updater.endpoints` pattern:

```json
{
  "version": "0.2.0",
  "notes": "Bug fixes and improvements",
  "pub_date": "2026-04-14T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<contents-of-.sig-file>",
      "url": "https://releases.haqly.com/erp/desktop/windows/x64/0.2.0/haqly-erp_x64.tar.gz"
    }
  }
}
```

### 4. Verify Auto-Update

1. Install the previous version of HAQLY ERP
2. Launch the application
3. The Tauri updater checks the endpoint and prompts if a new version is available
4. Download and install — the Ed25519 signature is verified against the pubkey in `tauri.conf.json`

## Troubleshooting

### Code Signing Fails
- Verify certificate thumbprint matches: `certutil -store My`
- Ensure timestamp URL is reachable: `http://timestamp.digicert.com`
- For EV certificates, ensure the USB token is plugged in

### Update Signature Invalid
- Verify the private key matches the public key in `tauri.conf.json`
- Check `UPDATES_PRIVATE_KEY_PATH` points to the correct `.pem` file
- Regenerate keys if needed: `bash scripts/generate-update-keys.sh`

### Build Errors
- Run `cargo check` in `src-server/` first to verify Rust compilation
- Run `npm run build` in project root to verify frontend
- Check PostgreSQL is running and `DATABASE_URL` is correct
