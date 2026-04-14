# HAQLY ERP — Dashboard Page Design

**Pattern**: Data-Dense Dashboard
**Style**: Soft UI Evolution
**Author**: Quadri Atharu

---

## Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│ Period Selector                    Company  |  Notifications │
├──────────────────────────────────────────────────────────────┤
│  KPI Revenue  │  KPI Expenses  │  KPI Net Income │ KPI Cash │
├──────────────────────────────────────────────────────────────┤
│  Revenue Chart (60%)          │  Quick Actions (40%)        │
├──────────────────────────────────────────────────────────────┤
│  Recent Transactions Table (60%) │ Activity Timeline (40%)  │
└──────────────────────────────────────────────────────────────┘
```

---

## KPI Cards

- 4-column grid on desktop, 2-column on tablet, 1-column on mobile
- Each card: accent left border (4px, semantic color per metric)
- Content: title (text-sm, secondary), value (text-2xl, mono font), change indicator (arrow + percentage)
- Sparkline: 30-day mini chart in card footer, 60px height
- Colors: Revenue = primary, Expenses = warning, Net Income = success, Cash = info

## Period Selector

- Positioned top-right of dashboard header
- Dropdown: "This Month", "This Quarter", "This Year", "Custom Range"
- Custom range opens date picker modal
- Period label displayed as: "Apr 2026" or "Q2 2026" or "FY 2026"

## Revenue Chart

- Line chart with area fill (primary-light)
- X-axis: daily/weekly/monthly based on period
- Y-axis: formatted NGN amounts
- Tooltip on hover with exact value
- Compare: previous period as dashed line in text-tertiary

## Quick Actions Panel

- Card with 8 action buttons in 2x4 grid
- Each button: icon (18px) + label, bg-surface, hover bg-surface-hover
- Actions: Create Invoice, Record Payment, Create Journal Entry, Run Payroll, Submit Tax Return, Create PO, View Reports, E-Invoice Submit

## Recent Transactions Table

- Dense table: 15 rows, columns: Date, Reference, Description, Account, Debit, Credit, Status
- Status badges: Posted (success-light), Pending (warning-light), Draft (bg-surface-hover)
- "View All" link at bottom to full transaction list
- Real-time refresh every 30 seconds

## Activity Timeline

- Vertical timeline with 10 most recent items
- Each item: icon (by type), description, timestamp (relative: "2m ago", "1h ago")
- Types: journal_posted (BookOpen), invoice_created (ShoppingCart), payment_received (CreditCard), tax_computed (Receipt), approval_pending (Clock)
- "View All Activity" link at bottom
