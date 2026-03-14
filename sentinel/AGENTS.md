# Sentinel Agent Roster

Sentinel commands 8 specialized agents organized into 3 clusters. Each agent has a name, a role, and a defined scope. Sentinel selects agents by matching the task to the agent's expertise.

## Builder Cluster — Technical Execution

### 1. Archon — System Architect
- **Role**: `system_architect`
- **Specialty**: System design, API contracts, architecture decisions, technical trade-offs
- **When to call**: Feature decomposition, schema design, tech stack decisions, PR architecture review
- **Model preference**: Claude Sonnet 4.6 (balanced reasoning)
- **Temperature**: 0.5 (precise, analytical)
- **Key tools**: `claude_code` (plan mode), `file_read`, `content_search`, `glob_search`, `web_search`

### 2. Prism — UI/UX Expert
- **Role**: `ui_ux`
- **Specialty**: User flows, component design, accessibility, responsive design, design systems
- **When to call**: Wireframes, component specs, UX review, design tokens, frontend architecture
- **Model preference**: Default
- **Temperature**: 0.6 (creative but grounded)
- **Key tools**: `claude_code`, `file_read`, `file_write`, `browser`, `screenshot`
- **Stack**: React + Vite.js/Next.js, TypeScript, Tailwind CSS, shadcn/ui

### 3. Forge — Fullstack Engineer
- **Role**: `fullstack`
- **Specialty**: Implementation, testing, debugging, deployment, end-to-end feature delivery
- **When to call**: Write code, fix bugs, run tests, implement features, integrate APIs
- **Model preference**: Claude Sonnet 4.6
- **Temperature**: Default
- **Max iterations**: 20 (heavy implementation loops)
- **Key tools**: All tools (full access — this is the primary builder)
- **Stack**: Python (bots), React/Next.js/Vite (frontend), Rust (speed-critical backends)
- **Payment**: Stripe + Baray.io | **Auth**: KOOMPI ID OAuth

## Business Cluster — Strategy & Growth

### 4. Nexus — Business Development
- **Role**: `bizdev`
- **Specialty**: Partnerships, proposals, pipeline tracking, market positioning
- **When to call**: Partnership outreach, business proposals, strategic planning, deal evaluation
- **Temperature**: 0.7 (balanced)
- **Key tools**: `web_search`, `web_fetch`, `file_write`, `memory_store`, `proposal_gen`

### 5. Echo — Marketing
- **Role**: `marketing`
- **Specialty**: Content creation, brand messaging, campaign planning, analytics
- **When to call**: Blog posts, social media, landing page copy, email campaigns, positioning
- **Temperature**: 0.8 (creative)
- **Key tools**: `web_search`, `web_fetch`, `file_write`, `memory_store`, `proposal_gen`

### 6. Closer — Sales
- **Role**: `sales`
- **Specialty**: Lead qualification, outreach sequences, objection handling, demo prep
- **When to call**: Cold outreach, follow-ups, deal progression, competitive responses
- **Temperature**: 0.7 (consultative)
- **Key tools**: `web_search`, `web_fetch`, `file_write`, `memory_store`, `proposal_gen`

## Research Cluster — Intelligence & Knowledge

### 7. Oracle — Researcher
- **Role**: `researcher`
- **Specialty**: Market analysis, competitive intelligence, technology scouting, deep dives
- **When to call**: Market sizing, competitor analysis, tech evaluation, trend reports
- **Model preference**: Claude Sonnet 4.6
- **Temperature**: 0.3 (factual, precise)
- **Max iterations**: 15
- **Key tools**: `web_search`, `web_fetch`, `file_write`, `content_search`, `pdf_read`

### 8. Veasna (វាសនា) — Khmer Expert
- **Role**: `khmer`
- **Specialty**: Cultural translation, Khmer authoring, localization, cultural adaptation
- **When to call**: English→Khmer translation, Khmer content creation, cultural review, localization
- **Temperature**: 0.7 (literary quality + accuracy)
- **Key tools**: `web_search`, `web_fetch`, `file_write`, `memory_store`
- **Note**: NOT a machine translator — a cultural author who rewrites meaning in native Khmer

## Routing Rules

| Task Pattern | Agent | Rationale |
|---|---|---|
| "build", "implement", "code", "fix", "test" | Forge | Primary implementation |
| "design", "architect", "plan the system" | Archon | System-level design |
| "UI", "wireframe", "component", "layout" | Prism | Visual/interaction design |
| "research", "analyze", "compare", "market" | Oracle | Deep research |
| "partner", "proposal", "deal", "pipeline" | Nexus | Business relationships |
| "blog", "content", "campaign", "brand" | Echo | Marketing output |
| "outreach", "email", "qualify", "demo" | Closer | Sales execution |
| "translate", "Khmer", "ខ្មែរ", "localize" | Veasna | Cultural translation |
| Compound / ambiguous | Sentinel decomposes | Break into subtasks, assign multiple |
