# Author: Quadri Atharu
"""Automated bank reconciliation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger

logger = get_logger(__name__)

TOLERANCE = 0.05


class BankReconciliationEngine:
    """Automated bank reconciliation engine."""

    def reconcile(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Perform bank reconciliation between book and bank statement."""
        book_balance = float(data.get("book_balance", 0))
        bank_balance = float(data.get("bank_balance", 0))
        company_id = data.get("company_id", "")
        bank_name = data.get("bank_name", "")
        account_number = data.get("account_number", "")
        period_end = data.get("period_end", datetime.now().isoformat())
        currency = data.get("currency", "NGN")

        outstanding_deposits_list: List[Dict[str, Any]] = data.get("outstanding_deposits", [])
        outstanding_cheques_list: List[Dict[str, Any]] = data.get("outstanding_cheques", [])
        bank_charges = float(data.get("bank_charges", 0))
        bank_interest = float(data.get("bank_interest", 0))
        errors_list: List[Dict[str, Any]] = data.get("errors", [])

        outstanding_deposits_total = round(sum(float(d.get("amount", 0)) for d in outstanding_deposits_list), 2)
        outstanding_cheques_total = round(sum(float(c.get("amount", 0)) for c in outstanding_cheques_list), 2)
        errors_total = round(sum(float(e.get("amount", 0)) for e in errors_list), 2)

        adjusted_bank = round(bank_balance + outstanding_deposits_total - outstanding_cheques_total, 2)
        adjusted_book = round(book_balance - bank_charges + bank_interest + errors_total, 2)
        difference = round(adjusted_book - adjusted_bank, 2)
        reconciled = abs(difference) < TOLERANCE

        result: Dict[str, Any] = {
            "company_id": company_id,
            "bank_name": bank_name,
            "account_number": account_number,
            "period_end": period_end,
            "currency": currency,
            "book_balance": round(book_balance, 2),
            "bank_balance": round(bank_balance, 2),
            "outstanding_deposits": outstanding_deposits_list,
            "outstanding_deposits_total": outstanding_deposits_total,
            "outstanding_cheques": outstanding_cheques_list,
            "outstanding_cheques_total": outstanding_cheques_total,
            "bank_charges": round(bank_charges, 2),
            "bank_interest": round(bank_interest, 0),
            "errors": errors_list,
            "errors_total": errors_total,
            "adjusted_bank_balance": adjusted_bank,
            "adjusted_book_balance": adjusted_book,
            "difference": difference,
            "reconciled": reconciled,
            "reconciliation_items": self._build_reconciliation_items(
                outstanding_deposits_total, outstanding_cheques_total,
                bank_charges, bank_interest, errors_total,
            ),
        }

        if not reconciled:
            result["investigation"] = self._suggest_investigation(difference)

        logger.info("bank_reconciliation", bank=bank_name, reconciled=reconciled, difference=difference)
        return result

    def auto_match_transactions(
        self,
        book_entries: List[Dict[str, Any]],
        bank_entries: List[Dict[str, Any]],
        tolerance: float = TOLERANCE,
    ) -> Dict[str, Any]:
        """Automatically match book entries to bank statement entries."""
        matched: List[Dict[str, Any]] = []
        unmatched_book: List[Dict[str, Any]] = list(book_entries)
        unmatched_bank: List[Dict[str, Any]] = list(bank_entries)

        for book in list(unmatched_book):
            best_match = None
            best_diff = float("inf")
            for bank in unmatched_bank:
                if book.get("reference") and bank.get("reference") and book["reference"] == bank["reference"]:
                    amt_diff = abs(float(book.get("amount", 0)) - float(bank.get("amount", 0)))
                    if amt_diff <= tolerance:
                        best_match = bank
                        best_diff = amt_diff
                        break

            if best_match is None:
                for bank in unmatched_bank:
                    amt_diff = abs(float(book.get("amount", 0)) - float(bank.get("amount", 0)))
                    date_diff = abs((datetime.fromisoformat(book.get("date", "2000-01-01")) - datetime.fromisoformat(bank.get("date", "2000-01-01"))).days) if book.get("date") and bank.get("date") else 999
                    if amt_diff <= tolerance and date_diff <= 3 and amt_diff < best_diff:
                        best_match = bank
                        best_diff = amt_diff

            if best_match is not None:
                matched.append({"book_entry": book, "bank_entry": best_match, "amount_difference": best_diff})
                unmatched_book.remove(book)
                unmatched_bank.remove(best_match)

        return {
            "total_book_entries": len(book_entries),
            "total_bank_entries": len(bank_entries),
            "matched_count": len(matched),
            "unmatched_book_count": len(unmatched_book),
            "unmatched_bank_count": len(unmatched_bank),
            "match_rate": round(len(matched) / max(len(book_entries), 1), 4),
            "matched_pairs": matched,
            "unmatched_book_entries": unmatched_book,
            "unmatched_bank_entries": unmatched_bank,
        }

    def _build_reconciliation_items(self, deposits: float, cheques: float, charges: float, interest: float, errors: float) -> List[Dict[str, Any]]:
        """Build reconciliation adjustment items list."""
        items: List[Dict[str, Any]] = []
        if deposits:
            items.append({"type": "add_to_bank", "description": "Outstanding deposits", "amount": deposits})
        if cheques:
            items.append({"type": "deduct_from_bank", "description": "Outstanding cheques", "amount": cheques})
        if charges:
            items.append({"type": "deduct_from_book", "description": "Bank charges", "amount": charges})
        if interest:
            items.append({"type": "add_to_book", "description": "Bank interest earned", "amount": interest})
        if errors:
            items.append({"type": "adjust_book", "description": "Book errors", "amount": errors})
        return items

    @staticmethod
    def _suggest_investigation(difference: float) -> Dict[str, Any]:
        """Suggest investigation steps for unreconciled difference."""
        suggestions: List[str] = [
            "Verify all outstanding deposits and cheques are recorded",
            "Check for bank charges or fees not yet booked",
            "Look for timing differences in deposit recordings",
        ]
        if abs(difference) % 9 < 0.01:
            suggestions.append("Difference divisible by 9 — possible transposition error")
        if difference > 0:
            suggestions.append("Book balance higher — possible unrecorded bank charges or payments")
        else:
            suggestions.append("Bank balance higher — possible unrecorded deposits or bank interest")

        return {"difference": difference, "suggestions": suggestions}

    def health_check(self) -> bool:
        return True
