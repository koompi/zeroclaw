#!/bin/bash
set -e

# ZeroClaw Runtime Entrypoint
# Handles volume permission mapping and KConsole AI Gateway key propagation.

# Fix permissions on /zeroclaw-data.
# Entrypoint runs as root (gosu drops to zeroclaw at the end), so no sudo needed.
# This is necessary because mounted volumes often default to root ownership.
echo "Entrypoint: Ensuring /zeroclaw-data is writable by 'zeroclaw'..."

# Ensure all required subdirectories exist BEFORE chown (volume mount wipes build-time contents)
mkdir -p /zeroclaw-data/.zeroclaw \
         /zeroclaw-data/workspace/skills \
         /zeroclaw-data/workspace/cron \
         /zeroclaw-data/.claude

# Copy default config if missing (volume mount wipes the build-time copy)
if [ ! -f /zeroclaw-data/.zeroclaw/config.toml ] && [ -f /usr/local/share/zeroclaw/config.toml ]; then
    cp /usr/local/share/zeroclaw/config.toml /zeroclaw-data/.zeroclaw/config.toml
fi
if [ ! -f /zeroclaw-data/.claude.json ] && [ -f /usr/local/share/zeroclaw/.claude.json ]; then
    cp /usr/local/share/zeroclaw/.claude.json /zeroclaw-data/.claude.json
fi

# Copy bundled skills if workspace skills dir is empty
if [ -d /usr/local/share/zeroclaw/skills ] && [ -z "$(ls -A /zeroclaw-data/workspace/skills/ 2>/dev/null)" ]; then
    cp -r /usr/local/share/zeroclaw/skills/* /zeroclaw-data/workspace/skills/ 2>/dev/null || true
fi

# Fix ownership of everything
chown -R zeroclaw:zeroclaw /zeroclaw-data

# Ensure /home/zeroclaw is also owned correctly (just in case)
if [ -d /home/zeroclaw ] && [ -d /home/zeroclaw ]; then
    FIND_HOME_CMD="find /home/zeroclaw -not -user zeroclaw -not -group zeroclaw -print -quit"
    if [ -n "$($FIND_HOME_CMD)" ]; then
        chown -R zeroclaw:zeroclaw /home/zeroclaw
    fi
fi

# ── KConsole AI Gateway key propagation ──────────────────────
# Users provide a single KCONSOLE_API_KEY (or ZEROCLAW_API_KEY).
# We propagate it to both ZeroClaw's config and Claude Code's env.
KCONSOLE_KEY="${KCONSOLE_API_KEY:-${ZEROCLAW_API_KEY:-}}"
if [ -n "$KCONSOLE_KEY" ]; then
    echo "Entrypoint: Configuring KConsole AI Gateway key..."
    # Set for ZeroClaw (it reads API_KEY env or config.toml api_key)
    export API_KEY="$KCONSOLE_KEY"
    export ZEROCLAW_API_KEY="$KCONSOLE_KEY"
    # Set for Claude Code (it reads ANTHROPIC_AUTH_TOKEN as x-api-key)
    export ANTHROPIC_AUTH_TOKEN="$KCONSOLE_KEY"
    # Inject into ZeroClaw config.toml if it has an empty api_key
    if [ -f /zeroclaw-data/.zeroclaw/config.toml ]; then
        sed -i "s|^api_key = \"\"$|api_key = \"$KCONSOLE_KEY\"|" /zeroclaw-data/.zeroclaw/config.toml
    fi
fi

# ── Skill API keys → workspace .env ──────────────────────────
# Skills read API keys from ~/.openclaw/workspace/.env (or /zeroclaw-data/workspace/.env).
# We write them here so skills can use them at runtime.
ENV_FILE="/zeroclaw-data/workspace/.env"
touch "$ENV_FILE"

# KCONSOLE_AI_KEY = AI gateway key (same as the main KCONSOLE_API_KEY)
if [ -n "$KCONSOLE_KEY" ]; then
    grep -q "^KCONSOLE_AI_KEY=" "$ENV_FILE" 2>/dev/null && \
        sed -i "s|^KCONSOLE_AI_KEY=.*|KCONSOLE_AI_KEY=\"$KCONSOLE_KEY\"|" "$ENV_FILE" || \
        echo "KCONSOLE_AI_KEY=\"$KCONSOLE_KEY\"" >> "$ENV_FILE"
fi

# KCONSOLE_API_KEY = org-level API token for KConsole platform (services, deployments)
ORG_KEY="${KCONSOLE_API_TOKEN:-${KCONSOLE_API_KEY:-}}"
if [ -n "$ORG_KEY" ]; then
    grep -q "^KCONSOLE_API_KEY=" "$ENV_FILE" 2>/dev/null && \
        sed -i "s|^KCONSOLE_API_KEY=.*|KCONSOLE_API_KEY=\"$ORG_KEY\"|" "$ENV_FILE" || \
        echo "KCONSOLE_API_KEY=\"$ORG_KEY\"" >> "$ENV_FILE"
fi

# KCONSOLE_API_URL = KConsole API base URL
API_URL="${KCONSOLE_API_URL:-https://api-kconsole.koompi.cloud}"
grep -q "^KCONSOLE_API_URL=" "$ENV_FILE" 2>/dev/null && \
    sed -i "s|^KCONSOLE_API_URL=.*|KCONSOLE_API_URL=\"$API_URL\"|" "$ENV_FILE" || \
    echo "KCONSOLE_API_URL=\"$API_URL\"" >> "$ENV_FILE"

# KSTORAGE_API_KEY = KStorage API key (defaults to KCONSOLE_API_KEY if not set separately)
STORAGE_KEY="${KSTORAGE_API_KEY:-${ORG_KEY:-}}"
if [ -n "$STORAGE_KEY" ]; then
    grep -q "^KSTORAGE_API_KEY=" "$ENV_FILE" 2>/dev/null && \
        sed -i "s|^KSTORAGE_API_KEY=.*|KSTORAGE_API_KEY=\"$STORAGE_KEY\"|" "$ENV_FILE" || \
        echo "KSTORAGE_API_KEY=\"$STORAGE_KEY\"" >> "$ENV_FILE"
fi

echo "Entrypoint: Skill env file written to $ENV_FILE"

# Final ownership fix — catch any files created/modified above (e.g. .env, config.toml)
chown -R zeroclaw:zeroclaw /zeroclaw-data

# If the first argument is a command not found in PATH, assume it's a subcommand of zeroclaw
if ! command -v "$1" >/dev/null 2>&1; then
    set -- zeroclaw "$@"
fi

# Drop from root to zeroclaw user via gosu (works with no-new-privileges)
exec gosu zeroclaw "$@"
