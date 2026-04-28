# Riverbase Skill

Machine-readable skill files that let **any AI agent** manage a Riverbase shop entirely through the GraphQL API — no frontend or Mini App needed.

## What is this?

Riverbase is a Telegram-native commerce platform. Shop owners normally manage products, orders, inventory, and settings through a web dashboard or the Telegram Mini App.

This repo replaces that UI layer with a structured set of **executable GraphQL references** so an AI agent (ChatGPT, Claude, Gemini, your own bot, etc.) can do everything the dashboard can — directly from a conversation.

Give the agent a JWT token and a shop ID, and it can:

- Create and update products, categories, and brands
- Process orders through PENDING → CONFIRMED → COMPLETED
- Handle payments (COD, ABA Transfer, direct)
- Manage inventory across multiple warehouse locations
- Configure shipping fees, delivery zones, and options
- Create coupons and discount rules
- Manage staff roles and permissions
- Set business hours, announcements, and shop modules
- **Customize shop layout: header, footer, about page, and blog layout**
- **CMS: create and publish blog posts with customizable blog page layouts**
- Run a loyalty/membership program
- Send contract quotations
- Connect custom domains
- Install and configure plugins

## For Humans

Browse the skill files to understand the full Riverbase API surface:

```
SKILL.md                          ← Start here: auth, enums, routing table
skills/
  catalog/
    products.md                   ← Product CRUD, search, variants, archive
    categories.md                 ← Categories & subcategories
    brands.md                     ← Brand CRUD
  orders/
    lifecycle.md                  ← Order state machine, POS creation
    queries.md                    ← List, filter, stats, dashboard
    payments.md                   ← Payment processing, COD, ABA
  inventory/
    locations.md                  ← Warehouse/store locations
    transactions.md               ← Stock in/out, transfers, adjustments
    checks.md                     ← Stock balance, availability checks
  admin/
    shop.md                       ← Shop settings, GPS, bot, customers
    modules.md                    ← Feature toggles, business hours
    team.md                       ← Roles & members
    shipping.md                   ← Shipping rules & delivery options
    discounts.md                  ← Coupons & discount rules (with recipes)
    sections.md                   ← Storefront sections & canvas designs
    shop-layout.md                ← Header, footer, about page & blog layout customization
    appearance.md                 ← Theme colors, fonts, border radius, branding
  advanced/
    membership.md                 ← VIP tiers & membership cards
    quotations.md                 ← Contract quotations & revisions
    content.md                    ← Blog posts, events & CMS/blog layout
    dns.md                        ← Custom domain management
    plugins.md                    ← Plugin marketplace
  superadmin/
    dashboard.md                  ← Platform-wide metrics & revenue
    shops.md                      ← All shops, activate/deactivate
    users.md                      ← All users, freeze/activate
    business-categories.md        ← Platform business category CRUD
```

Each file contains **copy-paste-ready GraphQL** queries and mutations with exact field names, enum values, and variable types — extracted from the production frontend.

## For AI Agents

### Quick Start

1. **Load [`SKILL.md`](SKILL.md)** — it contains auth setup, the enum reference, and a keyword-based router table.
2. **Run the Bootstrap Query** (in SKILL.md) to resolve the user's `shopId`.
3. **Load only the sub-file you need** based on what the user is asking. Don't load everything at once.
4. **Execute the GraphQL** against `https://api.riverbase.org/graphql` with the user's token (raw token in `Authorization` header, no `Bearer` prefix).

### System Prompt Example

```
You are an AI shop assistant for Riverbase.
You have access to the Riverbase Skill files.

The user's auth token: <JWT>
The user's shop ID: <SHOP_ID>

When the user asks you to manage their shop, read SKILL.md to find the 
right sub-skill file, then execute the GraphQL operations described in it.

Always resolve names to IDs before mutating. Confirm before destructive actions.
All prices are String type. Pagination defaults: limit 20, page 1.
```

### Key Rules

- **IDs first** — always search/list to resolve names to IDs before mutations
- **Price = String** — all monetary values are `String`, not `Float`
- **One file at a time** — the router table in SKILL.md tells you which file to load
- **`shopId` everywhere** — every operation needs it; resolve once at session start
- **⚠️ Delete/Remove = ALWAYS confirm** — there are 16 destructive mutations across the skill files (listed in SKILL.md). The agent must **never** execute any of them without showing a summary and getting an explicit "Yes" or a Telegram confirmation button tap first

## API Reference

| | |
|---|---|
| **Production GraphQL** | `https://api.riverbase.org/graphql` |
| **Staging GraphQL** | `https://staging-lite-api.riverbase.org/graphql` |
| **Upload Endpoint** | `https://api.riverbase.org/uploads/s3` |
| **Auth** | `Authorization: <token>` (no `Bearer` prefix) |
| **Method** | `POST` with JSON body `{"query": "...", "variables": {...}}` |

## Updating This Skill

This skill is maintained at [github.com/koompi/koompi-biz-skill](https://github.com/koompi/koompi-biz-skill).

When the platform adds new GraphQL mutations/queries or changes existing ones, the skill files need to be updated to match. To get the latest version:

```bash
# Option 1: Pull the latest changes (if you cloned the repo)
git pull origin main

# Option 2: Clone fresh (replace the existing copy)
rm -rf koompi-biz-skill
git clone https://github.com/koompi/koompi-biz-skill.git
```

### When to Update
- New GraphQL mutations or queries are added to the platform
- Existing mutations change their input/output fields
- New enums or types are introduced
- Shop layout, theme, or CMS features are added/changed

### Environment Variables for Agents

Agents need `RIVERBASE_API_URL`, `RIVERBASE_API_KEY`, and `RIVERBASE_SHOP_ID` to connect. These are stored in a `.env` file on disk, **not** in the shell environment.

The agent MUST read the file directly — do not just check `env` and give up. Execute:

```bash
# Try the most likely location first
grep -E 'RIVERBASE_API_URL|RIVERBASE_API_KEY|RIVERBASE_SHOP_ID' /zeroclaw-data/workspace/.env 2>/dev/null

# If not found, try alternatives
grep -E 'RIVERBASE_API_URL|RIVERBASE_API_KEY|RIVERBASE_SHOP_ID' /openclaw-data/workspace/.env /data/workspace/.env /workspace/.env .env 2>/dev/null

# Last resort: search the whole disk
find / -maxdepth 4 -name '.env' -exec grep -l 'RIVERBASE' {} \; 2>/dev/null | head -5
```

| Variable | Purpose |
|---|---|
| `RIVERBASE_API_URL` | GraphQL endpoint (`https://staging-lite-api.riverbase.org/graphql` for staging, `https://api.riverbase.org/graphql` for production) |
| `RIVERBASE_API_KEY` | Auth token — use in `Authorization` header (no `Bearer` prefix) |
| `RIVERBASE_SHOP_ID` | Default shop to operate on |

## License

Copyright © Riverbase. All rights reserved.
