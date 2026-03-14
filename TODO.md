# TODO — ZeroClaw Meta-Orchestration + KOOMPI Cloud Integration

**Owner:** Hangsia
**Priority:** High
**Target:** Sentinel meta-orchestrator → multi-agent workforce → KOOMPI Cloud deployment pipeline

---

## What is Meta-Orchestration?

ZeroClaw now has a **meta-orchestration layer** — Sentinel doesn't just run one agent, it commands a team of 8 specialized agents organized into 3 clusters. Think of it like a CEO (Sentinel) delegating work to department heads who each have their own tools and expertise.

### The Agent Roster

| Cluster | Agent | Code Name | Role |
|---------|-------|-----------|------|
| **Builder** | System Architect | **Archon** | Designs architecture, plans systems |
| **Builder** | UI/UX Expert | **Prism** | Specs interfaces, designs UX flows |
| **Builder** | Fullstack Engineer | **Forge** | Writes code, builds features, deploys |
| **Business** | Biz Dev | **Nexus** | Business strategy, partnerships |
| **Business** | Marketing | **Echo** | Content, campaigns, messaging |
| **Business** | Sales | **Closer** | Outreach, proposals, deal closure |
| **Research** | Researcher | **Oracle** | Deep research, analysis, fact-finding |
| **Research** | Khmer Expert | **Veasna** | Khmer language, culture, localization |

### How It Works

```
User: "Build me a landing page for my coffee shop"

Sentinel (orchestrator) receives the message
  → Decomposes into subtasks
  → Spawns Archon (architect): "Design the page structure"
  → Spawns Prism (UI/UX): "Spec the visual design"
  → Spawns Forge (engineer): "Build it using React+Vite"
  → Collects results, synthesizes, responds to user
```

### Key Subsystems (New Code)

**1. Orchestrator** (`src/orchestrator/`)
- **SubagentRegistry**: Tracks all spawned agents — who's running, who finished, parent-child relationships
- **SpawnPolicy**: Controls which agents can be spawned, max depth (prevents infinite recursion), max concurrency per parent
- **DepthConfig**: Max spawn depth = 10, max children per parent = 5
- **Announce system**: Children notify parents on completion (no polling), with retry + backoff
- **SessionKey**: Deterministic key pattern `agent:<id>:<type>:<uuid>` for routing

**2. Context Engine** (`src/context/`)
- **TokenBudget**: Manages context window — max tokens, reserve for response, compaction threshold (80%)
- **ContextAssembler**: Ranks segments by priority (system > user > assistant > tool), packs into budget
- **ContextCompactor**: When context gets too large, removes oldest messages first, keeps system + recent messages, generates summary of removed content
- Each session gets its own context — agents don't leak context to each other

**3. Canvas** (`src/canvas/`)
- Renders interactive UI through the gateway: HTML pages, forms, dashboards, tables, markdown
- Gateway serves canvas at `/__zeroclaw__/canvas/<session_id>` with token-based access control
- Agents can render forms and receive user actions back (e.g., approval buttons, config forms)
- XSS-protected HTML rendering with Tailwind CSS

**4. Personas** (`src/personas/`)
- Each agent has a personality, system prompt, tool allowlist, model preference, and temperature
- Cost-aware routing: Haiku for planning/review (Archon, Oracle), Sonnet for implementation (Forge, Prism)
- Factory function: `create_persona("forge")` or `create_persona("archon")`

**5. Workspace** (`src/workspace/`)
- Per-user isolated directories: `~/.zeroclaw/users/<user-id>/`
- Each user gets their own `chats/` and `dir-app-XX/` project directories
- Tools validate all file paths stay within user's workspace (no escaping)
- Handles Telegram @usernames, Discord IDs, emails → normalized safe directory names

**6. Delegation Tool** (`src/tools/delegate.rs`)
- The bridge between Sentinel and subagents
- Sentinel calls `delegate(agent="forge", prompt="build X", context="...")`
- Supports agentic mode (full tool-call loop) and non-agentic mode (single LLM call)
- Registers spawns with SubagentRegistry, tracks outcomes, enforces depth limits

**7. Claude Code Tool** (`src/tools/claude_code.rs`)
- Invokes Claude CLI as subprocess for heavy coding tasks
- Modes: Plan, Implement, Review, Fix — each with tailored system prompts
- Cost-aware: Haiku for plan/review, Sonnet for implement
- 5-minute timeout, structured error handling

### Data Flow

```
User message
  → Channel (Telegram/Discord/Slack)
    → Gateway
      → Sentinel (meta-orchestrator)
        → Decomposes task
        → Spawns subagents via DelegateTool
          → Each agent gets: persona prompt + context engine + allowed tools + workspace
          → Agents execute (may spawn sub-subagents up to depth 10)
          → Results flow back via announce system
        → Sentinel synthesizes results
        → Optional: renders Canvas (dashboard, form, report)
      → Response sent back through Channel
```

### Hooks for Orchestration Events

The hook system (`src/hooks/`) now tracks orchestration lifecycle:
- `on_subagent_spawning(parent, params)` — before spawn
- `on_subagent_spawned(parent, child, run_id)` — after successful spawn
- `on_subagent_ended(child, run_id, success, summary)` — completion
- Plus: compaction events, agent start/end, session lifecycle

### Sentinel Config Files (`sentinel/`)

- `SOUL.md` — Sentinel's identity: "Think like a CEO, execute like a machine"
- `AGENTS.md` — Routing rules: task patterns → agent assignments
- `IDENTITY.md` — Display names and handoff formats
- `TOOLS.md` — Per-persona tool access matrix and tech stack enforcement

---

## KOOMPI Cloud Integration

### Overview

Users chat with Sentinel, build apps via Forge/Prism, and deploy directly to KOOMPI Cloud — all from the conversation. Each app gets `app-name.koompi.cloud` automatically, with custom domain support, Baray payment, and KID auth wired in.

```
User: "Build me a landing page for my coffee shop"
  → Sentinel decomposes → Archon designs → Prism specs UI → Forge builds
  → Forge deploys to coffee-shop.koompi.cloud via koompi_cloud tool
  → User: "Add payments" → Forge wires Baray
  → User: "Add login" → Forge wires KID OAuth
  → User: "Use mycoffeeshop.com" → Sentinel configures custom domain
```

---

## Phase 1: KOOMPI Cloud Deploy Tool

- [ ] Create `src/tools/koompi_cloud.rs` implementing `Tool` trait
- [ ] Actions: `deploy`, `status`, `logs`, `rollback`, `delete`
- [ ] Deploy takes a workspace directory, builds it, pushes to KOOMPI Cloud
- [ ] Auto-generate subdomain: `<app-name>.koompi.cloud`
- [ ] Return live URL on successful deploy
- [ ] Support deploy targets: static (HTML/CSS/JS), Next.js, Vite, Python (FastAPI/Flask)
- [ ] Register in `all_tools_with_runtime()` factory
- [ ] Add to Forge's `allowed_tools` in persona

### Deploy Flow
```
1. User says "deploy this"
2. Sentinel routes to Forge
3. Forge calls koompi_cloud tool with action=deploy
4. Tool packages workspace → pushes to KOOMPI Cloud API
5. Cloud provisions container/static hosting
6. Returns: https://app-name.koompi.cloud
7. Sentinel announces URL to user
```

### Config Schema
```toml
[koompi_cloud]
enabled = true
api_url = "https://api.koompi.cloud"
api_key = ""  # or use KID OAuth token
default_region = "sg"  # Singapore
```

---

## Phase 2: Subdomain Management

- [ ] Auto-generate subdomain from project directory name (sanitized)
- [ ] Check subdomain availability before deploy via KOOMPI Cloud API
- [ ] Allow user to specify custom subdomain: `"deploy as my-store"`
- [ ] Subdomain validation: lowercase, alphanumeric + hyphens, 3-63 chars
- [ ] Prevent reserved names: `api`, `admin`, `www`, `mail`, `dash`, `oauth`
- [ ] Store subdomain mapping in user workspace metadata
- [ ] Support listing all user's deployed apps: `koompi_cloud action=list`

---

## Phase 3: Custom Domain

- [ ] Add `domain` action to koompi_cloud tool: `action=domain, domain=mycoffeeshop.com`
- [ ] Flow:
  1. User says "use mycoffeeshop.com for this app"
  2. Forge calls `koompi_cloud action=domain domain=mycoffeeshop.com`
  3. Tool calls KOOMPI Cloud API to register custom domain
  4. API returns DNS records needed (CNAME or A record)
  5. Sentinel tells user: "Point your DNS: CNAME mycoffeeshop.com → app-name.koompi.cloud"
  6. Tool polls for DNS propagation and SSL cert provisioning
  7. Once live: "mycoffeeshop.com is live with HTTPS"
- [ ] Support multiple custom domains per app
- [ ] Auto-provision SSL via Let's Encrypt (KOOMPI Cloud side)
- [ ] Domain verification via DNS TXT record
- [ ] Store domain mappings in user workspace metadata

---

## Phase 4: Baray Payment Integration

- [ ] Create `src/tools/baray_setup.rs` — scaffolds Baray payment into an existing app
- [ ] Actions: `setup`, `test`, `status`, `transactions`
- [ ] `setup` action:
  1. Reads Baray credentials from config or prompts user
  2. Generates payment endpoint code (AES-256-CBC encrypted intent)
  3. Adds `/pay` route to the app
  4. Adds Baray webhook handler for order confirmation
  5. Injects Baray redirect flow into frontend
- [ ] Support both USD and KHR currencies
- [ ] Banks: ABA, ACLEDA, Sathapana, Wing
- [ ] `test` action: creates a $0.03 test payment and verifies webhook
- [ ] `transactions` action: lists recent payments via Baray dashboard API
- [ ] Register in factory, add to Forge's allowed_tools

### Baray Config
```toml
[baray]
enabled = true
api_key = ""       # from dash.baray.io
secret_key = ""    # AES-256-CBC sk
iv = ""            # AES-256-CBC iv
webhook_url = ""   # auto-set to app-name.koompi.cloud/api/baray/webhook
```

### Reference
- API docs: https://baray.io/llm.txt
- Dashboard: https://dash.baray.io
- Encryption: AES-256-CBC with sk + iv from dashboard
- Flow: POST /pay → get intent_id → redirect to pay.baray.io/{intent_id}

---

## Phase 5: KOOMPI ID (KID) Auth Integration

- [ ] Create `src/tools/kid_setup.rs` — scaffolds KID OAuth into an existing app
- [ ] Actions: `setup`, `test`, `status`, `users`
- [ ] `setup` action:
  1. Reads KID client_id/secret from config or prompts user
  2. Generates OAuth 2.0 flow code (authorize → token exchange → userinfo)
  3. Adds `/auth/login`, `/auth/callback`, `/auth/logout` routes
  4. Adds session management (JWT or cookie-based)
  5. Injects login/logout UI components
  6. Installs `@koompi/oauth` SDK if Node.js project
- [ ] Scopes support: `profile.basic`, `profile.contact`, `wallet.read`, `wallet.transfer`, `wallet.balance`
- [ ] Token lifecycle: access (1hr), refresh (1yr), auto-refresh
- [ ] `test` action: opens OAuth flow, verifies token exchange works
- [ ] `users` action: lists authenticated users from app session store
- [ ] Register in factory, add to Forge's allowed_tools

### KID Config
```toml
[koompi_id]
enabled = true
client_id = ""
client_secret = ""
redirect_uri = ""  # auto-set to app-name.koompi.cloud/auth/callback
scopes = ["profile.basic", "profile.contact"]
```

### Reference
- OAuth base: https://oauth.koompi.org
- API docs: https://dash.koompi.org/llms.txt
- SDK: `npm install @koompi/oauth`
- Endpoints: /v2/oauth, /v2/oauth/token, /v2/oauth/refresh, /v2/oauth/userinfo
- ERC20/ERC1155: /v2/erc20/transfer, /v2/erc1155/mint (API key auth)

---

## Phase 6: Full Chat-to-Deploy Pipeline

- [ ] Wire all tools into Sentinel's routing logic:
  - "deploy" / "publish" / "go live" → `koompi_cloud deploy`
  - "add payments" / "accept payments" / "baray" → `baray_setup setup`
  - "add login" / "add auth" / "KID" → `kid_setup setup`
  - "custom domain" / "use my domain" → `koompi_cloud domain`
- [ ] Sentinel auto-chains: build → deploy → announce URL
- [ ] Add deploy status to heartbeat monitoring
- [ ] Add deploy history to user workspace metadata
- [ ] Support re-deploy on code changes: "update the site"
- [ ] Support environment variables per deployment
- [ ] Support preview deploys: `preview-{hash}.koompi.cloud`

---

## Phase 7: Sentinel Persona Updates

- [ ] Update Forge system prompt to include KOOMPI Cloud deploy instructions
- [ ] Update Archon to consider KOOMPI Cloud architecture constraints
- [ ] Add KOOMPI Cloud to TOOLS.md access matrix
- [ ] Update sentinel/AGENTS.md routing rules:
  - "deploy", "hosting", "domain" → Forge (with koompi_cloud tool)
  - "payment", "baray", "checkout" → Forge (with baray_setup tool)
  - "login", "auth", "KID", "OAuth" → Forge (with kid_setup tool)

---

## Acceptance Criteria

For each phase, all of the following must pass:

- [ ] Tool implements `Tool` trait with proper parameter schema
- [ ] Registered in `all_tools_with_runtime()` factory
- [ ] Added to relevant persona's `allowed_tools`
- [ ] Error paths return structured `ToolResult` (no panics)
- [ ] Secrets never logged or stored in memory
- [ ] Works end-to-end: user message → Sentinel → Forge → tool → deployed app
- [ ] Tests cover: happy path, auth failure, deploy failure, domain conflict

---

## Non-Goals (Out of Scope)

- Database provisioning (use external PostgreSQL/Redis for now)
- CI/CD pipeline (deploy is manual or chat-triggered, not git-push-triggered)
- Multi-region failover (single region for v1)
- Container orchestration UI (Sentinel is the UI)
- Billing/metering (KOOMPI Cloud handles this separately)

---

## Dependencies

| Dependency | Status | Owner |
|---|---|---|
| KOOMPI Cloud API | Needs API spec from cloud team | Cloud team |
| Baray API | Documented at baray.io/llm.txt | Ready |
| KID OAuth | Documented at dash.koompi.org/llms.txt | Ready |
| DNS automation | Needs KOOMPI Cloud DNS API | Cloud team |
| SSL provisioning | Let's Encrypt on KOOMPI Cloud side | Cloud team |
