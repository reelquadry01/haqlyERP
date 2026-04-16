# Author: Quadri Atharu
"""Accounting methods: accrual, cash, and modified cash basis processing."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from ..schemas.journal import JournalLineCreate
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class AccountingMethods:
    """Implements the three primary accounting methods used in Nigeria."""

    def process_accrual(self, transaction: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Process a transaction under the accrual basis.

        Revenue is recognized when earned regardless of when cash is received.
        Expenses are recognized when incurred regardless of when cash is paid.
        Returns a list of journal entry line dicts.
        """
        txn_type = transaction.get("transaction_type", "").upper()
        amount = float(transaction.get("amount", 0))
        if amount <= 0:
            raise AccountingError("Transaction amount must be positive for accrual processing")

        debit_account = transaction.get("debit_account_code", "")
        credit_account = transaction.get("credit_account_code", "")
        description = transaction.get("description", "")
        entry_date = transaction.get("transaction_date", datetime.now().isoformat())
        tax_amount = float(transaction.get("tax_amount", 0))
        tax_inclusive = transaction.get("tax_inclusive", False)

        lines: List[Dict[str, Any]] = []

        if tax_inclusive and tax_amount > 0:
            net_amount = _money_round(amount - tax_amount)
            lines.append({"account_code": debit_account, "description": description, "debit": net_amount, "credit": 0.0})
            lines.append({"account_code": "2310", "description": f"Output VAT - {description}", "debit": tax_amount, "credit": 0.0})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})
        else:
            gross = _money_round(amount + tax_amount) if tax_amount > 0 else amount
            lines.append({"account_code": debit_account, "description": description, "debit": gross, "credit": 0.0})
            if tax_amount > 0:
                lines.append({"account_code": "2310", "description": f"Output VAT - {description}", "debit": 0.0, "credit": tax_amount})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})

        if txn_type == "PURCHASE" and tax_amount > 0:
            lines = self._adjust_for_purchase_vat(lines, tax_amount, debit_account, credit_account, description, amount, tax_inclusive)

        logger.info("accrual_entry_created", transaction_type=txn_type, lines=len(lines))
        return lines

    def process_cash(self, transaction: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Process a transaction under the cash basis.

        Revenue is recognized only when cash is received.
        Expenses are recognized only when cash is paid.
        Only cash/bank accounts are used as the debit/credit counterpart.
        """
        txn_type = transaction.get("transaction_type", "").upper()
        amount = float(transaction.get("amount", 0))
        if amount <= 0:
            raise AccountingError("Transaction amount must be positive for cash processing")

        description = transaction.get("description", "")
        is_receipt = txn_type in ("SALES", "CASH_RECEIPT", "INVOICE")
        is_payment = txn_type in ("PURCHASE", "CASH_PAYMENT", "BILL", "EXPENSE")

        lines: List[Dict[str, Any]] = []

        if is_receipt:
            lines.append({"account_code": "1010", "description": f"Cash receipt - {description}", "debit": amount, "credit": 0.0})
            lines.append({"account_code": transaction.get("credit_account_code", "4000"), "description": description, "debit": 0.0, "credit": amount})
        elif is_payment:
            lines.append({"account_code": transaction.get("debit_account_code", "5000"), "description": description, "debit": amount, "credit": 0.0})
            lines.append({"account_code": "1010", "description": f"Cash payment - {description}", "debit": 0.0, "credit": amount})
        else:
            debit_account = transaction.get("debit_account_code", "1010")
            credit_account = transaction.get("credit_account_code", "4000")
            lines.append({"account_code": debit_account, "description": description, "debit": amount, "credit": 0.0})
            lines.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})

        logger.info("cash_entry_created", transaction_type=txn_type, lines=len(lines))
        return lines

    def process_modified_cash(self, transaction: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Process under modified cash basis: accrual for revenue, cash for expenses.

        Revenue is recognized when earned (accrual), but expenses are recognized
        only when cash is paid (cash basis). This is sometimes used by small
        Nigerian entities for simplicity while still recognizing receivables.
        """
        txn_type = transaction.get("transaction_type", "").upper()
        amount = float(transaction.get("amount", 0))
        if amount <= 0:
            raise AccountingError("Transaction amount must be positive for modified cash processing")

        description = transaction.get("description", "")
        is_revenue = txn_type in ("SALES", "INVOICE", "CASH_RECEIPT")

        if is_revenue:
            return self.process_accrual(transaction)
        else:
            return self.process_cash(transaction)

    @staticmethod
    def _adjust_for_purchase_vat(
        lines: List[Dict[str, Any]],
        tax_amount: float,
        debit_account: str,
        credit_account: str,
        description: str,
        amount: float,
        tax_inclusive: bool,
    ) -> List[Dict[str, Any]]:
        """Adjust journal lines for input VAT on purchases."""
        adjusted: List[Dict[str, Any]] = []
        if tax_inclusive and tax_amount > 0:
            net_amount = _money_round(amount - tax_amount)
            adjusted.append({"account_code": debit_account, "description": description, "debit": net_amount, "credit": 0.0})
            adjusted.append({"account_code": "1400", "description": f"Input VAT - {description}", "debit": tax_amount, "credit": 0.0})
            adjusted.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": amount})
        else:
            gross = _money_round(amount + tax_amount) if tax_amount > 0 else amount
            adjusted.append({"account_code": debit_account, "description": description, "debit": amount, "credit": 0.0})
            if tax_amount > 0:
                adjusted.append({"account_code": "1400", "description": f"Input VAT - {description}", "debit": tax_amount, "credit": 0.0})
            adjusted.append({"account_code": credit_account, "description": description, "debit": 0.0, "credit": gross})
        return adjusted

    def health_check(self) -> bool:
        return True
