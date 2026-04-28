# syntax=docker/dockerfile:1.7

# ── Stage 1: Runtime (Debian) ────────────────────
FROM debian:trixie-slim AS dev

# Install essential runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    nodejs \
    npm \
    gosu \
    git \
    chromium \
    python3 \
    python3-pip \
    python3-venv \
    zip \
    unzip \
    poppler-utils \
    jq \
    sed \
    && rm -rf /var/lib/apt/lists/*

# Download and install ZeroClaw v0.7.3 binary
RUN curl -L https://github.com/zeroclaw-labs/zeroclaw/releases/download/v0.7.3/zeroclaw-x86_64-unknown-linux-gnu.tar.gz | tar -xz -C /usr/local/bin/

# Create non-root user
RUN useradd -m -s /bin/bash zeroclaw

# Binary permissions (the tar usually extracts to 'zeroclaw')
RUN chmod +x /usr/local/bin/zeroclaw

# Copy our custom skills
COPY skills/ /usr/local/share/zeroclaw/skills/

# Useuv for fast python dependency management
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.list/bin:${PATH}"

# Pre-install common tools for the agent
RUN pip3 install --break-system-packages openpyxl pandas xlsxwriter pdfplumber reportlab Pillow matplotlib qrcode python-barcode

# Workdir setup
RUN mkdir -p /zeroclaw-data && chown -R zeroclaw:zeroclaw /zeroclaw-data
WORKDIR /zeroclaw-data

# Entrypoint setup
COPY dev/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
COPY dev/config.template.toml /usr/local/share/zeroclaw/config.template.toml
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 42617
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["daemon"]
