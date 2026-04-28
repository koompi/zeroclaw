# SEO & Shop Branding

Control how the shop appears in search engines and browsers.

## Get Current SEO Settings

```graphql
query ShopMetadata($shopId: String!) {
  shopId(shopId: $shopId) {
    id name logo
    seo { favicon metaTitle metaDescription ogImage keywords }
  }
}
```

## Update SEO Settings

```graphql
mutation UpdateSeo(
  $shopId: String!
  $favicon: String
  $metaTitle: String
  $metaDescription: String
  $ogImage: String
  $keywords: String
) {
  shopUpdateSeo(
    shopId: $shopId
    favicon: $favicon
    metaTitle: $metaTitle
    metaDescription: $metaDescription
    ogImage: $ogImage
    keywords: $keywords
  ) { id seo { favicon metaTitle metaDescription ogImage keywords } }
}
```

### Field Reference

| Field | Purpose | Example |
|---|---|---|
| `metaTitle` | Browser tab title + Google search title | `"Sunrise Coffee — Premium Roasted Beans in Phnom Penh"` |
| `metaDescription` | Google search snippet (150–160 chars) | `"Fresh roasted coffee beans delivered daily. Order online for pickup or delivery. Open Mon-Sat 8am-6pm."` |
| `keywords` | Comma-separated SEO keywords | `"coffee, roasted beans, phnom penh, cafe, specialty coffee"` |
| `favicon` | Browser tab icon (CDN URL, 32×32 recommended) | Upload via `POST /uploads/s3` |
| `ogImage` | Social media share image (1200×630 recommended) | Upload via `POST /uploads/s3` |

### Tips for AI-Generated SEO

- **metaTitle**: Keep under 60 characters. Include shop name + primary keywords.
- **metaDescription**: Keep under 160 characters. Include a call-to-action.
- **keywords**: 5–10 relevant terms, comma-separated.
- **ogImage**: Use the shop's best hero/banner image. Upload via `/uploads/s3` first.

---

## Update Shop Name

```graphql
mutation UpdateName($shopId: String!, $name: String!) {
  shopUpdateName(shopId: $shopId, name: $name) { id name }
}
```

> ⚠️ **CONFIRMATION REQUIRED** — The shop name is public-facing and appears everywhere (header, browser tab, SEO, etc.). Confirm before changing.

---

## Update Shop Logo

Upload the logo first, then set it:

```graphql
mutation UpdateLogo($shopId: String!, $logo: String!) {
  shopUpdateLogo(shopId: $shopId, logo: $logo) { id logo }
}
```

**Steps:**
1. Upload image via `POST /uploads/s3` (field name `file`, returns CDN URL)
2. Call `shopUpdateLogo` with the CDN URL

> **Recommended**: Use a square PNG with transparent background, minimum 200×200px.

---

## Set Business Category

Assign the shop's business type (e.g. Fashion, F&B, Electronics).

### List Available Categories
```graphql
query BusinessCategories {
  getBusinessCategories { id name }
}
```

### Set Category
```graphql
mutation SetBusinessCategory($shopId: String!, $businessCategoryId: String!) {
  shopSetBusinessCategory(shopId: $shopId, businessCategoryId: $businessCategoryId) {
    id businessCatories { id name }
  }
}
```

---

## Agent Notes

- **SEO fields are optional**: You can update just `metaTitle` and leave the rest unchanged. Only send the fields the user wants to change.
- **Images need upload first**: `favicon` and `ogImage` require CDN URLs from `POST /uploads/s3`. Don't use placeholder URLs.
- **metaTitle default**: If not set, falls back to the shop name.
- **Common user requests**: "Improve my SEO", "Set meta description", "Change my shop name", "Update my logo".
