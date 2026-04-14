# HAQLY ERP — Master Design System

**Product**: HAQLY ERP Desktop (Enterprise Financial ERP)
**Author**: Quadri Atharu
**Version**: 1.0.0
**Last Updated**: 2026-04-14

---

## 1. Product Identity

- **Type**: Enterprise Financial ERP Desktop Application
- **Platform**: Desktop-first (Tauri + Next.js), with responsive web fallback
- **Audience**: Nigerian SMB accountants, finance officers, tax professionals, business owners
- **Brand Values**: Trust, Precision, Compliance, Premium, Nigerian Identity

---

## 2. Design Pattern

**Data-Dense Dashboard (BI/Analytics Style #1)**

- Executive dashboard layout for complex data analysis
- Information density prioritized over whitespace
- Multi-column grids for KPI cards, tables, and charts
- Collapsible sidebars and filter panels to maximize viewport
- Scroll-within-panels pattern (each panel scrolls independently)
- Tab-based navigation within workspaces for related data

---

## 3. Visual Style

**Soft UI Evolution (#19)**

- Modern enterprise apps, SaaS aesthetic
- Soft shadows, subtle depth, premium feel
- Performance: Excellent (minimal animation, CSS-only effects)
- Accessibility: WCAG AA compliant

### Keywords
Soft shadows, subtle depth, premium feel, professional, trustworthy, Nigerian finance

---

## 4. Color System

### Primary Palette
| Token | Hex | Usage |
|---|---|---|
| `primary` | `#1B4332` | Forest Green — trust, finance, Nigerian identity |
| `primary-hover` | `#2D6A4F` | Primary hover state |
| `primary-light` | `rgba(27,67,50,0.12)` | Subtle primary backgrounds |
| `secondary` | `#2D6A4F` | Secondary actions, navigation |
| `accent` | `#D4AF37` | Gold — premium, Nigerian heritage, highlights |
| `accent-hover` | `#C4A030` | Accent hover state |

### Neutral Palette
| Token | Hex | Usage |
|---|---|---|
| `background` | `#F8F9FA` | App background |
| `surface` | `#FFFFFF` | Cards, panels, modals |
| `surface-hover` | `#F1F3F5` | Hovered surface |
| `surface-elevated` | `#FFFFFF` | Elevated elements (dropdowns, tooltips) |
| `border` | `#DEE2E6` | Default borders |
| `border-subtle` | `#E9ECEF` | Subtle dividers |
| `text` | `#1A1A2E` | Primary text |
| `text-secondary` | `#495057` | Secondary text, descriptions |
| `text-tertiary` | `#868E96` | Muted text, placeholders |
| `text-inverse` | `#FFFFFF` | Text on dark backgrounds |

### Semantic Colors
| Token | Hex | Usage |
|---|---|---|
| `error` | `#DC3545` | Errors, danger, destructive actions |
| `error-light` | `rgba(220,53,69,0.12)` | Error backgrounds |
| `success` | `#198754` | Success states, positive values |
| `success-light` | `rgba(25,135,84,0.12)` | Success backgrounds |
| `warning` | `#FFC107` | Warnings, pending states |
| `warning-light` | `rgba(255,193,7,0.12)` | Warning backgrounds |
| `info` | `#0DCAF0` | Informational, neutral highlights |
| `info-light` | `rgba(13,202,240,0.12)` | Info backgrounds |

### Dark Mode (Future)
All tokens have planned dark mode equivalents. Current production: Light mode only.

---

## 5. Typography

### Font Stack
| Role | Font | Fallback | Weight |
|---|---|---|---|
| UI | Inter | -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif | 400, 500, 600 |
| Heading | DM Serif Display | Georgia, serif | 400, 700 |
| Mono | JetBrains Mono | "Fira Code", monospace | 400, 500 |

### Scale
| Token | Size | Line Height | Usage |
|---|---|---|---|
| `text-xs` | 0.75rem (12px) | 1.5 | Badges, labels |
| `text-sm` | 0.875rem (14px) | 1.5 | Table cells, secondary text |
| `text-base` | 1rem (16px) | 1.6 | Body text, inputs |
| `text-lg` | 1.125rem (18px) | 1.5 | Section titles |
| `text-xl` | 1.25rem (20px) | 1.4 | Page titles |
| `text-2xl` | 1.5rem (24px) | 1.3 | KPI values |
| `text-3xl` | 1.875rem (30px) | 1.2 | Dashboard hero numbers |
| `text-4xl` | 2.25rem (36px) | 1.1 | Splash/logo |

---

## 6. Spacing Scale

| Token | Value | Usage |
|---|---|---|
| `space-1` | 4px | Inline gaps, icon padding |
| `space-2` | 8px | Compact padding, tight gaps |
| `space-3` | 12px | Standard padding, input gaps |
| `space-4` | 16px | Card padding, section gaps |
| `space-5` | 20px | Panel padding |
| `space-6` | 24px | Section spacing |
| `space-8` | 32px | Major section breaks |
| `space-10` | 40px | Page-level spacing |
| `space-12` | 48px | Hero spacing |

---

## 7. Shadows

| Token | Value | Usage |
|---|---|---|
| `shadow-xs` | `0 1px 2px rgba(0,0,0,0.04)` | Subtle elevation |
| `shadow-sm` | `0 2px 8px rgba(0,0,0,0.08)` | Cards, inputs |
| `shadow-md` | `0 4px 16px rgba(0,0,0,0.10)` | Dropdowns, popovers |
| `shadow-lg` | `0 8px 24px rgba(0,0,0,0.12)` | Modals, overlays |
| `shadow-xl` | `0 12px 40px rgba(0,0,0,0.16)` | Toast, full-screen overlay |

---

## 8. Border Radii

| Token | Value | Usage |
|---|---|---|
| `radius-sm` | 4px | Badges, small elements |
| `radius-md` | 8px | Buttons, inputs |
| `radius-lg` | 12px | Cards, panels |
| `radius-xl` | 16px | Modals, dialogs |
| `radius-full` | 9999px | Avatars, pills |

---

## 9. Transitions

| Token | Value | Usage |
|---|---|---|
| `transition-fast` | `150ms ease` | Hover states, focus rings |
| `transition-base` | `200ms ease` | Default transitions |
| `transition-slow` | `300ms ease` | Modals, page transitions |

---

## 10. Component Specifications

### Buttons
- **Primary**: `bg-primary text-inverse`, hover `bg-primary-hover`, shadow-sm on hover
- **Secondary**: `bg-surface border-border text-text`, hover `bg-surface-hover`
- **Ghost**: `transparent text-text-secondary`, hover `bg-surface-hover`
- **Danger**: `bg-error text-inverse`, hover darker red
- **Sizes**: sm (28px h, 4px 10px pad), md (36px h, 8px 16px pad), lg (44px h, 12px 24px pad)
- **Focus ring**: 2px outline primary with 2px offset
- **Icon gap**: 6px

### Inputs
- **Default**: `bg-surface border-border text-text`, h 36px, pad 8px 12px
- **Focus**: border-primary, shadow `0 0 0 3px primary-light`
- **Error**: border-error, helper text in error color
- **Label**: text-sm, font-weight 500, text-secondary, mb space-1
- **Helper**: text-xs, text-tertiary

### Cards
- **Default**: `bg-surface border-border radius-lg shadow-sm pad space-5`
- **Hover**: shadow-md, border subtle shift
- **KPI Card**: accent left border (4px), compact padding

### Tables
- **Header**: bg-surface-hover, text-xs, text-secondary, uppercase, tracking-wide, h 40px
- **Row**: h 44px, border-bottom border-subtle, hover bg-surface-hover
- **Striped**: alternating bg-background every other row
- **Dense**: h 36px, reduced padding

### Modals
- **Overlay**: bg-text at 40% opacity
- **Container**: bg-surface, radius-xl, shadow-xl, max-width 640px
- **Header**: text-lg font-semibold, border-bottom, pad space-5
- **Footer**: border-top, pad space-4, right-aligned actions

### Toasts
- **Position**: bottom-right, offset 24px from edges
- **Success**: bg-success-light, border-left 4px success
- **Error**: bg-error-light, border-left 4px error
- **Warning**: bg-warning-light, border-left 4px warning
- **Info**: bg-info-light, border-left 4px info
- **Auto-dismiss**: 5000ms, with manual close

### Navigation (Sidebar)
- **Width**: 260px collapsed, 72px mini
- **Item height**: 40px
- **Active**: bg-primary-light, text-primary, left border 3px primary
- **Hover**: bg-surface-hover
- **Icon**: 18px, gap 10px to label

---

## 11. Anti-Patterns

These are explicitly forbidden in HAQLY ERP:

- Bright neon colors (no #00FF00, #FF00FF, etc.)
- Harsh animations (no bounce, shake, rubber-band)
- AI purple/pink gradients (no generic AI aesthetic)
- Excessive whitespace in data-dense views (no 80px gutters between columns)
- Glassmorphism on data surfaces (blur + transparency kills readability)
- Flat design without depth cues (users need visual hierarchy)
- Rounded-everything (tables, inputs stay geometrically appropriate)
- Skeleton screens without purpose (use real loading states)

---

## 12. Responsive Breakpoints

| Breakpoint | Min Width | Layout |
|---|---|---|
| `desktop` | 1024px | Full sidebar, multi-column grids |
| `tablet` | 768px | Mini sidebar, 2-column grids |
| `mobile` | 375px | No sidebar (drawer), single column |

Desktop-first approach. All layouts designed at 1024px+ first.

---

## 13. Accessibility Requirements

- All interactive elements have visible focus rings (2px, primary color, 2px offset)
- Color contrast meets WCAG AA (4.5:1 for normal text, 3:1 for large text)
- Error states communicated via icon + text, not color alone
- ARIA labels on all interactive elements
- Keyboard navigation: Tab order follows visual order, Enter/Space activate
- Skip-to-content link on every page
- Reduced motion: respect `prefers-reduced-motion`
