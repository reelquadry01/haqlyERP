// Author: Quadri Atharu
use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use csv::Writer;
use rust_xlsxwriter::Workbook;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::reports_dto::{
    AccountStatement, AccountStatementRow, BalanceSheet, BalanceSheetSection, CashFlowStatement,
    IncomeStatement, RatioAnalysis, RetainedEarnings, StatementLine, TrialBalance, TrialBalanceRow,
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct AccountBalanceRow {
    id: Uuid,
    code: String,
    name: String,
    account_type: String,
    sub_type: Option<String>,
    balance: BigDecimal,
}

#[derive(Clone)]
pub struct ReportsService {
    pub pool: PgPool,
}

impl ReportsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_balances_as_of(
        &self,
        company_id: Uuid,
        as_of_date: NaiveDate,
    ) -> Result<Vec<AccountBalanceRow>> {
        Ok(sqlx::query_as::<_, AccountBalanceRow>(
            r#"SELECT a.id, a.code, a.name, a.account_type, a.sub_type,
                      COALESCE(b.balance, 0) AS balance
               FROM chart_of_accounts a
               LEFT JOIN (
                   SELECT jl.account_id, SUM(jl.debit - jl.credit) AS balance
                   FROM journal_lines jl
                   JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jh.company_id = $1 AND jh.status = 'posted' AND jh.date <= $2
                   GROUP BY jl.account_id
               ) b ON b.account_id = a.id
               WHERE a.company_id = $1 AND a.is_active = true
               ORDER BY a.code"#,
        )
        .bind(company_id)
        .bind(as_of_date)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn fetch_balances_for_period(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<Vec<AccountBalanceRow>> {
        Ok(sqlx::query_as::<_, AccountBalanceRow>(
            r#"SELECT a.id, a.code, a.name, a.account_type, a.sub_type,
                      COALESCE(b.balance, 0) AS balance
               FROM chart_of_accounts a
               LEFT JOIN (
                   SELECT jl.account_id, SUM(jl.debit - jl.credit) AS balance
                   FROM journal_lines jl
                   JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jh.company_id = $1 AND jh.status = 'posted'
                         AND jh.date >= $2 AND jh.date <= $3
                   GROUP BY jl.account_id
               ) b ON b.account_id = a.id
               WHERE a.company_id = $1 AND a.is_active = true
               ORDER BY a.code"#,
        )
        .bind(company_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&self.pool)
        .await?)
    }

    fn is_current_asset(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if ["ppe", "intangible", "investment property", "long-term", "non-current", "fixed asset"]
                .iter().any(|k| s.contains(k))
            {
                return false;
            }
            if ["cash", "bank", "receivable", "inventory", "prepayment", "short-term", "current"]
                .iter().any(|k| s.contains(k))
            {
                return true;
            }
        }
        let code: i32 = row.code.parse().unwrap_or(0);
        code > 0 && code < 1500
    }

    fn is_current_liability(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if ["long-term", "deferred", "non-current"]
                .iter().any(|k| s.contains(k))
            {
                return false;
            }
            if ["payable", "short-term", "tax payable", "accrued", "current"]
                .iter().any(|k| s.contains(k))
            {
                return true;
            }
        }
        let code: i32 = row.code.parse().unwrap_or(0);
        code > 0 && code < 2500
    }

    fn is_cost_of_sales(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if s.contains("cost of sale") || s.contains("cost_of_sale") || s == "cogs" {
                return true;
            }
        }
        let n = row.name.to_lowercase();
        n.contains("cost of sale") || n.contains("cost of goods") || n == "cogs"
    }

    fn is_finance_cost(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if s.contains("finance cost") || s.contains("interest") || s.contains("bank charge") {
                return true;
            }
        }
        let n = row.name.to_lowercase();
        n.contains("interest expense") || n.contains("finance cost") || n.contains("bank charge")
            || n.contains("loan interest")
    }

    fn is_tax_expense(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if s.contains("tax") || s.contains("education") || s.contains("nase") {
                return true;
            }
        }
        let n = row.name.to_lowercase();
        n.contains("tax") || n.contains("education tax") || n.contains("nase")
            || n.contains("companies income tax")
    }

    fn is_depreciation(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if s.contains("depreciation") || s.contains("amortization") {
                return true;
            }
        }
        let n = row.name.to_lowercase();
        n.contains("depreciation") || n.contains("amortization")
    }

    fn is_cash_account(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            let s = st.to_lowercase();
            if s.contains("cash") || s.contains("bank") || s.contains("cash equivalent") {
                return true;
            }
        }
        let n = row.name.to_lowercase();
        n.contains("cash") || n.contains("bank")
    }

    fn is_dividend_account(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            if st.to_lowercase().contains("dividend") {
                return true;
            }
        }
        row.name.to_lowercase().contains("dividend")
    }

    fn is_retained_earnings(row: &AccountBalanceRow) -> bool {
        if let Some(ref st) = row.sub_type {
            if st.to_lowercase().contains("retained") {
                return true;
            }
        }
        row.name.to_lowercase().contains("retained")
    }

    pub async fn trial_balance(
        &self,
        company_id: Uuid,
        as_of_date: Option<NaiveDate>,
    ) -> Result<TrialBalance> {
        let effective_date = as_of_date.unwrap_or_else(|| chrono::Utc::now().naive_utc().date());
        let balances = self.fetch_balances_as_of(company_id, effective_date).await?;

        let mut rows = Vec::new();
        let mut total_debit = BigDecimal::from(0);
        let mut total_credit = BigDecimal::from(0);

        for row in &balances {
            if row.balance == BigDecimal::from(0) {
                continue;
            }

            let is_debit_nature = matches!(
                row.account_type.to_lowercase().as_str(),
                "asset" | "expense"
            );

            let (debit, credit) = if is_debit_nature {
                if row.balance > BigDecimal::from(0) {
                    (row.balance.clone(), BigDecimal::from(0))
                } else {
                    (BigDecimal::from(0), row.balance.clone().abs())
                }
            } else if row.balance < BigDecimal::from(0) {
                (row.balance.clone().abs(), BigDecimal::from(0))
            } else {
                (BigDecimal::from(0), BigDecimal::from(0))
            };

            total_debit = &total_debit + &debit;
            total_credit = &total_credit + &credit;

            rows.push(TrialBalanceRow {
                account_code: row.code.clone(),
                account_name: row.name.clone(),
                account_type: row.account_type.clone(),
                debit,
                credit,
            });
        }

        Ok(TrialBalance {
            rows,
            is_balanced: total_debit == total_credit,
            total_debit,
            total_credit,
        })
    }

    pub async fn account_statement(
        &self,
        account_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<AccountStatement> {
        let account = sqlx::query_as::<_, (Uuid, String, String)>(
            "SELECT id, code, name FROM chart_of_accounts WHERE id = $1",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        let opening_balance: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               WHERE jl.account_id = $1 AND jh.status = 'posted' AND jh.date < $2"#,
        )
        .bind(account_id)
        .bind(from_date)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let line_rows = sqlx::query_as::<_, (String, String, Option<String>, BigDecimal, BigDecimal)>(
            r#"SELECT jh.date::text, jh.entry_number, COALESCE(jl.narration, jh.narration),
                      jl.debit, jl.credit
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               WHERE jl.account_id = $1 AND jh.status = 'posted'
               AND jh.date >= $2 AND jh.date <= $3
               ORDER BY jh.date, jh.entry_number"#,
        )
        .bind(account_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&self.pool)
        .await?;

        let mut rows = Vec::new();
        let mut running_balance = opening_balance.clone();

        for (date, entry_number, narration, debit, credit) in line_rows {
            running_balance = &running_balance + &debit - &credit;
            rows.push(AccountStatementRow {
                date,
                entry_number,
                narration: narration.unwrap_or_default(),
                debit,
                credit,
                balance: running_balance.clone(),
            });
        }

        Ok(AccountStatement {
            account_id,
            account_code: account.1,
            account_name: account.2,
            opening_balance,
            rows,
            closing_balance: running_balance,
        })
    }

    pub async fn income_statement(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<IncomeStatement> {
        let balances = self.fetch_balances_for_period(company_id, from_date, to_date).await?;

        let mut revenue = Vec::new();
        let mut total_revenue = BigDecimal::from(0);
        let mut cost_of_sales = Vec::new();
        let mut total_cost_of_sales = BigDecimal::from(0);
        let mut operating_expenses = Vec::new();
        let mut total_operating_expenses = BigDecimal::from(0);
        let mut finance_costs = Vec::new();
        let mut total_finance_costs = BigDecimal::from(0);
        let mut tax_expense = Vec::new();
        let mut total_tax_expense = BigDecimal::from(0);

        for row in &balances {
            if row.balance == BigDecimal::from(0) {
                continue;
            }

            let type_lower = row.account_type.to_lowercase();

            if type_lower == "revenue" {
                let amount = row.balance.clone().abs();
                total_revenue = &total_revenue + &amount;
                revenue.push(StatementLine {
                    account_code: row.code.clone(),
                    account_name: row.name.clone(),
                    amount,
                });
            } else if type_lower == "expense" {
                let amount = row.balance.clone().abs();

                if Self::is_cost_of_sales(row) {
                    total_cost_of_sales = &total_cost_of_sales + &amount;
                    cost_of_sales.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                } else if Self::is_tax_expense(row) {
                    total_tax_expense = &total_tax_expense + &amount;
                    tax_expense.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                } else if Self::is_finance_cost(row) {
                    total_finance_costs = &total_finance_costs + &amount;
                    finance_costs.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                } else {
                    total_operating_expenses = &total_operating_expenses + &amount;
                    operating_expenses.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                }
            }
        }

        let gross_profit = &total_revenue - &total_cost_of_sales;
        let operating_profit = &gross_profit - &total_operating_expenses;
        let net_profit_loss = &operating_profit - &total_finance_costs - &total_tax_expense;

        Ok(IncomeStatement {
            company_id,
            from_date: from_date.to_string(),
            to_date: to_date.to_string(),
            revenue,
            total_revenue,
            cost_of_sales,
            total_cost_of_sales,
            gross_profit,
            operating_expenses,
            total_operating_expenses,
            operating_profit,
            finance_costs,
            total_finance_costs,
            tax_expense,
            total_tax_expense,
            net_profit_loss,
        })
    }

    pub async fn balance_sheet(
        &self,
        company_id: Uuid,
        as_of_date: NaiveDate,
    ) -> Result<BalanceSheet> {
        let balances = self.fetch_balances_as_of(company_id, as_of_date).await?;

        let mut current_assets = Vec::new();
        let mut non_current_assets = Vec::new();
        let mut current_liabilities = Vec::new();
        let mut non_current_liabilities = Vec::new();
        let mut equity_lines = Vec::new();

        let mut current_asset_subtotal = BigDecimal::from(0);
        let mut non_current_asset_subtotal = BigDecimal::from(0);
        let mut current_liability_subtotal = BigDecimal::from(0);
        let mut non_current_liability_subtotal = BigDecimal::from(0);
        let mut equity_subtotal = BigDecimal::from(0);

        for row in &balances {
            if row.balance == BigDecimal::from(0) {
                continue;
            }

            let type_lower = row.account_type.to_lowercase();

            if type_lower == "asset" {
                let amount = row.balance.clone();
                if Self::is_current_asset(row) {
                    current_asset_subtotal = &current_asset_subtotal + &amount;
                    current_assets.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                } else {
                    non_current_asset_subtotal = &non_current_asset_subtotal + &amount;
                    non_current_assets.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                }
            } else if type_lower == "liability" {
                let amount = BigDecimal::from(0) - &row.balance;
                if Self::is_current_liability(row) {
                    current_liability_subtotal = &current_liability_subtotal + &amount;
                    current_liabilities.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                } else {
                    non_current_liability_subtotal = &non_current_liability_subtotal + &amount;
                    non_current_liabilities.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: row.name.clone(),
                        amount,
                    });
                }
            } else if type_lower == "equity" {
                let amount = BigDecimal::from(0) - &row.balance;
                equity_subtotal = &equity_subtotal + &amount;
                equity_lines.push(StatementLine {
                    account_code: row.code.clone(),
                    account_name: row.name.clone(),
                    amount,
                });
            }
        }

        let total_assets = &current_asset_subtotal + &non_current_asset_subtotal;
        let total_liabilities = &current_liability_subtotal + &non_current_liability_subtotal;
        let total_equity = equity_subtotal;

        let asset_sections = vec![
            BalanceSheetSection {
                category: "Non-Current Assets".to_string(),
                lines: non_current_assets,
                subtotal: non_current_asset_subtotal,
            },
            BalanceSheetSection {
                category: "Current Assets".to_string(),
                lines: current_assets,
                subtotal: current_asset_subtotal,
            },
        ];

        let liability_sections = vec![
            BalanceSheetSection {
                category: "Non-Current Liabilities".to_string(),
                lines: non_current_liabilities,
                subtotal: non_current_liability_subtotal,
            },
            BalanceSheetSection {
                category: "Current Liabilities".to_string(),
                lines: current_liabilities,
                subtotal: current_liability_subtotal,
            },
        ];

        let equity_section = vec![BalanceSheetSection {
            category: "Equity".to_string(),
            lines: equity_lines,
            subtotal: total_equity.clone(),
        }];

        Ok(BalanceSheet {
            company_id,
            as_of_date: as_of_date.to_string(),
            assets: asset_sections,
            is_balanced: total_assets == total_liabilities.clone() + total_equity.clone(),
            total_assets,
            liabilities: liability_sections,
            total_liabilities,
            equity: equity_section,
            total_equity,
        })
    }

    pub async fn cash_flow_statement(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        method: &str,
    ) -> Result<CashFlowStatement> {
        let income = self.income_statement(company_id, from_date, to_date).await?;
        let period_balances = self.fetch_balances_for_period(company_id, from_date, to_date).await?;

        let day_before = from_date - chrono::Duration::days(1);
        let opening_balances = self.fetch_balances_as_of(company_id, day_before).await?;
        let closing_balances = self.fetch_balances_as_of(company_id, to_date).await?;

        let mut operating_activities = Vec::new();
        let mut investing_activities = Vec::new();
        let mut financing_activities = Vec::new();
        let net_income = income.net_profit_loss.clone();

        if method == "indirect" {
            operating_activities.push(StatementLine {
                account_code: "NET".to_string(),
                account_name: "Net Profit/(Loss)".to_string(),
                amount: net_income.clone(),
            });

            let mut non_cash_total = BigDecimal::from(0);
            for row in &period_balances {
                if row.account_type.to_lowercase() == "expense"
                    && Self::is_depreciation(row)
                    && row.balance != BigDecimal::from(0)
                {
                    let amount = row.balance.clone().abs();
                    non_cash_total = &non_cash_total + &amount;
                    operating_activities.push(StatementLine {
                        account_code: row.code.clone(),
                        account_name: format!("Adjustment: {}", row.name),
                        amount,
                    });
                }
            }

            let mut opening_current_assets = BigDecimal::from(0);
            let mut closing_current_assets = BigDecimal::from(0);
            let mut opening_current_liabilities = BigDecimal::from(0);
            let mut closing_current_liabilities = BigDecimal::from(0);

            for row in &opening_balances {
                let tl = row.account_type.to_lowercase();
                if tl == "asset" && Self::is_current_asset(row) && !Self::is_cash_account(row) {
                    opening_current_assets = &opening_current_assets + &row.balance;
                } else if tl == "liability" && Self::is_current_liability(row) {
                    opening_current_liabilities =
                        &opening_current_liabilities + &(BigDecimal::from(0) - &row.balance);
                }
            }

            for row in &closing_balances {
                let tl = row.account_type.to_lowercase();
                if tl == "asset" && Self::is_current_asset(row) && !Self::is_cash_account(row) {
                    closing_current_assets = &closing_current_assets + &row.balance;
                } else if tl == "liability" && Self::is_current_liability(row) {
                    closing_current_liabilities =
                        &closing_current_liabilities + &(BigDecimal::from(0) - &row.balance);
                }
            }

            let change_in_current_assets = &closing_current_assets - &opening_current_assets;
            let change_in_current_liabilities =
                &closing_current_liabilities - &opening_current_liabilities;

            if change_in_current_assets != BigDecimal::from(0) {
                operating_activities.push(StatementLine {
                    account_code: "WCA".to_string(),
                    account_name: "(Increase)/Decrease in Working Capital Assets".to_string(),
                    amount: BigDecimal::from(0) - &change_in_current_assets,
                });
            }
            if change_in_current_liabilities != BigDecimal::from(0) {
                operating_activities.push(StatementLine {
                    account_code: "WCL".to_string(),
                    account_name: "Increase/(Decrease) in Working Capital Liabilities".to_string(),
                    amount: change_in_current_liabilities.clone(),
                });
            }

            let working_capital_impact =
                (BigDecimal::from(0) - &change_in_current_assets) + &change_in_current_liabilities;
            let net_operating = &net_income + &non_cash_total + &working_capital_impact;

            operating_activities.push(StatementLine {
                account_code: "NETOP".to_string(),
                account_name: "Net Cash from Operating Activities".to_string(),
                amount: net_operating.clone(),
            });

            let mut opening_nc_assets = BigDecimal::from(0);
            let mut closing_nc_assets = BigDecimal::from(0);
            for row in &opening_balances {
                if row.account_type.to_lowercase() == "asset" && !Self::is_current_asset(row) {
                    opening_nc_assets = &opening_nc_assets + &row.balance;
                }
            }
            for row in &closing_balances {
                if row.account_type.to_lowercase() == "asset" && !Self::is_current_asset(row) {
                    closing_nc_assets = &closing_nc_assets + &row.balance;
                }
            }
            let change_in_nc_assets = &closing_nc_assets - &opening_nc_assets;
            if change_in_nc_assets != BigDecimal::from(0) {
                investing_activities.push(StatementLine {
                    account_code: "INV".to_string(),
                    account_name: "Purchase of Non-Current Assets".to_string(),
                    amount: BigDecimal::from(0) - &change_in_nc_assets,
                });
            }
            let net_investing = BigDecimal::from(0) - &change_in_nc_assets;

            let mut opening_nc_liab = BigDecimal::from(0);
            let mut closing_nc_liab = BigDecimal::from(0);
            let mut opening_equity_total = BigDecimal::from(0);
            let mut closing_equity_total = BigDecimal::from(0);

            for row in &opening_balances {
                let tl = row.account_type.to_lowercase();
                if tl == "liability" && !Self::is_current_liability(row) {
                    opening_nc_liab = &opening_nc_liab + &(BigDecimal::from(0) - &row.balance);
                } else if tl == "equity" {
                    opening_equity_total =
                        &opening_equity_total + &(BigDecimal::from(0) - &row.balance);
                }
            }
            for row in &closing_balances {
                let tl = row.account_type.to_lowercase();
                if tl == "liability" && !Self::is_current_liability(row) {
                    closing_nc_liab = &closing_nc_liab + &(BigDecimal::from(0) - &row.balance);
                } else if tl == "equity" {
                    closing_equity_total =
                        &closing_equity_total + &(BigDecimal::from(0) - &row.balance);
                }
            }

            let change_in_nc_liab = &closing_nc_liab - &opening_nc_liab;
            if change_in_nc_liab != BigDecimal::from(0) {
                financing_activities.push(StatementLine {
                    account_code: "LOAN".to_string(),
                    account_name: "Loan Proceeds/(Repayments)".to_string(),
                    amount: change_in_nc_liab.clone(),
                });
            }

            let equity_change_excluding_income =
                &closing_equity_total - &opening_equity_total - &net_income;
            if equity_change_excluding_income > BigDecimal::from(0) {
                financing_activities.push(StatementLine {
                    account_code: "EQ".to_string(),
                    account_name: "Equity Contributions".to_string(),
                    amount: equity_change_excluding_income.clone(),
                });
            } else if equity_change_excluding_income < BigDecimal::from(0) {
                financing_activities.push(StatementLine {
                    account_code: "DIV".to_string(),
                    account_name: "Dividends Paid".to_string(),
                    amount: equity_change_excluding_income.clone(),
                });
            }

            let net_financing = change_in_nc_liab + equity_change_excluding_income;
            let net_change = &net_operating + &net_investing + &net_financing;

            let mut opening_cash = BigDecimal::from(0);
            let mut closing_cash_val = BigDecimal::from(0);
            for row in &opening_balances {
                if row.account_type.to_lowercase() == "asset" && Self::is_cash_account(row) {
                    opening_cash = &opening_cash + &row.balance;
                }
            }
            for row in &closing_balances {
                if row.account_type.to_lowercase() == "asset" && Self::is_cash_account(row) {
                    closing_cash_val = &closing_cash_val + &row.balance;
                }
            }

            return Ok(CashFlowStatement {
                company_id,
                from_date: from_date.to_string(),
                to_date: to_date.to_string(),
                method: method.to_string(),
                operating_activities,
                net_operating,
                investing_activities,
                net_investing,
                financing_activities,
                net_financing,
                net_change,
                opening_cash,
                closing_cash: closing_cash_val,
            });
        }

        let mut cash_from_customers = BigDecimal::from(0);
        let mut cash_paid_suppliers = BigDecimal::from(0);
        let mut cash_paid_expenses = BigDecimal::from(0);

        for row in &period_balances {
            if row.balance == BigDecimal::from(0) {
                continue;
            }
            let tl = row.account_type.to_lowercase();
            if tl == "revenue" {
                cash_from_customers = &cash_from_customers + &row.balance.abs();
            } else if tl == "expense" {
                if Self::is_cost_of_sales(row) {
                    cash_paid_suppliers = &cash_paid_suppliers + &row.balance.abs();
                } else if !Self::is_depreciation(row) {
                    cash_paid_expenses = &cash_paid_expenses + &row.balance.abs();
                }
            }
        }

        operating_activities.push(StatementLine {
            account_code: "REV".to_string(),
            account_name: "Cash Received from Customers".to_string(),
            amount: cash_from_customers.clone(),
        });
        operating_activities.push(StatementLine {
            account_code: "COS".to_string(),
            account_name: "Cash Paid to Suppliers".to_string(),
            amount: BigDecimal::from(0) - &cash_paid_suppliers,
        });
        operating_activities.push(StatementLine {
            account_code: "OPEX".to_string(),
            account_name: "Cash Paid for Operating Expenses".to_string(),
            amount: BigDecimal::from(0) - &cash_paid_expenses,
        });

        let net_operating = &cash_from_customers - &cash_paid_suppliers - &cash_paid_expenses;

        let mut opening_nc_assets = BigDecimal::from(0);
        let mut closing_nc_assets = BigDecimal::from(0);
        for row in &opening_balances {
            if row.account_type.to_lowercase() == "asset" && !Self::is_current_asset(row) {
                opening_nc_assets = &opening_nc_assets + &row.balance;
            }
        }
        for row in &closing_balances {
            if row.account_type.to_lowercase() == "asset" && !Self::is_current_asset(row) {
                closing_nc_assets = &closing_nc_assets + &row.balance;
            }
        }
        let change_in_nc_assets = &closing_nc_assets - &opening_nc_assets;
        if change_in_nc_assets != BigDecimal::from(0) {
            investing_activities.push(StatementLine {
                account_code: "INV".to_string(),
                account_name: "Purchase of Non-Current Assets".to_string(),
                amount: BigDecimal::from(0) - &change_in_nc_assets,
            });
        }
        let net_investing = BigDecimal::from(0) - &change_in_nc_assets;

        let mut opening_nc_liab = BigDecimal::from(0);
        let mut closing_nc_liab = BigDecimal::from(0);
        for row in &opening_balances {
            if row.account_type.to_lowercase() == "liability" && !Self::is_current_liability(row) {
                opening_nc_liab = &opening_nc_liab + &(BigDecimal::from(0) - &row.balance);
            }
        }
        for row in &closing_balances {
            if row.account_type.to_lowercase() == "liability" && !Self::is_current_liability(row) {
                closing_nc_liab = &closing_nc_liab + &(BigDecimal::from(0) - &row.balance);
            }
        }
        let change_in_nc_liab = &closing_nc_liab - &opening_nc_liab;
        if change_in_nc_liab != BigDecimal::from(0) {
            financing_activities.push(StatementLine {
                account_code: "LOAN".to_string(),
                account_name: "Loan Proceeds/(Repayments)".to_string(),
                amount: change_in_nc_liab.clone(),
            });
        }
        let net_financing = change_in_nc_liab;
        let net_change = &net_operating + &net_investing + &net_financing;

        let mut opening_cash = BigDecimal::from(0);
        let mut closing_cash_val = BigDecimal::from(0);
        for row in &opening_balances {
            if row.account_type.to_lowercase() == "asset" && Self::is_cash_account(row) {
                opening_cash = &opening_cash + &row.balance;
            }
        }
        for row in &closing_balances {
            if row.account_type.to_lowercase() == "asset" && Self::is_cash_account(row) {
                closing_cash_val = &closing_cash_val + &row.balance;
            }
        }

        Ok(CashFlowStatement {
            company_id,
            from_date: from_date.to_string(),
            to_date: to_date.to_string(),
            method: method.to_string(),
            operating_activities,
            net_operating,
            investing_activities,
            net_investing,
            financing_activities,
            net_financing,
            net_change,
            opening_cash,
            closing_cash: closing_cash_val,
        })
    }

    pub async fn retained_earnings(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<RetainedEarnings> {
        let income = self.income_statement(company_id, from_date, to_date).await?;

        let day_before = from_date - chrono::Duration::days(1);
        let opening_balances = self.fetch_balances_as_of(company_id, day_before).await?;
        let mut opening_balance = BigDecimal::from(0);

        for row in &opening_balances {
            if row.account_type.to_lowercase() == "equity" && Self::is_retained_earnings(row) {
                opening_balance = &opening_balance + &(BigDecimal::from(0) - &row.balance);
            }
        }

        let period_balances =
            self.fetch_balances_for_period(company_id, from_date, to_date).await?;
        let mut dividends_declared = BigDecimal::from(0);

        for row in &period_balances {
            if row.account_type.to_lowercase() == "equity" && Self::is_dividend_account(row) {
                dividends_declared = &dividends_declared + &row.balance.abs();
            }
        }

        let net_profit_loss = income.net_profit_loss.clone();
        let closing_balance = &opening_balance + &net_profit_loss - &dividends_declared;

        Ok(RetainedEarnings {
            company_id,
            from_date: from_date.to_string(),
            to_date: to_date.to_string(),
            opening_balance,
            net_profit_loss,
            dividends_declared,
            closing_balance,
        })
    }

    pub async fn ratio_analysis(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<RatioAnalysis> {
        let income = self.income_statement(company_id, from_date, to_date).await?;
        let bs = self.balance_sheet(company_id, to_date).await?;

        let zero = BigDecimal::from(0);

        let current_assets: BigDecimal = bs.assets
            .iter()
            .filter(|s| s.category == "Current Assets")
            .map(|s| s.subtotal.clone())
            .sum();
        let current_liab: BigDecimal = bs.liabilities
            .iter()
            .filter(|s| s.category == "Current Liabilities")
            .map(|s| s.subtotal.clone())
            .sum();

        let current_ratio = if current_liab != zero {
            &current_assets / &current_liab
        } else {
            zero.clone()
        };

        let quick_assets: BigDecimal = bs.assets
            .iter()
            .flat_map(|s| s.lines.iter())
            .filter(|l| {
                let n = l.account_name.to_lowercase();
                !n.contains("inventory") && !n.contains("prepayment")
            })
            .map(|l| l.amount.clone())
            .sum();
        let quick_ratio = if current_liab != zero {
            &quick_assets / &current_liab
        } else {
            zero.clone()
        };

        let cash_assets: BigDecimal = bs.assets
            .iter()
            .flat_map(|s| s.lines.iter())
            .filter(|l| {
                let n = l.account_name.to_lowercase();
                n.contains("cash") || n.contains("bank")
            })
            .map(|l| l.amount.clone())
            .sum();
        let cash_ratio = if current_liab != zero {
            &cash_assets / &current_liab
        } else {
            zero.clone()
        };

        let debt_to_equity = if bs.total_equity != zero {
            &bs.total_liabilities / &bs.total_equity
        } else {
            zero.clone()
        };

        let interest_expense: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN chart_of_accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND LOWER(a.account_type) = 'expense'
               AND (LOWER(a.name) LIKE '%interest%' OR LOWER(COALESCE(a.sub_type, '')) LIKE '%interest%')
               AND jh.status = 'posted'
               AND jh.date >= $2 AND jh.date <= $3"#,
        )
        .bind(company_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let interest_coverage = if interest_expense != zero {
            &income.net_profit_loss / &interest_expense
        } else {
            zero.clone()
        };

        let roe = if bs.total_equity != zero {
            &income.net_profit_loss / &bs.total_equity
        } else {
            zero.clone()
        };

        let roa = if bs.total_assets != zero {
            &income.net_profit_loss / &bs.total_assets
        } else {
            zero.clone()
        };

        let capital_employed = &bs.total_assets - &bs.total_liabilities;
        let roce = if capital_employed != zero {
            &income.net_profit_loss / &capital_employed
        } else {
            zero.clone()
        };

        let gross_margin = if income.total_revenue != zero {
            &income.gross_profit / &income.total_revenue
        } else {
            zero.clone()
        };

        let net_margin = if income.total_revenue != zero {
            &income.net_profit_loss / &income.total_revenue
        } else {
            zero.clone()
        };

        let asset_turnover = if bs.total_assets != zero {
            &income.total_revenue / &bs.total_assets
        } else {
            zero.clone()
        };

        let inventory: BigDecimal = bs.assets
            .iter()
            .flat_map(|s| s.lines.iter())
            .filter(|l| l.account_name.to_lowercase().contains("inventory"))
            .map(|l| l.amount.clone())
            .sum();
        let inventory_turnover = if inventory != zero {
            &income.total_revenue / &inventory
        } else {
            zero.clone()
        };

        let receivable: BigDecimal = bs.assets
            .iter()
            .flat_map(|s| s.lines.iter())
            .filter(|l| l.account_name.to_lowercase().contains("receivable"))
            .map(|l| l.amount.clone())
            .sum();
        let receivable_turnover = if receivable != zero {
            &income.total_revenue / &receivable
        } else {
            zero.clone()
        };

        Ok(RatioAnalysis {
            company_id,
            period_id: None,
            current_ratio,
            quick_ratio,
            cash_ratio,
            debt_to_equity,
            interest_coverage,
            roe,
            roa,
            roce,
            gross_margin,
            net_margin,
            asset_turnover,
            inventory_turnover,
            receivable_turnover,
        })
    }

    pub fn export_to_csv(&self, data: &[(Vec<String>, Vec<Vec<String>>)]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let mut writer = Writer::from_writer(&mut buffer);
            for (headers, rows) in data {
                writer.write_record(headers)?;
                for row in rows {
                    writer.write_record(row)?;
                }
                writer.write_record(vec![""])?;
            }
        }
        Ok(buffer)
    }

    pub fn export_to_excel(&self, data: &[(String, Vec<String>, Vec<Vec<String>>)]) -> Result<Vec<u8>> {
        let mut workbook = Workbook::new();
        for (sheet_name, headers, rows) in data {
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(sheet_name)?;
            for (col, header) in headers.iter().enumerate() {
                worksheet.write_string(0, col as u16, header)?;
            }
            for (row_idx, row) in rows.iter().enumerate() {
                for (col, cell) in row.iter().enumerate() {
                    worksheet.write_string((row_idx + 1) as u32, col as u16, cell)?;
                }
            }
        }
        let buffer = workbook.save_to_buffer()?;
        Ok(buffer.to_vec())
    }

    pub fn export_to_pdf(&self, title: &str, data: &[(Vec<String>, Vec<Vec<String>>)]) -> Result<Vec<u8>> {
        let mut content = String::new();
        content.push_str(&format!("{}\n{}\n\n", title, "=".repeat(title.len())));
        for (headers, rows) in data {
            let header_line = headers.join(" | ");
            content.push_str(&header_line);
            content.push_str(&format!("\n{}\n", "-".repeat(header_line.len())));
            for row in rows {
                content.push_str(&row.join(" | "));
                content.push('\n');
            }
            content.push_str("\n\n");
        }

        let escaped = content.replace('\n', "\\n").replace('(', "\\(").replace(')', "\\)");
        let simple_pdf = format!(
            "%PDF-1.0\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<</Font<</F1 4 0 R>>>>/Contents 5 0 R>>endobj\n4 0 obj<</Type/Font/Subtype/Type1/BaseFont/Courier>>endobj\n5 0 obj<</Length {}>>stream\nBT\n/F1 10 Tf\n12 750 Td\n{}ET\nendstream\nendobj\nxref\n0 6\n0000000000 65535 f \ntrailer<</Size 6/Root 1 0 R>>\nstartxref\n0\n%%EOF",
            escaped.len() + 30,
            escaped
        );

        Ok(simple_pdf.into_bytes())
    }
}
