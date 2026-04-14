# HAQLY ERP — Journal Entries Page Design

**Pattern**: Data-Dense Dashboard
**Style**: Soft UI Evolution
**Author**: Quadri Atharu

---

## Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│  Journal Entries    [Draft(5)] [Submitted(3)] [Posted(12)] │
├──────────────────────────────────────────────────────────────┤
│  Search/Filter Bar               │  [+ New Journal Entry]   │
├──────────────────────────────────────────────────────────────┤
│  Journal Entries Table (full width)                          │
│  Ref | Date | Narration | Debit Total | Credit Total | Status│
├──────────────────────────────────────────────────────────────┤
│  Selected Entry Detail Panel (expandable)                    │
│  Approval Stepper → Line Item Grid → Attachments             │
└──────────────────────────────────────────────────────────────┘
```

---

## Tab Navigation

- Three tabs at top: Draft, Submitted, Posted
- Tab counts shown as badges (primary-light bg)
- Default tab: Draft
- Switching tabs filters the table without page reload

## Create Entry Dialog

- Modal, max-width 960px
- Header: "New Journal Entry" + company name
- **Header Fields**: Date (date picker), Reference (auto-generated, editable), Narration (textarea)
- **Line Item Grid**: Inline editable table
  - Columns: Account (searchable dropdown from COA), Debit, Credit, Cost Center, Description
  - Add row button at bottom
  - Delete row button per row (icon only)
  - Running totals shown at bottom: Total Debit, Total Credit, Difference
  - **Validation**: Difference must be 0 (balanced entry) before submit
  - Unbalanced indicator: red highlight on difference field with "Entry not balanced" message
- **Footer**: Cancel, Save Draft, Submit for Approval

## Approval Stepper

- Shown in detail panel when entry is in Submitted status
- Steps: Created → Reviewed → Approved → Posted
- Each step shows: user, timestamp, optional comment
- Current step highlighted with primary color
- Completed steps show checkmark icon
- Actions per step: Approve, Reject (with reason modal), Request Revision

## Line Item Grid

- Displayed in detail panel below stepper
- Read-only view of all line items
- Accounts displayed as: code + name (e.g., "1001 - Cash Account")
- Amounts right-aligned, formatted NGN
- Totals row at bottom with double-underline convention

## Balanced Entry Validation

- Real-time balance check as line items are edited
- Visual indicator: green check when balanced, red warning when not
- Difference = Total Debit - Total Credit displayed prominently
- Submit button disabled until balanced
