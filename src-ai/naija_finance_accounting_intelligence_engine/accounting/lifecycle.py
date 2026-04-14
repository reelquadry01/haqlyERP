# Author: Quadri Atharu
"""Full accounting lifecycle orchestrator."""

from __future__ import annotations

from typing import Any, Dict, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from .adjustments import AdjustmentEngine
from .closing import ClosingEngine
from .journal_engine import JournalEngine
from .ledger_engine import LedgerEngine
from .transaction_recognition import TransactionRecognition
from .trial_balance import TrialBalanceEngine

logger = get_logger(__name__)


class AccountingLifecycle:
    """Orchestrates the full accounting lifecycle from transaction to close."""

    def __init__(self) -> None:
        self.recognition = TransactionRecognition()
        self.journal = JournalEngine()
        self.ledger = LedgerEngine()
        self.trial_balance = TrialBalanceEngine()
        self.adjustments = AdjustmentEngine()
        self.closing = ClosingEngine()

        self.trial_balance.set_ledger_engine(self.ledger)

    async def run_full_lifecycle(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Run the complete accounting lifecycle for a single transaction."""
        company_id = transaction.get("company_id", "")

        recognition_result = self.recognition.recognize_transaction(transaction)
        logger.info("lifecycle_step", step="recognition", recognized=recognition_result.get("recognized"))

        if not recognition_result.get("recognized"):
            return {
                "status": "not_recognized",
                "recognition": recognition_result,
                "message": recognition_result.get("reason", "Transaction does not meet recognition criteria"),
            }

        journal_data = {
            **transaction,
            "debit_account_code": recognition_result.get("suggested_debit_account", transaction.get("debit_account_code")),
            "credit_account_code": recognition_result.get("suggested_credit_account", transaction.get("credit_account_code")),
        }
        journal_result = self.journal.create_journal_entry(journal_data)
        logger.info("lifecycle_step", step="journal", entry_id=journal_result.get("id"))

        self.journal.approve_entry(journal_result["id"], approver="SYSTEM_LIFECYCLE")
        self.journal.post_entry(journal_result["id"])
        logger.info("lifecycle_step", step="approve_post", entry_id=journal_result.get("id"))

        ledger_result = self.ledger.post_journal_entry(journal_result)
        logger.info("lifecycle_step", step="ledger_post", gl_entries=ledger_result.get("posted_to_gl"))

        return {
            "status": "completed",
            "company_id": company_id,
            "recognition": recognition_result,
            "journal_entry": journal_result,
            "ledger_posting": ledger_result,
        }

    async def process_adjusting_entries(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Process adjusting entries through the lifecycle."""
        adjustment_list = data.get("adjustments", [])
        company_id = data.get("company_id", "")
        period_end = data.get("period_end", "")

        adjustment_result = self.adjustments.generate_all_period_end_adjustments(company_id, period_end, adjustment_list)

        posted_entries: list = []
        for adj in adjustment_result.get("adjustments", []):
            je = self.journal.create_journal_entry({
                **adj,
                "company_id": company_id,
                "is_adjusting": True,
            })
            self.journal.approve_entry(je["id"], approver="SYSTEM_ADJUSTMENTS")
            self.journal.post_entry(je["id"])
            ledger_result = self.ledger.post_journal_entry(je)
            posted_entries.append({"adjustment": adj, "journal_entry": je, "ledger_posting": ledger_result})

        return {
            "status": "adjustments_posted",
            "company_id": company_id,
            "period_end": period_end,
            "total_adjustments": adjustment_result.get("total_adjustments", 0),
            "posted_entries": posted_entries,
        }

    async def generate_trial_balance(self, company_id: str, period_end: str, period_start: Optional[str] = None) -> Dict[str, Any]:
        """Generate the trial balance for a period."""
        return self.trial_balance.generate_trial_balance(company_id, period_end, period_start)

    async def generate_adjusted_trial_balance(
        self,
        company_id: str,
        period_end: str,
        period_start: Optional[str] = None,
        adjusting_entries: Optional[list] = None,
    ) -> Dict[str, Any]:
        """Generate an adjusted trial balance."""
        return self.trial_balance.generate_adjusted_trial_balance(company_id, period_end, period_start, adjusting_entries)

    async def close_period(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Close a period (month or year end)."""
        close_type = data.get("close_type", "month_end")
        data["ledger_engine"] = self.ledger

        if close_type == "year_end":
            result = self.closing.close_year(data)
        else:
            result = self.closing.close_month(data)

        for entry in result.get("closing_entries", []):
            je = self.journal.create_journal_entry({
                **entry,
                "company_id": data.get("company_id", ""),
                "is_adjusting": True,
            })
            self.journal.approve_entry(je["id"], approver="SYSTEM_CLOSING")
            self.journal.post_entry(je["id"])
            self.ledger.post_journal_entry(je)

        logger.info("period_closed", company_id=data.get("company_id"), close_type=close_type)
        return result

    async def validate_before_close(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate that a period is ready for closing."""
        return self.closing.validate_period_ready_for_close(data)

    def health_check(self) -> bool:
        return True
