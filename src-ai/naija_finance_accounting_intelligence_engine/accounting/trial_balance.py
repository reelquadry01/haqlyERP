# Author: Quadri Atharu
"""Trial Balance generation with unbalanced entry detection."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class TrialBalanceEngine:
    """Trial Balance generation engine with unbalanced detection and drill-down."""

    def __init__(self) -> None:
        self._ledger_engine: Any = None

    def set_ledger_engine(self, ledger_engine: Any) -> None:
        """Inject the ledger engine for balance computation."""
        self._ledger_engine = ledger_engine

    def generate_trial_balance(
        self,
        company_id: str,
        period_end: str,
        period_start: Optional[str] = None,
        include_zero_balances: bool = False,
    ) -> Dict[str, Any]:
        """Generate a trial balance from the ledger."""
        if self._ledger_engine is None:
            raise AccountingError("Ledger engine not set; cannot generate trial balance")

        all_balances = self._ledger_engine.get_all_balances()

        lines: List[Dict[str, Any]] = []
        total_debit = 0.0
        total_credit = 0.0

        for bal in all_balances:
            closing = bal.get("closing_balance", 0)
            if not include_zero_balances and closing == 0 and bal.get("total_debit", 0) == 0 and bal.get("total_credit", 0) == 0:
                continue

            debit_col = 0.0
            credit_col = 0.0

            if closing > 0:
                if bal["account_type"] in ("ASSET", "EXPENSE"):
                    debit_col = closing
                else:
                    credit_col = closing
            elif closing < 0:
                if bal["account_type"] in ("ASSET", "EXPENSE"):
                    credit_col = abs(closing)
                else:
                    debit_col = abs(closing)

            lines.append({
                "account_code": bal["account_code"],
                "account_name": bal["account_name"],
                "account_type": bal["account_type"],
                "debit": _money_round(debit_col),
                "credit": _money_round(credit_col),
            })

            total_debit += debit_col
            total_credit += credit_col

        total_debit = _money_round(total_debit)
        total_credit = _money_round(total_credit)
        difference = _money_round(total_debit - total_credit)
        is_balanced = abs(difference) < Decimal('0.01')

        tb: Dict[str, Any] = {
            "company_id": company_id,
            "report_type": "trial_balance",
            "period_start": period_start,
            "period_end": period_end,
            "currency": "NGN",
            "lines": lines,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "difference": difference,
            "is_balanced": is_balanced,
            "account_count": len(lines),
            "generated_at": datetime.now().isoformat(),
        }

        if not is_balanced:
            tb["unbalanced_analysis"] = self._analyze_unbalanced(tb)

        logger.info("trial_balance_generated", company_id=company_id, is_balanced=is_balanced, difference=difference, account_count=len(lines))
        return tb

    def detect_unbalanced_entries(self, journal_entries: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Scan a list of journal entries for unbalanced ones."""
        unbalanced: List[Dict[str, Any]] = []

        for entry in journal_entries:
            total_debit = round(sum(l.get("debit", 0) for l in entry.get("lines", [])), 2)
            total_credit = round(sum(l.get("credit", 0) for l in entry.get("lines", [])), 2)
            diff = _money_round(total_debit - total_credit)
            if abs(diff) > Decimal('0.01'):
                unbalanced.append({
                    "entry_id": entry.get("id"),
                    "entry_number": entry.get("entry_number", ""),
                    "total_debit": total_debit,
                    "total_credit": total_credit,
                    "difference": diff,
                    "description": entry.get("description", ""),
                })

        return {
            "total_entries_scanned": len(journal_entries),
            "unbalanced_count": len(unbalanced),
            "unbalanced_entries": unbalanced,
            "all_balanced": len(unbalanced) == 0,
        }

    def generate_adjusted_trial_balance(
        self,
        company_id: str,
        period_end: str,
        period_start: Optional[str] = None,
        adjusting_entries: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        """Generate an adjusted trial balance incorporating adjusting entries."""
        unadjusted = self.generate_trial_balance(company_id, period_end, period_start, include_zero_balances=False)

        if adjusting_entries:
            for adj in adjusting_entries:
                for line in adj.get("lines", []):
                    account_code = str(line.get("account_code", ""))
                    debit = float(line.get("debit", 0))
                    credit = float(line.get("credit", 0))

                    for tb_line in unadjusted["lines"]:
                        if tb_line["account_code"] == account_code:
                            tb_line["debit"] = _money_round(tb_line["debit"] + debit)
                            tb_line["credit"] = _money_round(tb_line["credit"] + credit)
                            break
                    else:
                        acct_type = "EXPENSE"
                        if account_code and account_code[0] in ("1", "2", "3", "4"):
                            acct_map = {"1": "ASSET", "2": "LIABILITY", "3": "EQUITY", "4": "REVENUE"}
                            acct_type = acct_map.get(account_code[0], "EXPENSE")
                        unadjusted["lines"].append({
                            "account_code": account_code,
                            "account_name": f"Account {account_code}",
                            "account_type": acct_type,
                            "debit": _money_round(debit),
                            "credit": _money_round(credit),
                        })

            unadjusted["total_debit"] = _money_round(sum(l["debit"] for l in unadjusted["lines"]))
            unadjusted["total_credit"] = _money_round(sum(l["credit"] for l in unadjusted["lines"]))
            unadjusted["difference"] = _money_round(unadjusted["total_debit"] - unadjusted["total_credit"])
            unadjusted["is_balanced"] = abs(unadjusted["difference"]) < Decimal('0.01')
            unadjusted["adjusting_entries_applied"] = len(adjusting_entries)

        unadjusted["report_type"] = "adjusted_trial_balance"
        return unadjusted

    def generate_post_closing_trial_balance(self, company_id: str, period_end: str, period_start: Optional[str] = None) -> Dict[str, Any]:
        """Generate a post-closing trial balance (only permanent accounts)."""
        full_tb = self.generate_trial_balance(company_id, period_end, period_start, include_zero_balances=False)

        permanent_types = {"ASSET", "LIABILITY", "EQUITY"}
        closing_lines = [l for l in full_tb["lines"] if l["account_type"] in permanent_types]

        total_debit = _money_round(sum(l["debit"] for l in closing_lines))
        total_credit = _money_round(sum(l["credit"] for l in closing_lines))

        return {
            "company_id": company_id,
            "report_type": "post_closing_trial_balance",
            "period_start": period_start,
            "period_end": period_end,
            "currency": "NGN",
            "lines": closing_lines,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "difference": _money_round(total_debit - total_credit),
            "is_balanced": abs(total_debit - total_credit) < Decimal('0.01'),
            "account_count": len(closing_lines),
            "generated_at": datetime.now().isoformat(),
        }

    def _analyze_unbalanced(self, tb: Dict[str, Any]) -> Dict[str, Any]:
        """Provide analysis on why a trial balance may be unbalanced."""
        diff = tb.get("difference", 0)
        analysis: Dict[str, Any] = {
            "difference": diff,
            "possible_causes": [],
        }

        if abs(diff) > 0:
            if diff % 2 == 0:
                analysis["possible_causes"].append("Possible entry posted to wrong side (debit instead of credit or vice versa)")
            if abs(diff) >= 10:
                analysis["possible_causes"].append("Omitted or duplicated entry")
            if _money_round(diff * 9) == 0 or _money_round(diff / 9) == round(diff / 9):
                analysis["possible_causes"].append("Possible transposition error")

            analysis["suggested_suspense_account"] = "9900"
            analysis["suggested_suspense_entry"] = {
                "account_code": "9900",
                "debit": abs(diff) if diff < 0 else 0.0,
                "credit": abs(diff) if diff > 0 else 0.0,
                "description": f"Suspense entry for trial balance difference of {diff}",
            }

        return analysis

    def health_check(self) -> bool:
        return True
