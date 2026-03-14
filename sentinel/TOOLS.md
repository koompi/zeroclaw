# Sentinel Tools Reference

Tools available to Sentinel and its agents. Each agent has access to a subset based on their role.

## Tool Access Matrix

| Tool | Sentinel | Archon | Prism | Forge | Nexus | Echo | Closer | Oracle | Veasna |
|---|---|---|---|---|---|---|---|---|---|
| `claude_code` | — | plan | all | all | — | — | — | — | — |
| `shell` | — | — | — | ✅ | — | — | — | — | — |
| `file_read` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `file_write` | — | — | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `file_edit` | — | — | ✅ | ✅ | — | — | — | — | — |
| `content_search` | — | ✅ | ✅ | ✅ | — | — | — | ✅ | — |
| `glob_search` | — | ✅ | ✅ | ✅ | — | — | — | ✅ | — |
| `git_operations` | — | ✅ | — | ✅ | — | — | — | — | — |
| `web_search` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `web_fetch` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `browser` | — | — | ✅ | ✅ | — | — | — | — | — |
| `screenshot` | — | — | ✅ | ✅ | — | — | — | — | — |
| `memory_store` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `memory_recall` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `http_request` | — | — | — | ✅ | ✅ | — | ✅ | ✅ | — |
| `pdf_read` | — | — | — | ✅ | — | — | — | ✅ | — |
| `proposal_gen` | — | — | — | — | ✅ | ✅ | ✅ | — | — |
| `delegate` | ✅ | — | — | — | — | — | — | — | — |

## Sentinel-Only Tools

Sentinel has exclusive access to orchestration tools:

- **`delegate`**: Spawn a subagent with a task
- **`memory_store`** / **`memory_recall`**: Persist and retrieve cross-session knowledge
- **`web_search`** / **`web_fetch`**: Quick lookups when classifying tasks
- **`file_read`**: Read workspace state for context building

Sentinel does NOT have: `shell`, `file_write`, `file_edit`, `claude_code`, `browser`. It delegates execution, never executes directly.

## Technology Stack (Enforced by Forge & Prism)

When building, agents must use these defaults:

| Category | Technology |
|---|---|
| **Bots** | Python (asyncio, httpx, pydantic) |
| **Frontend** | React + Vite.js or Next.js, TypeScript, Tailwind CSS, shadcn/ui |
| **Backend (speed)** | Rust (axum, tokio, serde) |
| **Backend (rapid)** | Next.js API routes or Python FastAPI |
| **Payment** | Stripe + Baray.io (AES-256-CBC encrypted) |
| **Authentication** | KOOMPI ID OAuth 2.0 (@koompi/oauth SDK) |
| **Database** | PostgreSQL (primary), SQLite (embedded), Redis (cache) |

## Cost-Aware Model Routing

Each persona picks the cheapest model that can do the job:

| Persona | Claude Code Model | Rationale |
|---|---|---|
| **Archon** | `claude-haiku-4-5` | Plan/review only — no code generation needed |
| **Prism** | Default (Sonnet) | Full implementation of frontend components |
| **Forge** | `claude-sonnet-4-6` | Heavy implementation; escalate to Opus for complex tasks |
| **Oracle** | `claude-haiku-4-5` | Read/search only — no code generation |
| **Others** | No Claude Code access | Business/research personas don't code |

**Cost savings**: By routing plan/review to Haiku (~10x cheaper than Opus), a typical multi-agent workflow costs 60-80% less than running everything on Opus.

**Runtime override**: The `model` parameter on `claude_code` tool allows Sentinel to override at runtime when a task is harder than expected.

## Tool Safety Rules

1. **Shell**: Only Forge has shell access. All shell commands are logged.
2. **File writes**: Agents cannot write outside the user's workspace directory.
3. **HTTP requests**: Rate-limited. No requests to internal/private IPs without explicit config.
4. **Browser**: Sandboxed. No credential autofill. No form submission without user approval.
5. **Memory**: No secrets stored in memory. PII detection is active.
6. **Claude Code**: Runs in `--print` mode. `--dangerously-skip-permissions` only in sandboxed environments.
