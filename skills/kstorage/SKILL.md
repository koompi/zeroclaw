---
name: kstorage
description: Upload files to KStorage (KOOMPI's S3-compatible object storage) and get public CDN URLs. Use when Boss asks to upload a file, store an asset, get a CDN link, or host media. Also for checking storage usage or managing uploaded files.
---

# KStorage Skill

KOOMPI Cloud's S3/R2-compatible object storage with CDN. Upload any file → get a public URL instantly.

## Config

- **API Key:** Read from `~/.openclaw/workspace/.env` → `KCONSOLE_API_KEY`
- **Org ID:** `69c2b301e996d9081043e879`
- **CDN Base:** `https://storage.koompi.cloud`
- **API Base:** `https://api-kconsole.koompi.cloud`
- **Upload Path:** `/api/orgs/{orgId}/services/kstorage/api/storage`

## Upload Flow (3-Step)

### Step 1 — Get Upload Token
```bash
KEY=$(grep KCONSOLE_API_KEY ~/.openclaw/workspace/.env | cut -d= -f2 | tr -d '"')
FILENAME="my-file.png"
FILESIZE=$(stat -c%s "$FILENAME")

curl -s "https://api-kconsole.koompi.cloud/api/orgs/69c2b301e996d9081043e879/services/kstorage/api/storage/upload-token?filename=$FILENAME&size=$FILESIZE" \
  -H "Authorization: Bearer $KEY"
```

Returns JSON with `uploadUrl` and `key`.

### Step 2 — Upload Binary
```bash
curl -s -X PUT -H "Content-Type: application/octet-stream" \
  --data-binary "@$FILENAME" \
  "$UPLOAD_URL"
```

### Step 3 — Confirm Upload
```bash
curl -s -X POST "https://api-kconsole.koompi.cloud/api/orgs/69c2b301e996d9081043e879/services/kstorage/api/storage/complete" \
  -H "Authorization: Bearer $KEY" \
  -H "Content-Type: application/json" \
  -d '{"key": "THE_KEY_FROM_STEP_1"}'
```

### Public URL
```
https://storage.koompi.cloud/{key}
```

## One-Liner Script

For quick uploads, combine all steps:

```bash
upload_to_kstorage() {
  local FILE="$1"
  local KEY=$(grep KCONSOLE_API_KEY ~/.openclaw/workspace/.env | cut -d= -f2 | tr -d '"')
  local FNAME=$(basename "$FILE")
  local FSIZE=$(stat -c%s "$FILE")
  local ORG="69c2b301e996d9081043e879"
  local BASE="https://api-kconsole.koompi.cloud/api/orgs/$ORG/services/kstorage/api/storage"

  # Step 1: Get token
  local RESP=$(curl -s "$BASE/upload-token?filename=$FNAME&size=$FSIZE" -H "Authorization: Bearer $KEY")
  local UPLOAD_URL=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['uploadUrl'])")
  local KEY_NAME=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['key'])")

  # Step 2: Upload
  curl -s -X PUT -H "Content-Type: application/octet-stream" --data-binary "@$FILE" "$UPLOAD_URL"

  # Step 3: Confirm
  curl -s -X POST "$BASE/complete" -H "Authorization: Bearer $KEY" -H "Content-Type: application/json" -d "{\"key\":\"$KEY_NAME\"}"

  echo "https://storage.koompi.cloud/$KEY_NAME"
}
```

## Riverbase Upload (HQ Endpoint)

For BIS/Riverbase product images, use the HQ endpoint for auto-resizing (max 1920px):

```bash
# HQ upload (auto-resize to max 1920px width)
curl -s -X POST "https://lite-api.riverbase.org/uploads/s3/hq" \
  -H "Authorization: Bearer $BIS_TOKEN" \
  -F "file=@image.jpg"
```

Returns the CDN URL directly.

## Common Use Cases

- **Generated images** from KConsole AI Gateway → upload to KStorage for sharing
- **Social media assets** (logos, covers, banners) → CDN links for embedding
- **Product images** for Riverbase/BIS shops
- **Any file** Boss wants a shareable link for

## Notes

- Supports any file type (images, videos, PDFs, zips)
- No explicit size limit documented — large files may need longer timeouts
- Files are publicly accessible at the CDN URL
- No delete API documented yet
