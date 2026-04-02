# syntax=docker/dockerfile:1.7

# ── Stage 1: Build ────────────────────────────────────────────
FROM rust:1.93-slim@sha256:9663b80a1621253d30b146454f903de48f0af925c967be48c84745537cd35d8b AS builder

WORKDIR /app

# Install build dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 1. Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/robot-kit/Cargo.toml crates/robot-kit/Cargo.toml
# Create dummy targets declared in Cargo.toml so manifest parsing succeeds.
RUN mkdir -p src benches crates/robot-kit/src \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > benches/agent_benchmarks.rs \
    && echo "pub fn placeholder() {}" > crates/robot-kit/src/lib.rs
RUN --mount=type=cache,id=zeroclaw-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=zeroclaw-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=zeroclaw-target,target=/app/target,sharing=locked \
    cargo build --release --locked
RUN rm -rf src benches crates/robot-kit/src

# 2. Copy only build-relevant source paths (avoid cache-busting on docs/tests/scripts)
COPY src/ src/
COPY benches/ benches/
COPY crates/ crates/
COPY firmware/ firmware/
COPY web/ web/
# Keep release builds resilient when frontend dist assets are not prebuilt in Git.
RUN mkdir -p web/dist && \
    if [ ! -f web/dist/index.html ]; then \
      printf '%s\n' \
        '<!doctype html>' \
        '<html lang="en">' \
        '  <head>' \
        '    <meta charset="utf-8" />' \
        '    <meta name="viewport" content="width=device-width,initial-scale=1" />' \
        '    <title>ZeroClaw Dashboard</title>' \
        '  </head>' \
        '  <body>' \
        '    <h1>ZeroClaw Dashboard Unavailable</h1>' \
        '    <p>Frontend assets are not bundled in this build. Build the web UI to populate <code>web/dist</code>.</p>' \
        '  </body>' \
        '</html>' > web/dist/index.html; \
    fi
RUN --mount=type=cache,id=zeroclaw-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=zeroclaw-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=zeroclaw-target,target=/app/target,sharing=locked \
    cargo build --release --locked && \
    cp target/release/zeroclaw /app/zeroclaw && \
    strip /app/zeroclaw

# Prepare runtime directory structure and default config inline (no extra stage)
RUN mkdir -p /zeroclaw-data/.zeroclaw /zeroclaw-data/workspace /zeroclaw-data/.claude && \
    cat > /zeroclaw-data/.zeroclaw/config.toml <<EOF
workspace_dir = "/zeroclaw-data/workspace"
config_path = "/zeroclaw-data/.zeroclaw/config.toml"
# KConsole AI Gateway - users provide their KConsole API key at runtime
# via -e KCONSOLE_API_KEY="your-key" or -e ZEROCLAW_API_KEY="your-key"
api_key = ""
default_provider = "anthropic"
default_model = "anthropic/claude-sonnet-4-20250514"
default_temperature = 0.7

# Route through KConsole AI Gateway (Anthropic-compatible endpoint)
[model_providers.kconsole]
name = "anthropic"
base_url = "https://ai.koompi.cloud"

[gateway]
port = 42617
host = "[::]"
allow_public_bind = true
EOF
# Create onboarding complete marker
RUN echo '{"hasCompletedOnboarding": true}' > /zeroclaw-data/.claude.json

# ── Stage 2: Development Runtime (Debian) ────────────────────
FROM debian:trixie-slim@sha256:f6e2cfac5cf956ea044b4bd75e6397b4372ad88fe00908045e9a0d21712ae3ba AS dev

# Install essential runtime dependencies and gosu (for privilege dropping)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    nodejs \
    npm \
    gosu \
    git \
    && npm install -g @anthropic-ai/claude-code \
    && rm -rf /var/lib/apt/lists/*

# Create zeroclaw user (no sudo needed - entrypoint runs as root and drops via gosu)
RUN useradd -m -u 1000 -s /bin/bash zeroclaw

COPY --from=builder /zeroclaw-data /zeroclaw-data
COPY --from=builder /app/zeroclaw /usr/local/bin/zeroclaw

# Set ownership to zeroclaw user
RUN chown -R zeroclaw:zeroclaw /zeroclaw-data && \
    chmod +x /usr/local/bin/zeroclaw

# Overwrite minimal config with DEV template (Ollama defaults)
COPY dev/config.template.toml /zeroclaw-data/.zeroclaw/config.toml
RUN chown zeroclaw:zeroclaw /zeroclaw-data/.zeroclaw/config.toml

# Bundle built-in skills (KConsole, KStorage, AI, etc.)
COPY skills/ /zeroclaw-data/workspace/skills/
RUN chown -R zeroclaw:zeroclaw /zeroclaw-data/workspace/skills/

# Stash defaults outside the volume path so entrypoint can restore them on fresh mounts
RUN mkdir -p /usr/local/share/zeroclaw/skills && \
    cp /zeroclaw-data/.zeroclaw/config.toml /usr/local/share/zeroclaw/config.toml && \
    cp /zeroclaw-data/.claude.json /usr/local/share/zeroclaw/.claude.json && \
    cp -r /zeroclaw-data/workspace/skills/* /usr/local/share/zeroclaw/skills/ 2>/dev/null || true

# Environment setup
# Use consistent workspace path
ENV ZEROCLAW_WORKSPACE=/zeroclaw-data/workspace
ENV HOME=/home/zeroclaw
# Defaults for local dev (Ollama) - matches config.template.toml
ENV PROVIDER="ollama"
ENV ZEROCLAW_MODEL="llama3.2"
ENV ZEROCLAW_GATEWAY_PORT=42617

# KConsole AI Gateway configuration for Claude Code
# Claude Code uses ANTHROPIC_BASE_URL to talk to the Anthropic Messages API
# Our gateway provides /v1/messages endpoint that translates to upstream providers
ENV ANTHROPIC_BASE_URL="https://ai.koompi.cloud"
ENV API_TIMEOUT_MS="3000000"
# Skill API keys (injected at deploy time via KConsole env vars)
# KCONSOLE_AI_KEY  = same as KCONSOLE_API_KEY (AI gateway uses the same key)
# KCONSOLE_API_TOKEN = org-level API token for KConsole platform API (services, deployments)
# KCONSOLE_API_URL   = KConsole API base URL
# KSTORAGE_API_KEY   = org-level API key for KStorage (file upload/download)
ENV KCONSOLE_API_URL="https://api-kconsole.koompi.cloud"
# Model mapping: Claude model names → KConsole upstream models (GLM/Gemini)
# These are used by Claude Code internally for model selection
ENV ANTHROPIC_DEFAULT_HAIKU_MODEL="glm-4.7-flash"
ENV ANTHROPIC_DEFAULT_SONNET_MODEL="glm-5"
ENV ANTHROPIC_DEFAULT_OPUS_MODEL="glm-5.1"
ENV CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC="1"
# Pass your KConsole API key at runtime using:
#   -e ANTHROPIC_AUTH_TOKEN="your-kconsole-api-key"
# Or for ZeroClaw directly:
#   -e KCONSOLE_API_KEY="your-kconsole-api-key"

# Note: API_KEY is intentionally NOT set here.
# Users provide their KConsole API key at runtime.

WORKDIR /zeroclaw-data
EXPOSE 42617
# Fix: Ensure volumes are writable by fixing permissions at runtime first
COPY dev/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
# Entrypoint runs as root, fixes volume perms, then drops to zeroclaw via gosu
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["daemon"]

# ── Stage 3: Production Runtime (Debian) ─────────────────
FROM debian:trixie-slim AS release

# Install essential runtime dependencies and gosu (for privilege dropping)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    nodejs \
    npm \
    gosu \
    && npm install -g @anthropic-ai/claude-code \
    && rm -rf /var/lib/apt/lists/*

# Create zeroclaw user (no sudo needed - entrypoint runs as root and drops via gosu)
RUN useradd -m -u 1000 -s /bin/bash zeroclaw

COPY --from=builder /app/zeroclaw /usr/local/bin/zeroclaw
COPY --from=builder /zeroclaw-data /zeroclaw-data

# Bundle built-in skills (KConsole, KStorage, AI, etc.)
COPY skills/ /zeroclaw-data/workspace/skills/

# Set ownership to zeroclaw user
RUN chown -R zeroclaw:zeroclaw /zeroclaw-data && \
    chmod +x /usr/local/bin/zeroclaw

# Stash defaults outside the volume path so entrypoint can restore them on fresh mounts
RUN mkdir -p /usr/local/share/zeroclaw/skills && \
    cp /zeroclaw-data/.zeroclaw/config.toml /usr/local/share/zeroclaw/config.toml && \
    cp /zeroclaw-data/.claude.json /usr/local/share/zeroclaw/.claude.json && \
    cp -r /zeroclaw-data/workspace/skills/* /usr/local/share/zeroclaw/skills/ 2>/dev/null || true

# Environment setup
ENV ZEROCLAW_WORKSPACE=/zeroclaw-data/workspace
ENV HOME=/home/zeroclaw
# Default provider and model are set in config.toml, not here,
# so config file edits are not silently overridden
#ENV PROVIDER=
ENV ZEROCLAW_GATEWAY_PORT=42617

# KConsole AI Gateway configuration for Claude Code
ENV ANTHROPIC_BASE_URL="https://ai.koompi.cloud"
ENV API_TIMEOUT_MS="3000000"
# Skill API keys (injected at deploy time via KConsole env vars)
ENV KCONSOLE_API_URL="https://api-kconsole.koompi.cloud"
ENV ANTHROPIC_DEFAULT_HAIKU_MODEL="glm-4.7-flash"
ENV ANTHROPIC_DEFAULT_SONNET_MODEL="glm-5"
ENV ANTHROPIC_DEFAULT_OPUS_MODEL="glm-5.1"
ENV CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC="1"
# Pass your KConsole API key at runtime using:
#   -e ANTHROPIC_AUTH_TOKEN="your-kconsole-api-key"

# KCONSOLE_API_KEY / ZEROCLAW_API_KEY must be provided at runtime!

WORKDIR /zeroclaw-data
EXPOSE 42617
# Fix: Ensure volumes are writable by fixing permissions at runtime first
COPY dev/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
# Entrypoint runs as root, fixes volume perms, then drops to zeroclaw via gosu
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["daemon"]
