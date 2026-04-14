# Author: Quadri Atharu
"""Ledger Agent — posts to ledger, computes balances, and reconciles sub-ledgers."""

from __future__ import annotations

from typing import Any, Dict, List

from ..accounting import LedgerEngine
from .base_agent import BaseAgent


class LedgerAgent(BaseAgent):
    """Agent responsible for general ledger operations."""

    agent_name = "ledger_agent"

    def __init__(self, ledger_engine: LedgerEngine | None = None) -> None:
        super().__init__()
        self._ledger_engine = ledger_engine or LedgerEngine()
        self.register_skill("post_to_ledger", self._post_to_ledger)
        self.register_skill("compute_balances", self._compute_balances)
        self.register_skill("reconcile", self._reconcile)

    def _post_to_ledger(self, data: Dict[str, Any]) -> Dict[str, Any]:
        journal_entry = data.get("journal_entry")
        if journal_entry is None:
            return {"success": False, "error": "No journal_entry provided in data"}
        result = self._ledger_engine.post_journal_entry(journal_entry)
        return {
            "success": True,
            "journal_entry_id": result["journal_entry_id"],
            "posted_to_gl": result["posted_to_gl"],
            "posted_to_sub_ledger": result["posted_to_sub_ledger"],
            "posted_at": result["posted_at"],
        }

    def _compute_balances(self, data: Dict[str, Any]) -> Dict[str, Any]:
        account_code = data.get("account_code")
        account_type_filter = data.get("account_type_filter")
        period_start = data.get("period_start")
        period_end = data.get("period_end")

        if account_code:
            balance = self._ledger_engine.compute_account_balance(
                account_code, period_start=period_start, period_end=period_end
            )
            return {
                "success": True,
                "mode": "single",
                "balance": balance,
            }

        balances = self._ledger_engine.get_all_balances(account_type_filter=account_type_filter)
        total_debit = round(sum(b["total_debit"] for b in balances), 2)
        total_credit = round(sum(b["total_credit"] for b in balances), 2)
        return {
            "success": True,
            "mode": "all",
            "balances": balances,
            "account_count": len(balances),
            "total_debit": total_debit,
            "total_credit": total_credit,
            "trial_balance_ok": abs(total_debit - total_credit) < 0.01,
        }

    def _reconcile(self, data: Dict[str, Any]) -> Dict[str, Any]:
        control_account_code = data.get("control_account_code", "")
        if not control_account_code:
            account_codes = data.get("account_codes", [])
            if not account_codes:
                return {"success": False, "error": "No control_account_code or account_codes provided"}
            results: List[Dict[str, Any]] = []
            all_reconciled = True
            for code in account_codes:
                rec = self._ledger_engine.reconcile_sub_ledger(code)
                results.append(rec)
                if not rec["reconciled"]:
                    all_reconciled = False
            return {
                "success": True,
                "mode": "batch",
                "reconciliations": results,
                "all_reconciled": all_reconciled,
                "total_accounts": len(results),
                "reconciled_count": sum(1 for r in results if r["reconciled"]),
            }

        rec = self._ledger_engine.reconcile_sub_ledger(control_account_code)
        return {
            "success": True,
            "mode": "single",
            "reconciliation": rec,
            "reconciled": rec["reconciled"],
            "difference": rec["difference"],
        }
