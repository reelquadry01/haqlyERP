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
    IncomeStatement, RatioAnalysis, StatementLine, TrialBalance, TrialBalanceRow,
};
use crate::models::account::{Account, AccountType};

#[derive(Clone)]
pub struct ReportsService {
    pub pool: PgPool,
}

impl ReportsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn trial_balance(&self, company_id: Uuid, as_of_date: Option<NaiveDate>) -> Result<TrialBalance> {
        let accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut rows = Vec::new();
        let mut total_debit = BigDecimal::from(0);
        let mut total_credit = BigDecimal::from(0);

        for account in &accounts {
            let balance: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
                   FROM journal_lines jl
                   JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND ($3::date IS NULL OR jh.created_at::date <= $3)"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(as_of_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if balance == BigDecimal::from(0) {
                continue;
            }

            let (debit, credit) = if balance > BigDecimal::from(0) {
                (balance.clone(), BigDecimal::from(0))
            } else {
                (BigDecimal::from(0), balance.clone().abs())
            };

            total_debit = &total_debit + &debit;
            total_credit = &total_credit + &credit;

            rows.push(TrialBalanceRow {
                account_code: account.code.clone(),
                account_name: account.name.clone(),
                account_type: account.account_type.to_string(),
                debit,
                credit,
            });
        }

        Ok(TrialBalance {
            rows,
            total_debit,
            total_credit,
            is_balanced: total_debit == total_credit,
        })
    }

    pub async fn account_statement(
        &self,
        account_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<AccountStatement> {
        let account = sqlx::query_as::<_, Account>("SELECT * FROM chart_of_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_one(&self.pool)
            .await?;

        let opening_balance: BigDecimal = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               WHERE jl.account_id = $1 AND jh.status = 'posted' AND jh.created_at::date < $2"#,
        )
        .bind(account_id)
        .bind(from_date)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let line_rows = sqlx::query_as::<_, (String, String, String, BigDecimal, BigDecimal)>(
            r#"SELECT jh.created_at::text, jh.entry_number, COALESCE(jl.narration, jh.narration), jl.debit, jl.credit
               FROM journal_lines jl
               JOIN journal_headers jh ON jl.journal_header_id = jh.id
               WHERE jl.account_id = $1 AND jh.status = 'posted'
               AND jh.created_at::date >= $2 AND jh.created_at::date <= $3
               ORDER BY jh.created_at"#,
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
                narration,
                debit,
                credit,
                balance: running_balance.clone(),
            });
        }

        let closing_balance = running_balance;

        Ok(AccountStatement {
            account_id,
            account_code: account.code,
            account_name: account.name,
            opening_balance,
            rows,
            closing_balance,
        })
    }

    pub async fn income_statement(
        &self,
        company_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<IncomeStatement> {
        let revenue_accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND account_type = 'revenue' AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let expense_accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND account_type = 'expense' AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut revenue = Vec::new();
        let mut total_revenue = BigDecimal::from(0);

        for account in &revenue_accounts {
            let amount: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
                   FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND jh.created_at::date >= $3 AND jh.created_at::date <= $4"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(from_date)
            .bind(to_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if amount != BigDecimal::from(0) {
                total_revenue = &total_revenue + &amount;
                revenue.push(StatementLine {
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    amount,
                });
            }
        }

        let mut expenses = Vec::new();
        let mut total_expenses = BigDecimal::from(0);

        for account in &expense_accounts {
            let amount: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
                   FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND jh.created_at::date >= $3 AND jh.created_at::date <= $4"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(from_date)
            .bind(to_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if amount != BigDecimal::from(0) {
                total_expenses = &total_expenses + &amount;
                expenses.push(StatementLine {
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    amount,
                });
            }
        }

        let gross_profit = &total_revenue - &total_expenses;
        let net_income = gross_profit.clone();

        Ok(IncomeStatement {
            company_id,
            from_date: from_date.to_string(),
            to_date: to_date.to_string(),
            revenue,
            total_revenue,
            expenses,
            total_expenses,
            gross_profit,
            net_income,
        })
    }

    pub async fn balance_sheet(&self, company_id: Uuid, as_of_date: NaiveDate) -> Result<BalanceSheet> {
        let asset_accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND account_type = 'asset' AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let liability_accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND account_type = 'liability' AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let equity_accounts = sqlx::query_as::<_, Account>(
            "SELECT * FROM chart_of_accounts WHERE company_id = $1 AND account_type = 'equity' AND is_active = true ORDER BY code",
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut current_assets = Vec::new();
        let mut non_current_assets = Vec::new();
        let mut total_assets = BigDecimal::from(0);

        for account in &asset_accounts {
            let amount: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.debit - jl.credit), 0)
                   FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND jh.created_at::date <= $3"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(as_of_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if amount != BigDecimal::from(0) {
                total_assets = &total_assets + &amount;
                let line = StatementLine {
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    amount,
                };
                let code: i32 = account.code.parse().unwrap_or(0);
                if code < 1500 {
                    current_assets.push(line);
                } else {
                    non_current_assets.push(line);
                }
            }
        }

        let current_asset_subtotal: BigDecimal = current_assets.iter().map(|l| l.amount.clone()).sum();
        let non_current_asset_subtotal: BigDecimal = non_current_assets.iter().map(|l| l.amount.clone()).sum();

        let mut current_liabilities = Vec::new();
        let mut non_current_liabilities = Vec::new();
        let mut total_liabilities = BigDecimal::from(0);

        for account in &liability_accounts {
            let amount: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
                   FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND jh.created_at::date <= $3"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(as_of_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if amount != BigDecimal::from(0) {
                total_liabilities = &total_liabilities + &amount;
                let line = StatementLine {
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    amount,
                };
                let code: i32 = account.code.parse().unwrap_or(0);
                if code < 2500 {
                    current_liabilities.push(line);
                } else {
                    non_current_liabilities.push(line);
                }
            }
        }

        let current_liability_subtotal: BigDecimal = current_liabilities.iter().map(|l| l.amount.clone()).sum();
        let non_current_liability_subtotal: BigDecimal = non_current_liabilities.iter().map(|l| l.amount.clone()).sum();

        let mut equity = Vec::new();
        let mut total_equity = BigDecimal::from(0);

        for account in &equity_accounts {
            let amount: BigDecimal = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(jl.credit - jl.debit), 0)
                   FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
                   WHERE jl.account_id = $1 AND jh.company_id = $2 AND jh.status = 'posted'
                   AND jh.created_at::date <= $3"#,
            )
            .bind(account.id)
            .bind(company_id)
            .bind(as_of_date)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(BigDecimal::from(0));

            if amount != BigDecimal::from(0) {
                total_equity = &total_equity + &amount;
                equity.push(StatementLine {
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    amount,
                });
            }
        }

        let asset_sections = vec![
            BalanceSheetSection {
                category: "Current Assets".to_string(),
                lines: current_assets,
                subtotal: current_asset_subtotal,
            },
            BalanceSheetSection {
                category: "Non-Current Assets".to_string(),
                lines: non_current_assets,
                subtotal: non_current_asset_subtotal,
            },
        ];

        let liability_sections = vec![
            BalanceSheetSection {
                category: "Current Liabilities".to_string(),
                lines: current_liabilities,
                subtotal: current_liability_subtotal,
            },
            BalanceSheetSection {
                category: "Non-Current Liabilities".to_string(),
                lines: non_current_liabilities,
                subtotal: non_current_liability_subtotal,
            },
        ];

        let equity_section = vec![BalanceSheetSection {
            category: "Equity".to_string(),
            lines: equity,
            subtotal: total_equity.clone(),
        }];

        Ok(BalanceSheet {
            company_id,
            as_of_date: as_of_date.to_string(),
            assets: asset_sections,
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

        let operating_activities = match method {
            "indirect" => {
                let mut activities = Vec::new();
                activities.push(StatementLine {
                    account_code: "NET".to_string(),
                    account_name: "Net Income".to_string(),
                    amount: income.net_income.clone(),
                });
                activities
            }
            _ => {
                let mut activities = Vec::new();
                activities.push(StatementLine {
                    account_code: "REV".to_string(),
                    account_name: "Cash from Customers".to_string(),
                    amount: income.total_revenue.clone(),
                });
                activities.push(StatementLine {
                    account_code: "EXP".to_string(),
                    account_name: "Cash paid to Suppliers".to_string(),
                    amount: income.total_expenses.clone(),
                });
                activities
            }
        };

        let net_operating: BigDecimal = operating_activities.iter().map(|l| l.amount.clone()).sum();

        let investing_activities = vec![StatementLine {
            account_code: "INV".to_string(),
            account_name: "Cash used in investing".to_string(),
            amount: BigDecimal::from(0),
        }];
        let net_investing = BigDecimal::from(0);

        let financing_activities = vec![StatementLine {
            account_code: "FIN".to_string(),
            account_name: "Cash from financing".to_string(),
            amount: BigDecimal::from(0),
        }];
        let net_financing = BigDecimal::from(0);

        let net_change = &net_operating + &net_investing + &net_financing;

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
            opening_cash: BigDecimal::from(0),
            closing_cash: net_change.clone(),
        })
    }

    pub async fn ratio_analysis(&self, company_id: Uuid, from_date: NaiveDate, to_date: NaiveDate) -> Result<RatioAnalysis> {
        let income = self.income_statement(company_id, from_date, to_date).await?;
        let bs = self.balance_sheet(company_id, to_date).await?;

        let zero = BigDecimal::from(0);
        let one = BigDecimal::from(1);

        let current_ratio = if bs.total_liabilities != zero {
            &bs.total_assets / &bs.total_liabilities
        } else {
            zero.clone()
        };

        let quick_ratio = if bs.total_liabilities != zero {
            let quick_assets = bs.assets
                .iter()
                .flat_map(|s| s.lines.iter())
                .filter(|l| !l.account_name.to_lowercase().contains("inventory"))
                .map(|l| l.amount.clone())
                .sum::<BigDecimal>();
            quick_assets / &bs.total_liabilities
        } else {
            zero.clone()
        };

        let cash_ratio = if bs.total_liabilities != zero {
            let cash_assets: BigDecimal = bs.assets
                .iter()
                .flat_map(|s| s.lines.iter())
                .filter(|l| l.account_name.to_lowercase().contains("cash") || l.account_name.to_lowercase().contains("bank"))
                .map(|l| l.amount.clone())
                .sum();
            cash_assets / &bs.total_liabilities
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
               FROM journal_lines jl JOIN journal_headers jh ON jl.journal_header_id = jh.id
               JOIN chart_of_accounts a ON jl.account_id = a.id
               WHERE jh.company_id = $1 AND a.account_type = 'expense'
               AND (a.name ILIKE '%interest%' OR a.code LIKE '5205%')
               AND jh.status = 'posted'
               AND jh.created_at::date >= $2 AND jh.created_at::date <= $3"#,
        )
        .bind(company_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(BigDecimal::from(0));

        let interest_coverage = if interest_expense != zero {
            &income.net_income / &interest_expense
        } else {
            zero.clone()
        };

        let roe = if bs.total_equity != zero {
            &income.net_income / &bs.total_equity
        } else {
            zero.clone()
        };

        let roa = if bs.total_assets != zero {
            &income.net_income / &bs.total_assets
        } else {
            zero.clone()
        };

        let capital_employed = &bs.total_assets - &bs.total_liabilities;
        let roce = if capital_employed != zero {
            &income.net_income / &capital_employed
        } else {
            zero.clone()
        };

        let gross_margin = if income.total_revenue != zero {
            &income.gross_profit / &income.total_revenue
        } else {
            zero.clone()
        };

        let net_margin = if income.total_revenue != zero {
            &income.net_income / &income.total_revenue
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

        let simple_pdf = format!(
            "%PDF-1.0\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<</Font<</F1 4 0 R>>>>/Contents 5 0 R>>endobj\n4 0 obj<</Type/Font/Subtype/Type1/BaseFont/Courier>>endobj\n5 0 obj<</Length {}>>stream\nBT\n/F1 10 Tf\n12 750 Td\n{}ET\nendstream\nendobj\nxref\n0 6\n0000000000 65535 f \ntrailer<</Size 6/Root 1 0 R>>\nstartxref\n0\n%%EOF",
            content.len() + 30,
            content.replace('\n', "\\n").replace('(', "\\(").replace(')', "\\)")
        );

        Ok(simple_pdf.into_bytes())
    }
}
