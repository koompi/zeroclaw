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

# ── Config Template Generation ──────────────────────────
# Ensure the config directory in home exists
mkdir -p /home/zeroclaw/.zeroclaw

if [ -f /usr/local/share/zeroclaw/config.template.toml ]; then
    echo "Entrypoint: Generating config.toml for user zeroclaw..."
    # Replace variables in the template
    # We use | instead of / for sed to handle URLs in the API key
    cat /usr/local/share/zeroclaw/config.template.toml | \
        sed "s|\${ZEROCLAW_API_KEY}|$ZEROCLAW_API_KEY|g" | \
        sed "s|\${ZEROCLAW_TELEGRAM_BOT_TOKEN}|$ZEROCLAW_TELEGRAM_BOT_TOKEN|g" | \
        sed "s|\${ZEROCLAW_TELEGRAM_CHAT_ID:-}|$ZEROCLAW_TELEGRAM_CHAT_ID|g" | \
        sed "s|\${ZEROCLAW_TELEGRAM_ALLOWED_USERS}|$ZEROCLAW_TELEGRAM_ALLOWED_USERS|g" \
        > /home/zeroclaw/.zeroclaw/config.toml
    
    # Debug: Print the generated file (excluding sensitive keys)
    echo "Entrypoint: Generated config.toml content (redacted):"
    cat /home/zeroclaw/.zeroclaw/config.toml | sed 's/key = ".*"/key = "REDACTED"/' | sed 's/token = ".*"/token = "REDACTED"/'
    
    # Also sync to data dir for persistence
    cp /home/zeroclaw/.zeroclaw/config.toml /zeroclaw-data/.zeroclaw/config.toml
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

# ── Auto-propagate additional user env vars → workspace .env ──
# Any Docker env var that isn't a system var or already-handled ZeroClaw var
# gets written to .env so skills can read it (e.g. RIVERBASE_API_URL, DATABASE_URL, etc.)
# Excluded prefixes: ZEROCLAW_, KCONSOLE_, KSTORAGE_, ANTHROPIC_, API_KEY, and system vars.
SKIP_VARS="^(PATH|HOME|HOSTNAME|TERM|SHELL|PWD|SHLVL|OLDPWD|_|LANG|LC_ALL|LC_CTYPE|LC_MESSAGES|LC_NUMERIC|LC_TIME|GOSU_VERSION|ZEROCLAW_|KCONSOLE_|KSTORAGE_|ANTHROPIC_|API_KEY$|ZEROCLAW_DATA|DEBIAN_FRONTEND|GPG_KEY|PYTHON_|GOPATH|CARGO_|RUSTUP_|NODE_|NPM_|BUN_|JAVA_)"

while IFS='=' read -r key value; do
    # Skip vars matching excluded patterns
    if echo "$key" | grep -qE "$SKIP_VARS"; then
        continue
    fi
    # Skip empty keys or values
    if [ -z "$key" ] || [ -z "$value" ]; then
        continue
    fi
    # Only accept valid env var names (letters, digits, underscore; must start with letter or _)
    if ! echo "$key" | grep -qE '^[A-Za-z_][A-Za-z0-9_]*$'; then
        continue
    fi
    # Write to .env if not already present
    if ! grep -q "^${key}=" "$ENV_FILE" 2>/dev/null; then
        echo "${key}=\"${value}\"" >> "$ENV_FILE"
    fi
done < <(env)

echo "Entrypoint: Additional user env vars propagated to $ENV_FILE"

# Export API keys as env vars so skills/shell commands can use $KCONSOLE_API_KEY
# directly without needing to parse .env files
if [ -f "$ENV_FILE" ]; then
    set -a
    . "$ENV_FILE"
    set +a
fi

# Final ownership fix — catch any files created/modified above (e.g. .env, config.toml)
chown -R zeroclaw:zeroclaw /zeroclaw-data
chown -R zeroclaw:zeroclaw /home/zeroclaw

# If the first argument is a command not found in PATH, assume it's a subcommand of zeroclaw
if ! command -v "$1" >/dev/null 2>&1; then
    set -- zeroclaw "$@"
fi

# Drop from root to zeroclaw user via gosu and run in a loop
# If the process exits, it will be restarted by the loop.
# This allows the 'restart' command within ZeroClaw to work.
if [[ "$*" == *"daemon"* ]]; then
    echo "Entrypoint: Starting ZeroClaw Daemon in watch-loop..."
    while true; do
        gosu zeroclaw zeroclaw daemon
        EXIT_CODE=$?
        echo "Entrypoint: ZeroClaw exited with code $EXIT_CODE. Restarting in 1s..."
        sleep 1
    done
else
    exec gosu zeroclaw "$@"
fi
