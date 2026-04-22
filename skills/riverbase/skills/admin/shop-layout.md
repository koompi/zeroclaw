# Shop Layout: Header, Footer, About Page & Blog Layout

Customize the storefront's header, footer, about page, and blog layout via the GraphQL API. All layout configs are stored as a single `ShopLayout` document per shop and modified through JSON mutations.

---

## Get Shop Layout

Returns the full layout configuration (header, footer, about page, blog layout).

```graphql
query GetShopLayout($shopId: String!) {
  getShopLayout(shopId: $shopId) {
    id shopId createdAt updatedAt
    header {
      layout
      showAnnouncement
      announcementText
      showSearch
      showCart
      logoPosition
      sticky
      navigation { label href visibleTo highlight }
      blocks { blockType order props visible }
    }
    footer {
      layout
      description
      columns {
        title
        links { label href visibleTo highlight }
      }
      showSocial
      socialLinks { platform url icon }
      showNewsletter
      copyrightText
      blocks { blockType order props visible }
    }
    aboutPage {
      template
      heroTitle
      heroSubtitle
      heroImage
      storyText
      missionText
      visionText
      teamMembers { name role photo bio }
      showTimeline
      timelineItems { date title description }
      showStats
      stats { value label icon }
      blocks { blockType order props visible }
    }
    blogLayout {
      template
      showFeatured
      postsPerPage
      showSidebar
      showCategories
      showSearch
      showAuthor
      showDate
      showExcerpt
      heroTitle
      heroSubtitle
      blocks { blockType order props visible }
    }
  }
}
```

---

## Update Header

```graphql
mutation UpdateShopHeader($shopId: String!, $header: JSON!) {
  updateShopHeader(shopId: $shopId, header: $header) {
    id shopId
    header { layout showAnnouncement announcementText showSearch showCart logoPosition sticky navigation { label href visibleTo highlight } blocks { blockType order props visible } }
  }
}
```

### Header JSON Structure

The `$header` variable is a JSON object:

```json
{
  "layout": "classic",
  "logo_position": "left",
  "sticky": true,
  "show_search": true,
  "show_cart": true,
  "show_announcement": false,
  "announcement_text": "",
  "navigation": [
    { "label": "Home", "href": "/", "visible_to": "all", "highlight": false },
    { "label": "Products", "href": "/products", "visible_to": "all", "highlight": false },
    { "label": "About", "href": "/about", "visible_to": "all", "highlight": false },
    { "label": "Contact", "href": "/contact", "visible_to": "all", "highlight": false }
  ],
  "blocks": []
}
```

### Header Enums & Fields

| Field | Type | Description |
|---|---|---|
| `layout` | String | `"classic"`, `"centered"`, `"minimal"` |
| `logo_position` | String | `"left"` or `"center"` |
| `sticky` | Boolean | Header stays visible on scroll |
| `show_search` | Boolean | Show search bar in header |
| `show_cart` | Boolean | Show cart icon in header |
| `show_announcement` | Boolean | Show announcement bar above header |
| `announcement_text` | String | Text displayed in announcement bar |
| `navigation` | Array | Navigation links (see NavLink below) |
| `blocks` | Array | Custom layout blocks |

**NavLink object:**
| Field | Type | Description |
|---|---|---|
| `label` | String | Display text |
| `href` | String | Link target (`/products`, `/about`, `/contact`, etc.) |
| `visible_to` | String | `"all"` (currently only value) |
| `highlight` | Boolean | Highlight this nav item (bold/accent) |

### Header Presets

**Classic** — Logo left, navigation right, clean and professional:
```json
{
  "layout": "classic",
  "logo_position": "left",
  "sticky": true,
  "show_search": true,
  "show_cart": true,
  "show_announcement": false,
  "announcement_text": "",
  "navigation": [
    { "label": "Home", "href": "/", "visible_to": "all", "highlight": false },
    { "label": "Products", "href": "/products", "visible_to": "all", "highlight": false },
    { "label": "About", "href": "/about", "visible_to": "all", "highlight": false },
    { "label": "Contact", "href": "/contact", "visible_to": "all", "highlight": false }
  ],
  "blocks": []
}
```

**Centered** — Centered logo with announcement bar:
```json
{
  "layout": "centered",
  "logo_position": "center",
  "sticky": true,
  "show_search": true,
  "show_cart": true,
  "show_announcement": true,
  "announcement_text": "Free shipping on orders over $50",
  "navigation": [
    { "label": "Shop", "href": "/products", "visible_to": "all", "highlight": false },
    { "label": "Collections", "href": "/collections", "visible_to": "all", "highlight": false },
    { "label": "About Us", "href": "/about", "visible_to": "all", "highlight": false }
  ],
  "blocks": []
}
```

**Minimal** — Clean header with minimal navigation:
```json
{
  "layout": "minimal",
  "logo_position": "left",
  "sticky": false,
  "show_search": false,
  "show_cart": true,
  "show_announcement": false,
  "announcement_text": "",
  "navigation": [
    { "label": "Shop", "href": "/products", "visible_to": "all", "highlight": false },
    { "label": "About", "href": "/about", "visible_to": "all", "highlight": false }
  ],
  "blocks": []
}
```

---

## Update Footer

```graphql
mutation UpdateShopFooter($shopId: String!, $footer: JSON!) {
  updateShopFooter(shopId: $shopId, footer: $footer) {
    id shopId
    footer { layout description columns { title links { label href visibleTo highlight } } showSocial socialLinks { platform url icon } showNewsletter copyrightText blocks { blockType order props visible } }
  }
}
```

### Footer JSON Structure

```json
{
  "layout": "columns",
  "description": "",
  "columns": [
    {
      "title": "Shop",
      "links": [
        { "label": "All Products", "href": "/products", "visible_to": "all", "highlight": false },
        { "label": "New Arrivals", "href": "/products?sort=newest", "visible_to": "all", "highlight": false }
      ]
    },
    {
      "title": "Help",
      "links": [
        { "label": "Contact Us", "href": "/contact", "visible_to": "all", "highlight": false },
        { "label": "Shipping Info", "href": "/shipping", "visible_to": "all", "highlight": false }
      ]
    }
  ],
  "show_social": true,
  "social_links": [
    { "platform": "facebook", "url": "https://facebook.com/myshop", "icon": "" },
    { "platform": "telegram", "url": "https://t.me/myshop", "icon": "" }
  ],
  "show_newsletter": true,
  "copyright_text": "© 2025 My Shop. All rights reserved.",
  "blocks": []
}
```

### Footer Enums & Fields

| Field | Type | Description |
|---|---|---|
| `layout` | String | `"columns"`, `"simple"`, `"centered"` |
| `description` | String | Short shop description shown in footer |
| `columns` | Array | Link columns (see FooterColumn below) |
| `show_social` | Boolean | Show social media links |
| `social_links` | Array | Social platform links (see SocialLink below) |
| `show_newsletter` | Boolean | Show newsletter signup form |
| `copyright_text` | String | Copyright notice at bottom |
| `blocks` | Array | Custom layout blocks |

**FooterColumn object:**
| Field | Type | Description |
|---|---|---|
| `title` | String | Column heading |
| `links` | Array | Array of NavLink objects (same format as header) |

**SocialLink object:**
| Field | Type | Description |
|---|---|---|
| `platform` | String | `"facebook"`, `"telegram"`, `"instagram"`, `"tiktok"`, `"twitter"`, `"youtube"`, `"linkedin"` |
| `url` | String | Full URL to social profile |
| `icon` | String | Icon identifier (usually empty `""`) |

### Footer Presets

**Columns** — Multi-column links with social and newsletter:
```json
{
  "layout": "columns",
  "show_social": true,
  "social_links": [
    { "platform": "facebook", "url": "", "icon": "" },
    { "platform": "telegram", "url": "", "icon": "" }
  ],
  "show_newsletter": true,
  "copyright_text": "© 2025 Your Brand. All rights reserved.",
  "columns": [
    {
      "title": "Shop",
      "links": [
        { "label": "All Products", "href": "/products", "visible_to": "all", "highlight": false },
        { "label": "New Arrivals", "href": "/products?sort=newest", "visible_to": "all", "highlight": false },
        { "label": "Best Sellers", "href": "/products?sort=popular", "visible_to": "all", "highlight": false }
      ]
    },
    {
      "title": "Help",
      "links": [
        { "label": "Contact Us", "href": "/contact", "visible_to": "all", "highlight": false },
        { "label": "Shipping Info", "href": "/shipping", "visible_to": "all", "highlight": false },
        { "label": "Returns", "href": "/returns", "visible_to": "all", "highlight": false }
      ]
    },
    {
      "title": "About",
      "links": [
        { "label": "Our Story", "href": "/about", "visible_to": "all", "highlight": false },
        { "label": "Blog", "href": "/blog", "visible_to": "all", "highlight": false }
      ]
    }
  ],
  "blocks": []
}
```

**Simple** — Compact footer with social links:
```json
{
  "layout": "simple",
  "show_social": true,
  "social_links": [
    { "platform": "facebook", "url": "", "icon": "" },
    { "platform": "telegram", "url": "", "icon": "" }
  ],
  "show_newsletter": false,
  "copyright_text": "© 2025 Your Brand",
  "columns": [],
  "blocks": []
}
```

**Centered** — Centered layout with newsletter signup:
```json
{
  "layout": "centered",
  "show_social": true,
  "social_links": [
    { "platform": "facebook", "url": "", "icon": "" },
    { "platform": "instagram", "url": "", "icon": "" },
    { "platform": "tiktok", "url": "", "icon": "" }
  ],
  "show_newsletter": true,
  "copyright_text": "© 2025 Your Brand. Made with love.",
  "columns": [],
  "blocks": []
}
```

---

## Update About Page

```graphql
mutation UpdateShopAboutPage($shopId: String!, $aboutPage: JSON!) {
  updateShopAboutPage(shopId: $shopId, aboutPage: $aboutPage) {
    id shopId
    aboutPage {
      template heroTitle heroSubtitle heroImage storyText missionText visionText
      teamMembers { name role photo bio }
      showTimeline timelineItems { date title description }
      showStats stats { value label icon }
      blocks { blockType order props visible }
    }
  }
}
```

### About Page JSON Structure

```json
{
  "template": "company_story",
  "hero_title": "Our Story",
  "hero_subtitle": "Learn about who we are and what drives us",
  "hero_image": "",
  "story_text": "Founded in 2020, we started with a passion for quality...",
  "mission_text": "To provide the best products and experience for our customers.",
  "vision_text": "To become the leading brand in our industry.",
  "team_members": [
    { "name": "John Doe", "role": "CEO", "photo": "", "bio": "Founder and visionary leader." },
    { "name": "Jane Smith", "role": "Product Manager", "photo": "", "bio": "Passionate about great products." }
  ],
  "show_timeline": true,
  "timeline_items": [
    { "date": "2020", "title": "Founded", "description": "Our journey began" },
    { "date": "2022", "title": "Growing", "description": "Expanded our team and products" },
    { "date": "2024", "title": "Today", "description": "Serving customers worldwide" }
  ],
  "show_stats": true,
  "stats": [
    { "value": "1000+", "label": "Happy Customers", "icon": "" },
    { "value": "500+", "label": "Products", "icon": "" },
    { "value": "50+", "label": "Team Members", "icon": "" }
  ],
  "blocks": []
}
```

### About Page Enums & Fields

| Field | Type | Description |
|---|---|---|
| `template` | String | `"company_story"`, `"team_showcase"`, `"mission_values"`, `"simple_bio"`, `"photo_timeline"` |
| `hero_title` | String | Main heading |
| `hero_subtitle` | String | Subheading below hero |
| `hero_image` | String | Hero background/featured image URL (upload first) |
| `story_text` | String | Brand story paragraph |
| `mission_text` | String | Mission statement |
| `vision_text` | String | Vision statement |
| `team_members` | Array | Team member cards (see below) |
| `show_timeline` | Boolean | Display company timeline |
| `timeline_items` | Array | Timeline events (see below) |
| `show_stats` | Boolean | Display statistics |
| `stats` | Array | Stat cards (see below) |
| `blocks` | Array | Custom layout blocks |

**TeamMember object:**
| Field | Type | Description |
|---|---|---|
| `name` | String | Full name |
| `role` | String | Job title/position |
| `photo` | String | Portrait image URL (upload first via `/uploads/s3`) |
| `bio` | String | Short biography |

**TimelineItem object:**
| Field | Type | Description |
|---|---|---|
| `date` | String | Date label (e.g. `"2024"`, `"Jan 2024"`) |
| `title` | String | Event title |
| `description` | String | Brief description |

**StatItem object:**
| Field | Type | Description |
|---|---|---|
| `value` | String | Display value (e.g. `"1000+"`, `"24/7"`) |
| `label` | String | Label below the value |
| `icon` | String | Icon identifier (usually empty `""`) |

### About Page Presets

**Company Story** — Timeline-based story with stats (best for most businesses)
**Team Showcase** — Highlight team members prominently
**Mission & Values** — Focus on mission, vision, and values with stats
**Simple Bio** — Clean, minimal about page
**Photo Timeline** — Visual journey through milestones

> See full preset JSON examples in the repository source.

---

## Update Blog Layout

```graphql
mutation UpdateShopBlogLayout($shopId: String!, $blogLayout: JSON!) {
  updateShopBlogLayout(shopId: $shopId, blogLayout: $blogLayout) {
    id shopId
    blogLayout {
      template showFeatured postsPerPage showSidebar showCategories showSearch showAuthor showDate showExcerpt heroTitle heroSubtitle
      blocks { blockType order props visible }
    }
  }
}
```

### Blog Layout JSON Structure

```json
{
  "template": "classic_list",
  "show_featured": true,
  "posts_per_page": 10,
  "show_sidebar": true,
  "show_categories": true,
  "show_search": true,
  "show_author": true,
  "show_date": true,
  "show_excerpt": true,
  "hero_title": "Blog",
  "hero_subtitle": "Latest news and updates",
  "blocks": []
}
```

### Blog Layout Enums & Fields

| Field | Type | Description |
|---|---|---|
| `template` | String | `"classic_list"`, `"magazine_grid"`, `"minimal_list"`, `"card_grid"` |
| `show_featured` | Boolean | Highlight the latest post as featured |
| `posts_per_page` | Int | Number of posts per page |
| `show_sidebar` | Boolean | Show sidebar with categories/search |
| `show_categories` | Boolean | Show category filter |
| `show_search` | Boolean | Show blog search bar |
| `show_author` | Boolean | Show author name on posts |
| `show_date` | Boolean | Show publish date on posts |
| `show_excerpt` | Boolean | Show post excerpt/preview |
| `hero_title` | String | Blog page heading |
| `hero_subtitle` | String | Blog page subheading |
| `blocks` | Array | Custom layout blocks |

### Blog Layout Presets

**Classic List** — Traditional blog with sidebar (10 posts/page)
**Magazine Grid** — Grid layout with featured post (12 posts/page)
**Minimal List** — Clean, text-focused list (20 posts/page)
**Card Grid** — Card-based grid with images (9 posts/page)

---

## Update Header + Footer Together

Update both header and footer in a single mutation:

```graphql
mutation UpdateShopLayout($shopId: String!, $header: JSON!, $footer: JSON!) {
  updateShopLayout(shopId: $shopId, header: $header, footer: $footer) {
    id shopId
    header { layout showAnnouncement announcementText showSearch showCart logoPosition sticky navigation { label href visibleTo highlight } blocks { blockType order props visible } }
    footer { layout description columns { title links { label href visibleTo highlight } } showSocial socialLinks { platform url icon } showNewsletter copyrightText blocks { blockType order props visible } }
  }
}
```

---

## Reset Shop Layout

Resets all layout configuration (header, footer, about page, blog layout) to platform defaults.

> ⚠️ **CONFIRMATION REQUIRED** — Warn the user that this will discard all header, footer, about page, and blog layout customizations. Wait for explicit "Yes".

```graphql
mutation ResetShopLayout($shopId: String!) {
  resetShopLayout(shopId: $shopId) {
    id shopId
    header { layout }
    footer { layout }
    aboutPage { template }
    blogLayout { template }
  }
}
```

---

## LayoutBlock (Reusable across all sections)

All layout sections (header, footer, about page, blog) support custom `blocks` arrays for extensible content:

| Field | Type | Description |
|---|---|---|
| `blockType` | String | Type of content block |
| `order` | Int | Display order (0 = first) |
| `props` | JSON | Block-specific properties |
| `visible` | Boolean | Whether block is shown |

Currently `blocks` is typically empty (`[]`) — reserved for future extensibility.

---

## Agent Notes

- **Read first, modify second**: Always fetch the current layout with `getShopLayout` before making changes. Mutations replace the entire section (header/footer/about/blog), not individual fields.
- **snake_case in JSON**: The JSON variables use `snake_case` (e.g., `logo_position`, `show_search`, `hero_title`), while the GraphQL response uses `camelCase` (e.g., `logoPosition`, `showSearch`, `heroTitle`). This is a quirk of the API — send snake_case, receive camelCase.
- **Presets are starting points**: When a user says "set up a professional header", pick the `classic` preset and customize from there. For "minimal footer", use the `simple` preset.
- **Images**: Upload hero images and team member photos via `POST /uploads/s3` first, then use the returned CDN URL.
- **Combined update**: Use `updateShopLayout` when changing both header and footer simultaneously to avoid race conditions.
- **About page templates**: The `template` field controls the page layout. Pick one that matches the user's intent — `company_story` for most businesses, `team_showcase` for agencies, `simple_bio` for individuals.
