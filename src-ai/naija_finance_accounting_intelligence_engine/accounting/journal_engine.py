# Author: Quadri Atharu
"""Double-entry journal engine with auto-suggest capabilities."""

from __future__ import annotations

import uuid
from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from ..schemas.journal import JournalEntryCreate, JournalLineCreate, JournalStatus

logger = get_logger(__name__)

ACCOUNT_SUGGESTIONS: Dict[str, Dict[str, str]] = {
    "SALES": {"debit": "1100", "credit": "4000", "debit_name": "Accounts Receivable", "credit_name": "Sales Revenue"},
    "CASH_RECEIPT": {"debit": "1010", "credit": "1100", "debit_name": "Cash", "credit_name": "Accounts Receivable"},
    "PURCHASE": {"debit": "5000", "credit": "2100", "debit_name": "Purchases", "credit_name": "Accounts Payable"},
    "CASH_PAYMENT": {"debit": "2100", "credit": "1010", "debit_name": "Accounts Payable", "credit_name": "Cash"},
    "EXPENSE": {"debit": "5100", "credit": "1010", "debit_name": "Operating Expenses", "credit_name": "Cash"},
    "SALARY": {"debit": "5100", "credit": "1010", "debit_name": "Salaries & Wages", "credit_name": "Cash"},
    "TAX_PAYMENT": {"debit": "2300", "credit": "1010", "debit_name": "Tax Payable", "credit_name": "Cash"},
    "DEPRECIATION": {"debit": "5200", "credit": "1610", "debit_name": "Depreciation Expense", "credit_name": "Accumulated Depreciation"},
    "INVOICE": {"debit": "1100", "credit": "4000", "debit_name": "Accounts Receivable", "credit_name": "Sales Revenue"},
    "BILL": {"debit": "5000", "credit": "2100", "debit_name": "Purchases/Expenses", "credit_name": "Accounts Payable"},
    "TRANSFER": {"debit": "1020", "credit": "1010", "debit_name": "Bank Account", "credit_name": "Cash"},
    "CREDIT_NOTE": {"debit": "4000", "credit": "1100", "debit_name": "Sales Revenue", "credit_name": "Accounts Receivable"},
    "DEBIT_NOTE": {"debit": "2100", "credit": "5000", "debit_name": "Accounts Payable", "credit_name": "Purchases/Expenses"},
}

TAX_ACCOUNTS = {
    "vat_output": {"code": "2310", "name": "Output VAT"},
    "vat_input": {"code": "1400", "name": "Input VAT"},
    "wht": {"code": "2350", "name": "Withholding Tax Payable"},
    "cit": {"code": "2320", "name": "CIT Payable"},
    "edu_tax": {"code": "2330", "name": "Education Tax Payable"},
    "cgt": {"code": "2340", "name": "Capital Gains Tax Payable"},
}


class JournalEngine:
    """Double-entry journal creation engine with auto-suggest and validation."""

    def __init__(self) -> None:
        self._entries: Dict[str, Dict[str, Any]] = {}
        self._entry_counter = 0

    def create_journal_entry(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a complete double-entry journal entry from transaction data."""
        self._entry_counter += 1
        entry_number = f"JE-{datetime.now().strftime('%Y%m')}-{self._entry_counter:05d}"
        entry_id = str(uuid.uuid4())

        transaction_type = data.get("transaction_type", "JOURNAL").upper()
        amount = float(data.get("amount", 0))
        description = data.get("description", "")
        entry_date = data.get("entry_date", datetime.now().isoformat())
        company_id = data.get("company_id", "")

        if amount <= 0:
            raise AccountingError("Journal entry amount must be positive")

        lines = data.get("lines")
        if lines is None:
            lines = self._auto_generate_lines(transaction_type, amount, description, data)

        validated_lines = self._validate_double_entry(lines)
        total_debit = round(sum(l.get("debit", 0) for l in validated_lines), 2)
        total_credit = round(sum(l.get("credit", 0) for l in validated_lines), 2)

        balance_diff = abs(total_debit - total_credit)
        if balance_diff > 0.01:
            raise AccountingError(
                f"Journal entry does not balance: debits={total_debit}, credits={total_credit}, diff={balance_diff}",
                details={"total_debit": total_debit, "total_credit": total_credit},
            )

        entry: Dict[str, Any] = {
            "id": entry_id,
            "entry_number": entry_number,
            "company_id": company_id,
            "entry_date": entry_date,
            "description": description,
            "reference": data.get("reference"),
            "source_type": transaction_type,
            "source_id": data.get("source_id"),
            "is_adjusting": data.get("is_adjusting", False),
            "is_reversing": data.get("is_reversing", False),
            "reversing_entry_id": data.get("reversing_entry_id"),
            "status": JournalStatus.DRAFT.value,
            "lines": validated_lines,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "created_by": data.get("created_by"),
            "approved_by": None,
            "created_at": datetime.now().isoformat(),
            "updated_at": datetime.now().isoformat(),
        }

        self._entries[entry_id] = entry
        logger.info("journal_entry_created", entry_number=entry_number, total_debit=total_debit, total_credit=total_credit)
        return entry

    def reverse_entry(self, entry_id: str, reason: str = "") -> Dict[str, Any]:
        """Create a reversing journal entry for a given entry."""
        original = self._entries.get(entry_id)
        if original is None:
            raise AccountingError(f"Journal entry {entry_id} not found")
        if original.get("status") == JournalStatus.VOID.value:
            raise AccountingError(f"Entry {entry_id} is already voided")

        reversed_lines: List[Dict[str, Any]] = []
        for line in original.get("lines", []):
            reversed_lines.append({
                "account_code": line["account_code"],
                "description": f"Reversal: {line.get('description', original.get('description', ''))}",
                "debit": line.get("credit", 0),
                "credit": line.get("debit", 0),
                "reference": line.get("reference"),
                "cost_center": line.get("cost_center"),
                "tax_code": line.get("tax_code"),
            })

        reverse_data = {
            "transaction_type": "JOURNAL",
            "amount": original.get("total_debit", 0),
            "description": f"Reversal of {original.get('entry_number', '')} — {reason}",
            "lines": reversed_lines,
            "company_id": original.get("company_id", ""),
            "is_reversing": True,
            "reversing_entry_id": entry_id,
            "entry_date": datetime.now().isoformat(),
        }

        original["status"] = JournalStatus.REVERSED.value
        original["updated_at"] = datetime.now().isoformat()
        return self.create_journal_entry(reverse_data)

    def approve_entry(self, entry_id: str, approver: str) -> Dict[str, Any]:
        """Approve a journal entry."""
        entry = self._entries.get(entry_id)
        if entry is None:
            raise AccountingError(f"Journal entry {entry_id} not found")
        if entry.get("status") not in (JournalStatus.DRAFT.value, JournalStatus.SUBMITTED.value):
            raise AccountingError(f"Cannot approve entry in status: {entry.get('status')}")
        entry["status"] = JournalStatus.APPROVED.value
        entry["approved_by"] = approver
        entry["updated_at"] = datetime.now().isoformat()
        logger.info("journal_entry_approved", entry_id=entry_id, approver=approver)
        return entry

    def post_entry(self, entry_id: str) -> Dict[str, Any]:
        """Mark a journal entry as posted (ready for ledger)."""
        entry = self._entries.get(entry_id)
        if entry is None:
            raise AccountingError(f"Journal entry {entry_id} not found")
        if entry.get("status") != JournalStatus.APPROVED.value:
            raise AccountingError(f"Only approved entries can be posted. Current: {entry.get('status')}")
        entry["status"] = JournalStatus.POSTED.value
        entry["updated_at"] = datetime.now().isoformat()
        logger.info("journal_entry_posted", entry_id=entry_id)
        return entry

    def void_entry(self, entry_id: str) -> Dict[str, Any]:
        """Void a journal entry (only draft/submitted)."""
        entry = self._entries.get(entry_id)
        if entry is None:
            raise AccountingError(f"Journal entry {entry_id} not found")
        if entry.get("status") not in (JournalStatus.DRAFT.value, JournalStatus.SUBMITTED.value):
            raise AccountingError("Only draft/submitted entries can be voided")
        entry["status"] = JournalStatus.VOID.value
        entry["updated_at"] = datetime.now().isoformat()
        logger.info("journal_entry_voided", entry_id=entry_id)
        return entry

    def get_entry(self, entry_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve a journal entry by ID."""
        return self._entries.get(entry_id)

    def list_entries(self, company_id: Optional[str] = None, status: Optional[str] = None) -> List[Dict[str, Any]]:
        """List journal entries with optional filtering."""
        entries = list(self._entries.values())
        if company_id:
            entries = [e for e in entries if e.get("company_id") == company_id]
        if status:
            entries = [e for e in entries if e.get("status") == status]
        return sorted(entries, key=lambda e: e.get("created_at", ""), reverse=True)

    def suggest_accounts(self, transaction_type: str, description: str = "") -> Dict[str, Any]:
        """Auto-suggest debit/credit accounts based on transaction type."""
        suggestion = ACCOUNT_SUGGESTIONS.get(transaction_type.upper())
        if suggestion:
            return {
                "transaction_type": transaction_type,
                "suggested_debit_account": suggestion["debit"],
                "suggested_debit_name": suggestion["debit_name"],
                "suggested_credit_account": suggestion["credit"],
                "suggested_credit_name": suggestion["credit_name"],
                "confidence": 0.90,
            }
        return {
            "transaction_type": transaction_type,
            "suggested_debit_account": None,
            "suggested_credit_account": None,
            "confidence": 0.0,
            "message": "No auto-suggestion available; manual entry required",
        }

    def generate_tax_lines(self, amount: float, tax_type: str = "vat", tax_inclusive: bool = False, tax_rate: float = 0.075) -> List[Dict[str, Any]]:
        """Generate tax-related journal lines for a transaction."""
        lines: List[Dict[str, Any]] = []

        if tax_type.lower() == "vat":
            if tax_inclusive:
                taxable = round(amount / (1 + tax_rate), 2)
                vat_amount = round(amount - taxable, 2)
            else:
                taxable = amount
                vat_amount = round(amount * tax_rate, 2)
            lines.append({"account_code": TAX_ACCOUNTS["vat_output"]["code"], "description": f"Output VAT @ {tax_rate * 100}%", "debit": 0.0, "credit": vat_amount, "tax_code": "VAT_OUT"})
            lines.append({"account_code": TAX_ACCOUNTS["vat_input"]["code"], "description": f"Input VAT @ {tax_rate * 100}%", "debit": vat_amount, "credit": 0.0, "tax_code": "VAT_IN"})

        elif tax_type.lower() == "wht":
            wht_rate = float(self._get_wht_rate(amount))
            wht_amount = round(amount * wht_rate, 2)
            lines.append({"account_code": TAX_ACCOUNTS["wht"]["code"], "description": f"WHT @ {wht_rate * 100}%", "debit": 0.0, "credit": wht_amount, "tax_code": "WHT"})

        return lines

    def _auto_generate_lines(self, transaction_type: str, amount: float, description: str, data: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Auto-generate journal lines based on transaction type."""
        suggestion = ACCOUNT_SUGGESTIONS.get(transaction_type, {})
        debit_account = data.get("debit_account_code") or suggestion.get("debit", "9999")
        credit_account = data.get("credit_account_code") or suggestion.get("credit", "9999")
        tax_amount = float(data.get("tax_amount", 0))
        tax_inclusive = data.get("tax_inclusive", False)

        lines: List[Dict[str, Any]] = []

        if tax_inclusive and tax_amount > 0:
            net_amount = round(amount - tax_amount, 2)
            lines.append({"account_code": debit_account, "description": description, "debit": net_amount, "credit": 0.0})
            lines.append({"account_code": TAX_ACCOUNTS["vat_output"]["code"], "description": f"Output VAT — {description}", "debit": tax_amount, "credit": 0.0})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})
        elif tax_amount > 0:
            lines.append({"account_code": debit_account, "description": description, "debit": amount, "credit": 0.0})
            lines.append({"account_code": TAX_ACCOUNTS["vat_output"]["code"], "description": f"Output VAT — {description}", "debit": 0.0, "credit": tax_amount})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount + tax_amount})
        else:
            lines.append({"account_code": debit_account, "description": description, "debit": amount, "credit": 0.0})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})

        return lines

    @staticmethod
    def _validate_double_entry(lines: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Validate and normalize journal lines for double-entry compliance."""
        validated: List[Dict[str, Any]] = []
        for line in lines:
            debit = float(line.get("debit", 0))
            credit = float(line.get("credit", 0))
            if debit > 0 and credit > 0:
                raise AccountingError(f"Line cannot have both debit ({debit}) and credit ({credit})")
            if debit == 0 and credit == 0:
                raise AccountingError("Journal line must have either a debit or credit amount")
            if not line.get("account_code"):
                raise AccountingError("Journal line must have an account code")
            validated.append({
                "account_code": line["account_code"],
                "description": line.get("description", ""),
                "debit": round(debit, 2),
                "credit": round(credit, 2),
                "reference": line.get("reference"),
                "cost_center": line.get("cost_center"),
                "tax_code": line.get("tax_code"),
            })
        if len(validated) < 2:
            raise AccountingError("Journal entry must have at least 2 lines")
        return validated

    @staticmethod
    def _get_wht_rate(amount: float) -> float:
        """Return a default WHT rate based on amount thresholds."""
        if amount <= 0:
            return 0.0
        return 0.05

    def health_check(self) -> bool:
        return True
