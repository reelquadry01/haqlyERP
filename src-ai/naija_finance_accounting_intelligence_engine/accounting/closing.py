# Author: Quadri Atharu
"""Month-end and year-end closing procedures."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger

logger = get_logger(__name__)

TEMPORARY_ACCOUNT_TYPES = {"REVENUE", "EXPENSE"}
PERMANENT_ACCOUNT_TYPES = {"ASSET", "LIABILITY", "EQUITY"}


class ClosingEngine:
    """Month-end and year-end closing engine for Nigerian entities."""

    def close_month(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Execute month-end closing procedures."""
        company_id = data.get("company_id", "")
        period_end = data.get("period_end", datetime.now().isoformat())
        period_start = data.get("period_start")
        income_summary_account = data.get("income_summary_account", "3500")
        retained_earnings_account = data.get("retained_earnings_account", "3100")

        ledger_engine = data.get("ledger_engine")
        if ledger_engine is None:
            raise AccountingError("Ledger engine is required for month-end closing")

        trial_balance = ledger_engine.get_all_balances()
        closing_entries: List[Dict[str, Any]] = []

        revenue_lines, total_revenue = self._close_nominal_accounts(trial_balance, "REVENUE", income_summary_account)
        if revenue_lines:
            closing_entries.append({
                "type": "close_revenue",
                "description": f"Close revenue accounts to income summary — period ending {period_end}",
                "lines": revenue_lines,
                "total": total_revenue,
            })

        expense_lines, total_expense = self._close_nominal_accounts(trial_balance, "EXPENSE", income_summary_account)
        if expense_lines:
            closing_entries.append({
                "type": "close_expenses",
                "description": f"Close expense accounts to income summary — period ending {period_end}",
                "lines": expense_lines,
                "total": total_expense,
            })

        net_income = round(total_revenue - total_expense, 2)

        if abs(net_income) > 0.01:
            close_is_lines: List[Dict[str, Any]] = []
            if net_income > 0:
                close_is_lines = [
                    {"account_code": income_summary_account, "description": "Close income summary — net income", "debit": round(net_income, 2), "credit": 0.0},
                    {"account_code": retained_earnings_account, "description": "Transfer net income to retained earnings", "debit": 0.0, "credit": round(net_income, 2)},
                ]
            else:
                close_is_lines = [
                    {"account_code": retained_earnings_account, "description": "Transfer net loss to retained earnings", "debit": round(abs(net_income), 2), "credit": 0.0},
                    {"account_code": income_summary_account, "description": "Close income summary — net loss", "debit": 0.0, "credit": round(abs(net_income), 2)},
                ]
            closing_entries.append({
                "type": "close_income_summary",
                "description": f"Close income summary to retained earnings",
                "lines": close_is_lines,
                "total": abs(net_income),
            })

        dividend_lines = self._process_dividends(data, retained_earnings_account)
        if dividend_lines:
            closing_entries.append({
                "type": "close_dividends",
                "description": "Close dividends to retained earnings",
                "lines": dividend_lines,
                "total": data.get("dividends_declared", 0),
            })

        result: Dict[str, Any] = {
            "company_id": company_id,
            "closing_type": "month_end",
            "period_end": period_end,
            "period_start": period_start,
            "total_revenue_closed": total_revenue,
            "total_expenses_closed": total_expense,
            "net_income": net_income,
            "closing_entries": closing_entries,
            "entry_count": len(closing_entries),
            "status": "closed",
            "closed_at": datetime.now().isoformat(),
        }
        logger.info("month_end_closed", company_id=company_id, period_end=period_end, net_income=net_income)
        return result

    def close_year(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Execute year-end closing procedures with full Nigerian compliance."""
        company_id = data.get("company_id", "")
        fiscal_year_end = data.get("fiscal_year_end", datetime.now().isoformat())
        retained_earnings_account = data.get("retained_earnings_account", "3100")
        income_summary_account = data.get("income_summary_account", "3500")
        dividend_account = data.get("dividend_account", "3300")
        appropriation_account = data.get("appropriation_account", "3400")

        ledger_engine = data.get("ledger_engine")
        if ledger_engine is None:
            raise AccountingError("Ledger engine is required for year-end closing")

        all_balances = ledger_engine.get_all_balances()

        revenue_total = 0.0
        expense_total = 0.0
        closing_entries: List[Dict[str, Any]] = []

        for bal in all_balances:
            if bal.get("account_type") == "REVENUE":
                closing_balance = bal.get("closing_balance", 0)
                if abs(closing_balance) > 0.01:
                    lines = [
                        {"account_code": bal["account_code"], "description": f"Close {bal['account_name']}", "debit": round(abs(closing_balance), 2), "credit": 0.0},
                        {"account_code": income_summary_account, "description": f"Transfer {bal['account_name']} to income summary", "debit": 0.0, "credit": round(abs(closing_balance), 2)},
                    ]
                    closing_entries.append({"type": "close_revenue", "account_code": bal["account_code"], "account_name": bal["account_name"], "amount": closing_balance, "lines": lines})
                    revenue_total += closing_balance

            elif bal.get("account_type") == "EXPENSE":
                closing_balance = bal.get("closing_balance", 0)
                if abs(closing_balance) > 0.01:
                    lines = [
                        {"account_code": income_summary_account, "description": f"Transfer {bal['account_name']} to income summary", "debit": round(abs(closing_balance), 2), "credit": 0.0},
                        {"account_code": bal["account_code"], "description": f"Close {bal['account_name']}", "debit": 0.0, "credit": round(abs(closing_balance), 2)},
                    ]
                    closing_entries.append({"type": "close_expense", "account_code": bal["account_code"], "account_name": bal["account_name"], "amount": closing_balance, "lines": lines})
                    expense_total += closing_balance

        net_income = round(revenue_total - expense_total, 2)

        tax_provision = round(float(data.get("tax_provision", 0)), 2)
        bonus_provision = round(float(data.get("bonus_provision", 0)), 2)
        dividend_declared = round(float(data.get("dividends_declared", 0)), 2)
        transfer_to_reserve = round(float(data.get("transfer_to_reserve", 0)), 2)

        appropriation_lines: List[Dict[str, Any]] = []

        if tax_provision > 0:
            appropriation_lines.append({"account_code": appropriation_account, "description": "Tax provision for the year", "debit": tax_provision, "credit": 0.0})
            appropriation_lines.append({"account_code": "2320", "description": "CIT provision", "debit": 0.0, "credit": tax_provision})
            closing_entries.append({"type": "tax_provision", "amount": tax_provision, "lines": appropriation_lines[-2:]})

        if bonus_provision > 0:
            appropriation_lines.append({"account_code": appropriation_account, "description": "Staff bonus provision", "debit": bonus_provision, "credit": 0.0})
            appropriation_lines.append({"account_code": "2900", "description": "Bonus provision", "debit": 0.0, "credit": bonus_provision})
            closing_entries.append({"type": "bonus_provision", "amount": bonus_provision, "lines": appropriation_lines[-2:]})

        retained_after_appropriations = round(net_income - tax_provision - bonus_provision - dividend_declared - transfer_to_reserve, 2)

        is_close_lines: List[Dict[str, Any]] = []
        if net_income > 0:
            is_close_lines.append({"account_code": income_summary_account, "description": "Close income summary", "debit": round(net_income, 2), "credit": 0.0})
            is_close_lines.append({"account_code": retained_earnings_account, "description": "Transfer to retained earnings", "debit": 0.0, "credit": round(net_income, 2)})
        elif net_income < 0:
            is_close_lines.append({"account_code": retained_earnings_account, "description": "Absorb net loss", "debit": round(abs(net_income), 2), "credit": 0.0})
            is_close_lines.append({"account_code": income_summary_account, "description": "Close income summary (loss)", "debit": 0.0, "credit": round(abs(net_income), 2)})

        if is_close_lines:
            closing_entries.append({"type": "close_income_summary", "amount": abs(net_income), "lines": is_close_lines})

        if dividend_declared > 0:
            div_lines = [
                {"account_code": retained_earnings_account, "description": "Dividends declared", "debit": dividend_declared, "credit": 0.0},
                {"account_code": dividend_account, "description": "Dividends payable", "debit": 0.0, "credit": dividend_declared},
            ]
            closing_entries.append({"type": "dividends_declared", "amount": dividend_declared, "lines": div_lines})

        if transfer_to_reserve > 0:
            reserve_lines = [
                {"account_code": retained_earnings_account, "description": "Transfer to general reserve", "debit": transfer_to_reserve, "credit": 0.0},
                {"account_code": "3200", "description": "General reserve", "debit": 0.0, "credit": transfer_to_reserve},
            ]
            closing_entries.append({"type": "transfer_to_reserve", "amount": transfer_to_reserve, "lines": reserve_lines})

        result: Dict[str, Any] = {
            "company_id": company_id,
            "closing_type": "year_end",
            "fiscal_year_end": fiscal_year_end,
            "revenue_total": round(revenue_total, 2),
            "expense_total": round(expense_total, 2),
            "net_income_before_appropriations": net_income,
            "appropriations": {
                "tax_provision": tax_provision,
                "bonus_provision": bonus_provision,
                "dividends_declared": dividend_declared,
                "transfer_to_reserve": transfer_to_reserve,
            },
            "retained_earnings_after": retained_after_appropriations,
            "closing_entries": closing_entries,
            "entry_count": len(closing_entries),
            "status": "closed",
            "closed_at": datetime.now().isoformat(),
        }
        logger.info("year_end_closed", company_id=company_id, net_income=net_income, retained_after=retained_after_appropriations)
        return result

    def reopen_period(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Reopen a closed period by reversing closing entries."""
        company_id = data.get("company_id", "")
        period = data.get("period", "")
        closing_entries = data.get("closing_entries", [])

        if not closing_entries:
            raise AccountingError("No closing entries provided for reversal")

        reversal_entries: List[Dict[str, Any]] = []
        for entry in closing_entries:
            reversed_lines: List[Dict[str, Any]] = []
            for line in entry.get("lines", []):
                reversed_lines.append({
                    "account_code": line["account_code"],
                    "description": f"Reversal: {line.get('description', '')}",
                    "debit": line.get("credit", 0),
                    "credit": line.get("debit", 0),
                })
            reversal_entries.append({
                "type": f"reverse_{entry.get('type', 'unknown')}",
                "description": f"Reversal of period close — {period}",
                "lines": reversed_lines,
            })

        result = {
            "company_id": company_id,
            "period": period,
            "status": "reopened",
            "reversal_entries": reversal_entries,
            "entry_count": len(reversal_entries),
            "reopened_at": datetime.now().isoformat(),
        }
        logger.info("period_reopened", company_id=company_id, period=period)
        return result

    def validate_period_ready_for_close(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate that a period is ready for closing."""
        checks: Dict[str, Any] = {
            "all_entries_posted": True,
            "no_unbalanced_entries": True,
            "adjustments_complete": True,
            "reconciliation_complete": True,
            "issues": [],
        }

        pending_entries = data.get("pending_entries", [])
        unbalanced = data.get("unbalanced_entries", [])
        open_adjustments = data.get("open_adjustments", [])
        unreconciled = data.get("unreconciled_items", [])

        if pending_entries:
            checks["all_entries_posted"] = False
            checks["issues"].append(f"{len(pending_entries)} unposted journal entries")

        if unbalanced:
            checks["no_unbalanced_entries"] = False
            checks["issues"].append(f"{len(unbalanced)} unbalanced journal entries")

        if open_adjustments:
            checks["adjustments_complete"] = False
            checks["issues"].append(f"{len(open_adjustments)} open adjusting entries")

        if unreconciled:
            checks["reconciliation_complete"] = False
            checks["issues"].append(f"{len(unreconciled)} unreconciled items")

        checks["ready_for_close"] = all([
            checks["all_entries_posted"],
            checks["no_unbalanced_entries"],
            checks["adjustments_complete"],
            checks["reconciliation_complete"],
        ])

        return checks

    def _close_nominal_accounts(self, trial_balance: List[Dict[str, Any]], account_type: str, income_summary_account: str) -> tuple:
        """Close nominal (temporary) accounts to income summary."""
        lines: List[Dict[str, Any]] = []
        total = 0.0

        for bal in trial_balance:
            if bal.get("account_type") != account_type:
                continue
            closing_balance = bal.get("closing_balance", 0)
            if abs(closing_balance) < 0.01:
                continue

            if account_type == "REVENUE":
                lines.append({"account_code": bal["account_code"], "description": f"Close {bal['account_name']}", "debit": round(abs(closing_balance), 2), "credit": 0.0})
                total += closing_balance
            elif account_type == "EXPENSE":
                lines.append({"account_code": bal["account_code"], "description": f"Close {bal['account_name']}", "debit": 0.0, "credit": round(abs(closing_balance), 2)})
                total += closing_balance

        if lines:
            if account_type == "REVENUE":
                lines.append({"account_code": income_summary_account, "description": "Transfer revenue to income summary", "debit": 0.0, "credit": round(total, 2)})
            elif account_type == "EXPENSE":
                lines.append({"account_code": income_summary_account, "description": "Transfer expenses to income summary", "debit": round(total, 2), "credit": 0.0})

        return lines, round(total, 2)

    def _process_dividends(self, data: Dict[str, Any], retained_earnings_account: str) -> List[Dict[str, Any]]:
        """Process dividend declarations as part of closing."""
        dividends = float(data.get("dividends_declared", 0))
        if dividends <= 0:
            return []
        return [
            {"account_code": retained_earnings_account, "description": "Dividends declared", "debit": round(dividends, 2), "credit": 0.0},
            {"account_code": "3300", "description": "Dividends payable", "debit": 0.0, "credit": round(dividends, 2)},
        ]

    def health_check(self) -> bool:
        return True
