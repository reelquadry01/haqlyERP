# HAQLY ERP — Tax Page Design

**Pattern**: Data-Dense Dashboard
**Style**: Soft UI Evolution
**Author**: Quadri Atharu

---

## Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│  Tax Management                    [Tax Year: 2026 ▾]       │
├──────────────────────────────────────────────────────────────┤
│  Tabs: [Configurations] [Computations] [Schedules] [Risk]   │
├──────────────────────────────────────────────────────────────┤
│  Content area per tab                                         │
└──────────────────────────────────────────────────────────────┘
```

---

## Tab: Configurations

- Rate tables grouped by tax type: VAT, WHT, PAYE, CIT, CGT, Stamp Duty, Edu Tax
- Each group collapsible, expandable
- Table columns: Tax Name, Rate, Effective From, Effective To, Status
- Status: Active (success badge), Expired (tertiary), Draft (warning)
- [+ Add Rate] button per tax type group
- Inline edit of rates

## Tab: Computations

- Computation cards in 2-column grid
- Each card: Tax type icon, period, taxable amount, computed tax, effective rate
- Expandable: shows breakdown of how tax was calculated
- PAYE computation card shows: consolidated salary, relief allowances, taxable income, tax bands breakdown
- VAT computation: output VAT collected, input VAT paid, net VAT payable
- WHT computation: deductions by category, credit available
- Action buttons per card: Export, Generate Schedule, Submit to FIRS

## Tab: Schedules

- Table of tax filing schedules for current year
- Columns: Tax Type, Period, Due Date, Amount, Status, Actions
- Status with color coding:
  - Overdue → error color, bold, with alert icon
  - Due Soon (7 days) → warning color
  - Filed → success color with checkmark
  - Not Due → tertiary
- Calendar view toggle (monthly calendar with due dates highlighted)

## Risk Flags

- Dedicated risk panel within Risk tab
- Color-coded risk indicators:
  - **High Risk** (error color): Late filing detected, WHT under-deducted, VAT gap > 10%
  - **Medium Risk** (warning color): Approaching deadline, unusual deduction pattern, rate change not applied
  - **Low Risk** (info color): Minor discrepancy, optimization opportunity
- Each flag: icon, description, recommended action, link to relevant module
- Risk score gauge (0-100) for overall tax compliance
- Historical risk trend chart (6 months)
