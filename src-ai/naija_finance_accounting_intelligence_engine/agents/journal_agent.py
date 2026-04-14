# Author: Quadri Atharu
"""Journal Agent — processes transactions, creates journals, validates, and suggests accounts."""

from __future__ import annotations

from typing import Any, Dict

from ..accounting import JournalEngine
from .base_agent import BaseAgent


class JournalAgent(BaseAgent):
    """Agent responsible for journal entry operations."""

    agent_name = "journal_agent"

    def __init__(self, journal_engine: JournalEngine | None = None) -> None:
        super().__init__()
        self._journal_engine = journal_engine or JournalEngine()
        self.register_skill("process_transaction", self._process_transaction)
        self.register_skill("create_journal", self._create_journal)
        self.register_skill("validate_journal", self._validate_journal)
        self.register_skill("suggest_accounts", self._suggest_accounts)

    def _process_transaction(self, data: Dict[str, Any]) -> Dict[str, Any]:
        transaction_type = data.get("transaction_type", "JOURNAL")
        amount = float(data.get("amount", 0))
        description = data.get("description", "")
        entry_date = data.get("entry_date")
        company_id = data.get("company_id", "")
        debit_account = data.get("debit_account_code")
        credit_account = data.get("credit_account_code")
        tax_amount = float(data.get("tax_amount", 0))
        tax_inclusive = data.get("tax_inclusive", False)

        journal_data: Dict[str, Any] = {
            "transaction_type": transaction_type,
            "amount": amount,
            "description": description,
            "company_id": company_id,
            "debit_account_code": debit_account,
            "credit_account_code": credit_account,
            "tax_amount": tax_amount,
            "tax_inclusive": tax_inclusive,
            "source_id": data.get("source_id"),
            "reference": data.get("reference"),
            "is_adjusting": data.get("is_adjusting", False),
            "created_by": data.get("created_by"),
        }
        if entry_date:
            journal_data["entry_date"] = entry_date
        if data.get("lines"):
            journal_data["lines"] = data["lines"]

        entry = self._journal_engine.create_journal_entry(journal_data)
        return {
            "success": True,
            "journal_entry": entry,
            "entry_id": entry["id"],
            "entry_number": entry["entry_number"],
            "total_debit": entry["total_debit"],
            "total_credit": entry["total_credit"],
            "status": entry["status"],
        }

    def _create_journal(self, data: Dict[str, Any]) -> Dict[str, Any]:
        entry = self._journal_engine.create_journal_entry(data)
        if data.get("auto_approve"):
            approver = data.get("approver", "system")
            self._journal_engine.approve_entry(entry["id"], approver)
            entry = self._journal_engine.get_entry(entry["id"]) or entry
        if data.get("auto_post") and entry.get("status") == "APPROVED":
            self._journal_engine.post_entry(entry["id"])
            entry = self._journal_engine.get_entry(entry["id"]) or entry
        return {
            "success": True,
            "journal_entry": entry,
            "entry_id": entry["id"],
            "entry_number": entry["entry_number"],
        }

    def _validate_journal(self, data: Dict[str, Any]) -> Dict[str, Any]:
        entry_id = data.get("entry_id", "")
        lines = data.get("lines", [])
        entry = self._journal_engine.get_entry(entry_id) if entry_id else None

        errors: list[str] = []
        warnings: list[str] = []
        if entry:
            lines = entry.get("lines", [])

        if not lines:
            errors.append("Journal entry has no lines")
            return {"valid": False, "errors": errors, "warnings": warnings}

        total_debit = round(sum(float(l.get("debit", 0)) for l in lines), 2)
        total_credit = round(sum(float(l.get("credit", 0)) for l in lines), 2)
        balance_diff = abs(total_debit - total_credit)

        if balance_diff > 0.01:
            errors.append(f"Entry does not balance: debits={total_debit}, credits={total_credit}, diff={balance_diff}")
        if len(lines) < 2:
            errors.append("Journal entry must have at least 2 lines")

        for idx, line in enumerate(lines):
            debit = float(line.get("debit", 0))
            credit = float(line.get("credit", 0))
            if debit > 0 and credit > 0:
                errors.append(f"Line {idx + 1}: cannot have both debit ({debit}) and credit ({credit})")
            if debit == 0 and credit == 0:
                errors.append(f"Line {idx + 1}: must have either a debit or credit amount")
            if not line.get("account_code"):
                errors.append(f"Line {idx + 1}: missing account code")
            if debit < 0 or credit < 0:
                errors.append(f"Line {idx + 1}: negative amounts are not allowed")

        if total_debit == 0 and total_credit == 0:
            warnings.append("Journal entry has zero total amounts")

        return {
            "valid": len(errors) == 0,
            "errors": errors,
            "warnings": warnings,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "balanced": balance_diff <= 0.01,
            "line_count": len(lines),
        }

    def _suggest_accounts(self, data: Dict[str, Any]) -> Dict[str, Any]:
        transaction_type = data.get("transaction_type", "")
        description = data.get("description", "")
        suggestion = self._journal_engine.suggest_accounts(transaction_type, description)
        return {
            "success": True,
            "suggestion": suggestion,
            "transaction_type": transaction_type,
        }
