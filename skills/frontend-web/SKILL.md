# SKILL.md: Frontend Web & UI Architecture (V4)

This skill dictates the absolute standard for frontend web development, UI architecture, and Vibe Coding orchestration. It encompasses the "Universal Synthesis" (Light) and "Obsidian" (Dark) design systems, component-level specs, animation choreography, and strict rules for Next.js, Tailwind v4, and Claude Code delegation.

## 1. Core Directives & Vibe Coding Protocol

* **The Boilerplate Failsafe:** NEVER bootstrap a Next.js project from scratch using `npx create-next-app` when building a landing page or full-stack app. This leads to unstable CSS architectures and Claude Code hallucinating configs. **ALWAYS clone the official template:** `https://github.com/KOOMPICloud/landing-page-shadcn` as the baseline.
* **Tailwind v4 Compilation Rule:** Claude Code struggles with dynamic Tailwind v4 utility generation. If custom variables (like `bg-[var(--paper)]`) fail to compile, explicitly write out raw standard classes (e.g., `bg-[#FBFBFA]`, `text-[#1D1D1F]`, `p-8`) in the components.
* **High-Fidelity UI Shortcut:** If Claude Code drops layout padding or hallucinates while converting a pixel-perfect HTML/Tailwind mockup into React/Next.js components, DO NOT let it guess. Manually convert the markup to JSX, or drop the raw HTML into the `public` folder and use a Next.js `redirect()` for perfect fidelity.
* **Design Token Injection:** When spawning Claude Code, ALWAYS provide the exact hex values from this file. Never let Claude "pick" colors, fonts, or spacing. Provide the blueprint; Claude executes.
* **Claude Code Anti-Patterns:**
  * NEVER let Claude use emoji in UI — always remind it to use Lucide icons.
  * NEVER let Claude add generic purple/blue gradients — enforce Obsidian palette.
  * NEVER let Claude use system-ui or sans-serif fallbacks — specify exact font families.
  * ALWAYS audit Claude's output against this spec before reporting completion.

---

## 2. Design Tokens

### Spacing System (4px base unit)
```
4px  · micro (icons padding, badge gaps)
8px  · xs    (inline spacing, icon-text gaps)
12px · sm    (list item padding, tight card padding)
16px · md    (card internal padding, form element padding)
20px · lg    (section element gaps, card margins)
24px · xl    (section padding on mobile, card padding)
32px · 2xl   (section padding on tablet, grouped element gaps)
48px · 3xl   (section padding on desktop, hero vertical rhythm)
64px · 4xl   (major section dividers, page-level vertical rhythm)
96px · 5xl   (hero top/bottom padding on desktop)
128px· 6xl   (page-level hero breathing room
```

### Border Radius
```
4px   · badges, small tags
8px   · buttons, inputs, small cards
12px  · medium cards, code blocks
16px  · large cards, modal containers
24px  · hero cards, bento grid items
32px  · full-width feature panels, pill buttons
9999px· pill shapes (CTA buttons, nav items)
```

### Elevation & Shadows
**System A (Light):**
```
xs:   none — use 1px borders only
sm:   0 1px 3px rgba(0,0,0,0.04)
md:   0 4px 12px rgba(0,0,0,0.06)
hover: 0 8px 24px rgba(0,0,0,0.08)
```
**System B (Dark):**
```
glow-xs:  0 0 0 1px rgba(0,240,255,0.1)
glow-sm:  0 0 20px rgba(0,240,255,0.08)
glow-md:  0 0 40px rgba(0,240,255,0.12)
glow-hover:0 0 60px rgba(0,240,255,0.15)
```

### Color Palettes

**System A (Universal Synthesis — Light):**
```
Backgrounds:   #FBFBFA (page), #F5F5F7 (section-alt), #FFFFFF (cards)
Text Primary:  #1D1D1F (headings, body)
Text Secondary:#6B7280 (descriptions, captions)
Text Tertiary: #9CA3AF (disabled, placeholders)
Borders:       rgba(0,0,0,0.06) (dividers), rgba(0,0,0,0.1) (cards)
Accent:        #2563EB (interactive, links, CTA)
Accent Hover:  #1D4ED8
Success:       #10B981
Warning:       #F59E0B
Error:         #EF4444
```

**System B (Obsidian — Dark):**
```
Backgrounds:   #000000 (page), #09090B (section-alt), #111113 (cards)
Surface:       rgba(255,255,255,0.03) (subtle fill), rgba(255,255,255,0.05) (glassmorphism)
Text Primary:  #FAFAFA (headings)
Text Secondary:#A1A1AA (body)
Text Tertiary: #71717A (captions, disabled)
Borders:       rgba(255,255,255,0.06) (dividers), rgba(255,255,255,0.1) (cards, glass)
Accent:        #00F0FF (neon cyan — use sparingly)
Accent Alt:    #818CF8 (indigo — secondary actions)
Success:       #34D399
Warning:       #FBBF24
Error:         #F87171
```

### Gradient Rules
**ALLOWED:**
- Subtle mesh: `radial-gradient(ellipse at 20% 50%, rgba(0,240,255,0.06) 0%, transparent 60%)`
- Accent fade: `linear-gradient(135deg, #00F0FF 0%, #818CF8 100%)` — for text gradient only, not backgrounds
- Section atmosphere: `radial-gradient(circle at 50% 0%, rgba(0,240,255,0.04) 0%, transparent 50%)`

**FORBIDDEN (no exceptions):**
- Generic purple/blue hero gradients
- Rainbow/multicolor mesh
- Heavy gradient backgrounds that fight with content
- CSS `background-blend-mode` hacks

---

## 3. Typography

### Font Families
```
Display/Heading:  'Plus Jakarta Sans' (700-800) or 'Satoshi' (700-800)
Subheading:       'Plus Jakarta Sans' (600) or 'Satoshi' (600)
Body:             'Inter' (400-500)
Code/Technical:   'JetBrains Mono' (400-500)
Eyebrow/Overline: 'Inter' (600, uppercase, letter-spacing: 0.12em, 0.75rem)
```

### Typographic Scale (fluid via clamp)
```
Hero H1:    clamp(2.5rem, 5vw + 1rem, 5.5rem)   · line-height: 1.05 · letter-spacing: -0.04em
Section H2:  clamp(2rem, 3vw + 0.5rem, 3.5rem)    · line-height: 1.1  · letter-spacing: -0.03em
Card H3:     1.25rem–1.5rem                        · line-height: 1.3  · letter-spacing: -0.01em
Body:        1rem–1.125rem                          · line-height: 1.6
Small/Caption: 0.875rem                             · line-height: 1.5
Eyebrow:     0.75rem                                · letter-spacing: 0.12em · uppercase · weight: 600
```

### Font Loading
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&family=Plus+Jakarta+Sans:wght@600;700;800&display=swap" rel="stylesheet">
```
For Next.js, use `next/font/google` with `display: 'swap'` and `subsets: ['latin']`.

### Typographic Rules
- Headings: NEVER use titlecase. Sentence case only (e.g., "Cloud infrastructure" not "Cloud Infrastructure").
- Paragraphs: max-width `65ch` for optimal readability.
- Don't center body text. Left-align everything except hero headlines and eyebrow labels.
- Avoid more than 3 font weights on a single page.

---

## 4. Component Specifications

### Buttons
```
Primary (System A):
  bg: #2563EB → hover: #1D4ED8
  text: #FFFFFF · weight: 600 · size: 0.9375rem
  padding: 12px 24px · border-radius: 12px
  transition: all 0.2s cubic-bezier(0.2, 1, 0.3, 1)

Primary (System B):
  bg: #00F0FF · text: #000000 · weight: 700
  OR bg: transparent + border 1px solid rgba(0,240,255,0.3) + text: #00F0FF
  padding: 12px 24px · border-radius: 12px
  hover: glow effect (box-shadow: 0 0 30px rgba(0,240,255,0.2))

Secondary (both systems):
  bg: transparent · border: 1px solid [border-color]
  text: [text-primary] · weight: 600
  hover: bg [surface]

Ghost (both systems):
  bg: transparent · no border
  hover: bg [surface/subtle]

Button sizes:
  sm: 8px 16px · text: 0.8125rem
  md: 12px 24px · text: 0.9375rem
  lg: 16px 32px · text: 1rem
```

### Cards (Bento Grid)
```
Padding: 24px (mobile), 32px (desktop)
Border-radius: 24px
Border: 1px solid [border-color]
Background: [card-bg]

Hover state:
  scale: 1.01 · transition: 0.3s cubic-bezier(0.2, 1, 0.3, 1)
  System A: subtle shadow elevation (0 8px 24px rgba(0,0,0,0.08))
  System B: border glow (0 0 0 1px rgba(0,240,255,0.15))

Bento grid gap: 16px (mobile), 24px (desktop)
Card icon: 40px–48px · Lucide · [accent-color]
Card title: [H3 spec]
Card description: [body spec] · color: [text-secondary]
```

### Inputs & Forms
```
Height: 44px (standard), 40px (compact)
Padding: 0 16px
Border: 1px solid [border-color] · radius: 8px
Background: [card-bg]
Focus: border-color: [accent] + outline: 2px solid [accent] offset 2px
Placeholder color: [text-tertiary]
Font: [body spec]
Error: border-color: #EF4444 + text: #EF4444 below field
```

### Tags & Badges
```
Padding: 4px 12px · border-radius: 9999px
Font: 0.75rem · weight: 500
Default: bg: [surface] · text: [text-secondary]
Accent: bg: [accent]/10 · text: [accent] (e.g., bg: rgba(0,240,255,0.1) + text: #00F0FF)
Success: bg: #10B981/10 · text: #10B981
Error: bg: #EF4444/10 · text: #EF4444
```

### Navigation
```
Height: 64px–72px
Background: [page-bg] + backdrop-blur: 12px (glassmorphism for System B)
Border-bottom: 1px solid [border-color]
Logo: [Display font] · weight: 700 · text: [text-primary]
Nav links: [body spec] · weight: 500 · color: [text-secondary]
Active link: color: [text-primary]
Mobile: hamburger menu (Lucide Menu/X icons), slide-in panel from right
```

### Footer
```
Padding: 48px 24px (mobile), 64px 24px (desktop)
Border-top: 1px solid [border-color]
Columns: 4 (desktop), 2 (tablet), 1 (mobile)
Link style: [body spec] · color: [text-secondary] · hover: color: [text-primary]
Copyright: [caption spec] · color: [text-tertiary]
```

---

## 5. Section Composition Patterns

### Hero Section
```
Layout: max-width 1280px, centered, padding 96px 24px top/bottom (mobile: 48px 24px)
Content: Eyebrow → H1 (max 8 words for punch) → Subtext (max 65ch) → CTA buttons (2 max)
Alignment: center (default) or left-aligned for documentation-heavy pages
Visual: Optional hero illustration/graphic below text or beside on desktop
Background: [page-bg] with optional subtle radial gradient
Animation: Stagger entrance — eyebrow fades in 0.3s, H1 0.5s (delay 0.1s), body 0.5s (delay 0.2s), buttons 0.5s (delay 0.3s)
```

### Features / Bento Grid
```
Layout: max-width 1280px, centered, gap 24px
Grid: 3 columns (desktop), 2 (tablet), 1 (mobile)
Card: See Card spec above
Section header: Eyebrow → H2 → Optional description (centered)
Max cards: 6 per section. More? Split into two sections.
```

### Testimonials / Social Proof
```
Layout: max-width 1024px, centered
Cards: Quote text → Author name → Author role/company
Style: Clean quote marks (Lucide Quote icon), no heavy decoration
Grid: 2-3 columns (desktop), 1 (mobile)
Optional: Logo strip below (gray/monochrome company logos, 48px height, auto-width)
```

### Pricing
```
Layout: max-width 1024px, centered
Cards: 2-3 columns (desktop), 1 (mobile)
Popular tier: Highlighted border (accent color), "Popular" badge, scale: 1.02
Card content: Plan name → Price (H2) → Description → Feature list (checkmarks) → CTA button
Feature list: Lucide Check icon (16px) + text per line
```

### CTA / Final Call-to-Action
```
Layout: Full-width section, centered, padding 96px 24px
Background: [accent] for light mode, or gradient background for dark mode
Content: H2 → Description → Single CTA button (high contrast)
```

---

## 6. Icons & Imagery

* **NO EMOJI IN UI.** Emoji are unprofessional, inconsistent across platforms, and break visual hierarchy. Never use them in buttons, cards, nav, headings, or any UI element.
* **Primary Icon Library:** **Lucide Icons** (`lucide-react` for React, `lucide` for vanilla JS).
  * CDN (vanilla HTML): `<script src="https://unpkg.com/lucide@latest"></script>` then `<i data-lucide="icon-name"></i>` + `lucide.createIcons()`.
  * React: `import { Code, Cloud, Smartphone } from 'lucide-react'`.
  * Prefer: outline style, 24px default, 1.5px stroke-width.
  * Search: https://lucide.dev/icons
* **Custom SVG Icons:** When Lucide doesn't have the exact icon (brand logos, custom illustrations), create inline SVGs. Rules:
  * 24x24 viewBox, `currentColor` for stroke/fill where possible.
  * 1.5px stroke, `stroke-linecap: round`, `stroke-linejoin: round`.
* **Icon Sizing:**
  * Inline with text: 16px (`w-4 h-4`)
  * Card/service icons: 24px (`w-6 h-6`)
  * Hero/feature highlights: 32px–40px (`w-8 h-8` to `w-10 h-10`)
  * Never below 14px or above 48px for standard UI icons.

### Image Treatment
* **Radius:** 12px for standalone images, 0px for full-bleed.
* **Loading:** Always use `loading="lazy"` for below-fold images. Hero image = `loading="eager"` + `fetchpriority="high"`.
* **Object-fit:** `cover` for card thumbnails, `contain` for logos/screenshots.
* **Aspect Ratios:** 16:9 (hero/cards), 4:3 (features), 1:1 (testimonials/avatars).
* **Next.js:** Use `next/image` with `sizes` prop and responsive `width`/`height`.

---

## 7. Interaction Physics & Animation Choreography

### Easing
```
Standard:    cubic-bezier(0.2, 1, 0.3, 1)   — snappy entrance/exit
Spring:      cubic-bezier(0.34, 1.56, 0.64, 1) — bouncy micro-interactions
Smooth:      cubic-bezier(0.4, 0, 0.2, 1)     — page transitions
Decelerate:  cubic-bezier(0, 0, 0.2, 1)        — content entering viewport
```

### Hover Micro-interactions
* **Cards:** `scale(1.01)` + border glow or shadow elevation. Duration: 300ms. Easing: Standard.
* **Buttons:** `scale(1.02)` on press (100ms), release snaps back. Easing: Spring.
* **Links:** Underline slides in from left. `background-size: 100% 1px` transition.
* **Icons:** Subtle rotate (5deg) on hover for decorative icons. Never rotate action icons.

### Entrance Animations (Viewport-triggered)
```
Fade Up (default):    opacity 0→1, translateY 20px→0 · 0.6s · stagger 0.1s per item
Fade In:              opacity 0→1 · 0.4s
Scale In:             opacity 0→1, scale 0.95→1 · 0.5s (for modals, tooltips)
Slide In (panels):    translateX -100%→0 (left panel) or 100%→0 (right panel) · 0.4s
Blur In:              opacity 0→1, blur 10px→0 · 0.6s (for hero content)
```

### Scroll-triggered Patterns
* **Stagger reveal:** Children animate in sequence (0.08s–0.12s delay between each).
* **Parallax:** Background elements move at 0.5x scroll speed. Subtle only.
* **Counter animation:** Numbers count up from 0 when entering viewport (for stats/metrics).
* **Progress bars:** Width animates from 0% to target when in view.

### Reduced Motion
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

### Live Activity Indicators
* Inject small animated pulse dots (green or cyan) for live system state, AI processing, or connectivity.
* Use Lucide `Activity` icon + CSS pulse animation, NOT emoji.

---

## 8. Loading & Skeleton States

### Skeleton Screens
```
Background: [surface] or [border-color]
Animation: shimmer — linear gradient moving left to right
  background: linear-gradient(90deg, [surface] 25%, [surface-light] 50%, [surface] 75%)
  background-size: 200% 100%
  animation: shimmer 1.5s infinite
Shape: match the content shape (rounded for text lines, circular for avatars)
Radius: match component radius (8px for inputs, 24px for cards)
```

### Loading Indicators
* **Spinner:** 20px, 2px border, border-color: [accent] with transparent top, CSS spin 0.6s linear infinite.
* **Dots:** Three dots, stagger-bounce animation, 6px each, gap 4px.
* **Progress bar:** Full-width thin bar (3px height), [accent] background, indeterminate animation.

---

## 9. Content Engineering

* **Realistic Copy:** Write like an engineer building the future. Use precise terms ("Immutable", "Partition", "A/B Testing", "Telemetry"). No generic marketing fluff.
* **Code as Art:** Embed small snippets of JSON, terminal outputs, or system logs into the UI to ground the design in technical reality.
* **Microcopy:** Buttons should be verbs ("Deploy Now", "Start Building") not nouns ("Deployment", "Getting Started").
* **Empty States:** Never show a blank screen. Include an illustration + descriptive text + primary action button.
* **Truncation:** Use `text-overflow: ellipsis` for long text. Tooltip on hover for full text.

---

## 10. Accessibility & Responsiveness

* **Focus States:** Visible focus rings on all interactive elements (`outline: 2px solid #2563EB`, `outline-offset: 2px`).
* **Reduced Motion:** Wrap all GSAP/Framer Motion in `prefers-reduced-motion` checks. Disable parallax and reduce stagger to instant state changes.
* **Responsive:** Mobile-first. Breakpoints at `768px` (tablet) and `1024px` (desktop). Fluid typography via `clamp()`.
* **Color Contrast:** Body text on light bg must be `#1D1D1F` or darker. On dark bg, body text must be `#D4D4D8` or lighter. WCAG AA minimum.
* **Touch Targets:** Minimum 44x44px for all interactive elements on mobile.
* **Skip Link:** Include a "Skip to content" link at the top of every page (visually hidden, visible on focus).
* **Semantic HTML:** Use `<nav>`, `<main>`, `<section>`, `<article>`, `<footer>`. Never use `<div>` for landmark elements.
* **Image Alt Text:** Descriptive alt for content images, empty `alt=""` for decorative only.

---

## 11. The Dual Design Systems

### System A: Universal Synthesis (Light Mode / "Industrial Apple")
Used for documentation, dashboards, agency sites, and technical manifestos.
* **Material Honesty:** Use off-whites (`#FBFBFA`, #F5F5F7`) for backgrounds. Pure white (`#FFFFFF`) reserved ONLY for cards.
* **Bento Architecture:** Strict grid of cards with high border-radii (24px to 32px).
* **Invisible Depth:** No heavy `box-shadow`. Use 1px solid borders to separate space.
* **Motion:** Clean, precise. Fade-ups and scale-ins. No bounce.

### System B: Obsidian (Dark Mode / "KOOMPI Ecosystem")
Used for consumer-facing landing pages, AI tools, and immersive experiences.
* **Deep Space:** Absolute black (`#000000`) or deep zinc (`#09090B`) bases.
* **Precision Glassmorphism:** `backdrop-filter: blur(16px)` with `bg-white/5` and 1px `white/10` borders. Do NOT use generic purple/blue "AI slop" gradients.
* **Neon Accents:** Vivid Cyan (`#00F0FF`) or Cobalt Blue used sparingly — buttons, glow rings, active states only.
* **Advanced Motion:** GSAP or Framer Motion for 3D tilt effects, magnetic buttons, staggered reveals.
* **Atmosphere:** Subtle radial gradients, noise texture overlay (CSS SVG filter), depth via layered glass panels.

---

## 12. Execution Trigger

When Boss requests a frontend component, UI, or webpage:

1. **Identify the system:** System A (Light) or System B (Dark). When in doubt, ask.
2. **Identify section patterns needed:** Hero, Features, Pricing, Testimonials, CTA, etc.
3. **Spawn Claude Code** with this file's exact path injected into the prompt.
4. **Provide the blueprint:** Specify fonts, colors, spacing, and component specs from this document. Do NOT let Claude choose.
5. **Enforce the rules:** No emoji in UI. No generic gradients. Exact font families. Exact hex values.
6. **Audit the output:** Verify Claude followed the spec. Fix any violations before reporting completion.
7. **Deploy:** Follow the KConsole deployment protocol (remove `.env*`, zip, upload, verify build).
