// Author: Quadri Atharu

export interface User {
  id: string; email: string; full_name: string; role: string;
  company_id: string; is_active: boolean; phone?: string; avatar_url?: string;
}
export interface Account {
  id: string; code: string; name: string; account_type: string; sub_type: string;
  balance: number; is_bank_account: boolean; is_tax_account: boolean;
  parent_account_id: string | null; is_active: boolean;
}
export interface JournalEntry {
  id: string; entry_number: string; date: string; narration: string;
  status: 'draft' | 'validated' | 'submitted' | 'approved' | 'posted' | 'reversed';
  total_debit: number; total_credit: number; reference: string | null;
  company_id: string; lines: JournalLine[];
}
export interface JournalLine {
  id: string; account_id: string; account_code: string; account_name: string;
  debit: number; credit: number; description: string | null;
  currency_code?: string; exchange_rate?: number;
}
export interface Invoice {
  id: string; invoice_number: string; customer_id: string; customer_name: string;
  date: string; due_date: string; status: 'draft' | 'sent' | 'paid' | 'overdue' | 'cancelled';
  subtotal: number; vat_amount: number; total: number; lines: InvoiceLine[];
  linked_invoice_id?: string;
}
export interface InvoiceLine {
  id: string; product_id: string; description: string; quantity: number;
  unit_price: number; vat_rate: number; line_total: number;
}
export interface Bill {
  id: string; bill_number: string; vendor_id: string; vendor_name: string;
  date: string; due_date: string; status: 'draft' | 'received' | 'approved' | 'paid' | 'cancelled';
  subtotal: number; wht_amount: number; total: number; bill_type: string; linked_bill_id?: string;
}
export interface Product {
  id: string; sku: string; name: string; category: string;
  quantity_on_hand: number; reorder_level: number; unit_cost: number;
  selling_price: number; warehouse_id: string; is_active: boolean;
}
export interface Employee {
  id: string; full_name: string; department: string; designation: string;
  gross_salary: number; start_date: string; status: string; tax_id?: string;
  pension_provider?: string; bank_name?: string; bank_account?: string;
}
export interface Payslip {
  id: string; employee_id: string; employee_name: string; period: string;
  gross_pay: number; total_deductions: number; net_pay: number;
  paye: number; pension_employee: number; pension_employer: number;
  nhf: number; nsitf: number; itf: number;
}
export interface Asset {
  id: string; asset_number: string; name: string; category: string;
  cost: number; residual_value: number; useful_life_years: number;
  depreciation_method: string; accumulated_depreciation: number;
  net_book_value: number; location: string | null; status: string;
}
export interface Loan {
  id: string; loan_number: string; lender: string; principal: number;
  interest_rate: number; outstanding_balance: number; monthly_payment: number;
  next_due_date: string; maturity_date: string; loan_type: string;
}
export interface Contact {
  id: string; name: string; company: string; email: string; phone: string;
  type: string; last_activity: string | null;
}
export interface Deal {
  id: string; name: string; contact_id: string; value: number;
  expected_close_date: string; stage: string; probability: number;
}
export interface TrialBalanceRow {
  account_code: string; account_name: string; account_type: string;
  debit_balance: number; credit_balance: number;
}
export interface TrialBalance {
  rows: TrialBalanceRow[]; total_debit: number; total_credit: number; is_balanced: boolean;
}
export interface IncomeStatement {
  revenue: { items: { name: string; amount: number }[]; total: number };
  cost_of_sales: { items: { name: string; amount: number }[]; total: number };
  gross_profit: number;
  operating_expenses: { items: { name: string; amount: number }[]; total: number };
  operating_profit: number;
  finance_costs: { items: { name: string; amount: number }[]; total: number };
  tax_expense: { items: { name: string; amount: number }[]; total: number };
  net_profit_loss: number;
}
export interface BalanceSheet {
  non_current_assets: { items: { name: string; amount: number }[]; total: number };
  current_assets: { items: { name: string; amount: number }[]; total: number };
  total_assets: number;
  non_current_liabilities: { items: { name: string; amount: number }[]; total: number };
  current_liabilities: { items: { name: string; amount: number }[]; total: number };
  total_liabilities: number;
  equity: { items: { name: string; amount: number }[]; total: number };
  total_equity: number;
  is_balanced: boolean;
}
export interface CashFlowStatement {
  operating: { items: { name: string; amount: number }[]; total: number };
  investing: { items: { name: string; amount: number }[]; total: number };
  financing: { items: { name: string; amount: number }[]; total: number };
  net_change: number; opening_cash: number; closing_cash: number;
}
export interface KPIWidget {
  title: string; value: number; previous_value: number;
  change_percent: number; trend: 'up' | 'down' | 'flat';
}
export interface PaginatedResponse<T> {
  data: T[]; next_cursor: string | null; has_more: boolean; total_count: number;
}
export interface ApiError {
  code: string; message: string; details?: string;
}
