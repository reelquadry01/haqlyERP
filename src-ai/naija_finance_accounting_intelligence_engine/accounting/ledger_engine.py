# Author: Quadri Atharu
"""General Ledger posting engine with sub-ledger support."""

from __future__ import annotations

import uuid
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple

from ..core.exceptions import AccountingError
from ..core.logging import get_logger

logger = get_logger(__name__)

ACCOUNT_TYPES = {
    "1": "ASSET",
    "2": "LIABILITY",
    "3": "EQUITY",
    "4": "REVENUE",
    "5": "EXPENSE",
}

DEBIT_NORMAL = {"ASSET", "EXPENSE"}
CREDIT_NORMAL = {"LIABILITY", "EQUITY", "REVENUE"}

SUBLEDGER_TYPES = {
    "1100": "accounts_receivable",
    "2100": "accounts_payable",
    "1600": "fixed_assets",
    "1200": "inventory",
}


class LedgerEngine:
    """General Ledger posting engine with T-account balancing and sub-ledger support."""

    def __init__(self) -> None:
        self._ledger_entries: Dict[str, List[Dict[str, Any]]] = {}
        self._account_balances: Dict[str, Dict[str, Any]] = {}
        self._sub_ledger_entries: Dict[str, List[Dict[str, Any]]] = {}

    def post_journal_entry(self, journal_entry: Dict[str, Any]) -> Dict[str, Any]:
        """Post a journal entry to the general ledger and sub-ledgers."""
        entry_id = journal_entry.get("id", str(uuid.uuid4()))
        company_id = journal_entry.get("company_id", "")
        entry_date = journal_entry.get("entry_date", datetime.now().isoformat())
        description = journal_entry.get("description", "")
        je_number = journal_entry.get("entry_number", "")

        posted_entries: List[Dict[str, Any]] = []
        sub_entries: List[Dict[str, Any]] = []

        for line in journal_entry.get("lines", []):
            account_code = str(line.get("account_code", ""))
            debit = float(line.get("debit", 0))
            credit = float(line.get("credit", 0))

            account_type = self._infer_account_type(account_code)
            ledger_entry = {
                "id": str(uuid.uuid4()),
                "account_code": account_code,
                "account_name": self._get_account_name(account_code),
                "account_type": account_type,
                "journal_entry_id": entry_id,
                "entry_date": entry_date,
                "description": line.get("description", description),
                "debit": round(debit, 2),
                "credit": round(credit, 2),
                "cost_center": line.get("cost_center"),
                "reference": line.get("reference"),
                "company_id": company_id,
                "created_at": datetime.now().isoformat(),
            }

            self._add_ledger_entry(account_code, ledger_entry)
            posted_entries.append(ledger_entry)

            if account_code in SUBLEDGER_TYPES:
                sub_entry = self._create_sub_ledger_entry(
                    account_code=account_code,
                    subledger_type=SUBLEDGER_TYPES[account_code],
                    journal_entry_id=entry_id,
                    entry_date=entry_date,
                    description=line.get("description", description),
                    debit=debit,
                    credit=credit,
                    counterparty=journal_entry.get("counterparty"),
                    reference=line.get("reference"),
                    due_date=journal_entry.get("due_date"),
                    company_id=company_id,
                )
                sub_entries.append(sub_entry)

        self._update_all_balances()

        result: Dict[str, Any] = {
            "journal_entry_id": entry_id,
            "entry_number": je_number,
            "posted_to_gl": len(posted_entries),
            "posted_to_sub_ledger": len(sub_entries),
            "ledger_entries": posted_entries,
            "sub_ledger_entries": sub_entries,
            "posted_at": datetime.now().isoformat(),
        }
        logger.info("journal_posted_to_ledger", entry_id=entry_id, gl_entries=len(posted_entries), sub_entries=len(sub_entries))
        return result

    def compute_account_balance(self, account_code: str, period_start: Optional[str] = None, period_end: Optional[str] = None) -> Dict[str, Any]:
        """Compute the balance for a specific account, optionally within a period."""
        entries = self._ledger_entries.get(account_code, [])

        if period_start or period_end:
            entries = [
                e for e in entries
                if (not period_start or e.get("entry_date", "") >= period_start)
                and (not period_end or e.get("entry_date", "") <= period_end)
            ]

        total_debit = round(sum(e.get("debit", 0) for e in entries), 2)
        total_credit = round(sum(e.get("credit", 0) for e in entries), 2)
        account_type = self._infer_account_type(account_code)

        if account_type in DEBIT_NORMAL:
            closing_balance = round(total_debit - total_credit, 2)
        else:
            closing_balance = round(total_credit - total_debit, 2)

        opening_balance = 0.0
        if period_start:
            pre_entries = [e for e in self._ledger_entries.get(account_code, []) if e.get("entry_date", "") < period_start]
            pre_debit = sum(e.get("debit", 0) for e in pre_entries)
            pre_credit = sum(e.get("credit", 0) for e in pre_entries)
            if account_type in DEBIT_NORMAL:
                opening_balance = round(pre_debit - pre_credit, 2)
            else:
                opening_balance = round(pre_credit - pre_debit, 2)

        return {
            "account_code": account_code,
            "account_name": self._get_account_name(account_code),
            "account_type": account_type,
            "normal_balance": "debit" if account_type in DEBIT_NORMAL else "credit",
            "opening_balance": opening_balance,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "closing_balance": closing_balance,
            "entry_count": len(entries),
            "period_start": period_start,
            "period_end": period_end,
        }

    def get_t_account(self, account_code: str) -> Dict[str, Any]:
        """Generate a T-account view for a given account."""
        balance_data = self.compute_account_balance(account_code)
        entries = self._ledger_entries.get(account_code, [])

        debit_side: List[Dict[str, Any]] = []
        credit_side: List[Dict[str, Any]] = []

        for entry in entries:
            if entry.get("debit", 0) > 0:
                debit_side.append({"date": entry["entry_date"], "description": entry["description"], "amount": entry["debit"], "reference": entry.get("reference")})
            elif entry.get("credit", 0) > 0:
                credit_side.append({"date": entry["entry_date"], "description": entry["description"], "amount": entry["credit"], "reference": entry.get("reference")})

        return {
            "account_code": account_code,
            "account_name": balance_data["account_name"],
            "account_type": balance_data["account_type"],
            "debit_side": debit_side,
            "credit_side": credit_side,
            "total_debit": balance_data["total_debit"],
            "total_credit": balance_data["total_credit"],
            "balance": balance_data["closing_balance"],
            "balance_type": balance_data["normal_balance"],
        }

    def get_sub_ledger(self, control_account_code: str) -> Dict[str, Any]:
        """Retrieve sub-ledger entries for a control account."""
        sub_type = SUBLEDGER_TYPES.get(control_account_code)
        if sub_type is None:
            return {"control_account": control_account_code, "sub_ledger_type": None, "entries": [], "total_balance": 0.0}

        entries = self._sub_ledger_entries.get(control_account_code, [])
        total_debit = round(sum(e.get("debit", 0) for e in entries), 2)
        total_credit = round(sum(e.get("credit", 0) for e in entries), 2)
        account_type = self._infer_account_type(control_account_code)

        if account_type in DEBIT_NORMAL:
            balance = round(total_debit - total_credit, 2)
        else:
            balance = round(total_credit - total_debit, 2)

        return {
            "control_account": control_account_code,
            "sub_ledger_type": sub_type,
            "entries": entries,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "total_balance": balance,
            "entry_count": len(entries),
        }

    def reconcile_sub_ledger(self, control_account_code: str) -> Dict[str, Any]:
        """Reconcile sub-ledger total with the control account balance."""
        gl_balance = self.compute_account_balance(control_account_code)
        sub_ledger = self.get_sub_ledger(control_account_code)

        gl_closing = gl_balance["closing_balance"]
        sub_closing = sub_ledger.get("total_balance", 0.0)
        difference = round(gl_closing - sub_closing, 2)
        reconciled = abs(difference) < 0.01

        return {
            "control_account": control_account_code,
            "gl_closing_balance": gl_closing,
            "sub_ledger_closing_balance": sub_closing,
            "difference": difference,
            "reconciled": reconciled,
            "gl_entry_count": gl_balance["entry_count"],
            "sub_ledger_entry_count": sub_ledger.get("entry_count", 0),
        }

    def get_all_balances(self, account_type_filter: Optional[str] = None) -> List[Dict[str, Any]]:
        """Return all account balances, optionally filtered by account type."""
        balances: List[Dict[str, Any]] = []
        for code in sorted(self._ledger_entries.keys()):
            if account_type_filter:
                acct_type = self._infer_account_type(code)
                if acct_type != account_type_filter.upper():
                    continue
            balances.append(self.compute_account_balance(code))
        return balances

    def _add_ledger_entry(self, account_code: str, entry: Dict[str, Any]) -> None:
        """Add an entry to the ledger for a specific account."""
        if account_code not in self._ledger_entries:
            self._ledger_entries[account_code] = []
        self._ledger_entries[account_code].append(entry)

    def _create_sub_ledger_entry(
        self,
        account_code: str,
        subledger_type: str,
        journal_entry_id: str,
        entry_date: str,
        description: str,
        debit: float,
        credit: float,
        counterparty: Optional[str],
        reference: Optional[str],
        due_date: Optional[str],
        company_id: str,
    ) -> Dict[str, Any]:
        """Create a sub-ledger entry for a control account."""
        entry = {
            "id": str(uuid.uuid4()),
            "subledger_type": subledger_type,
            "control_account_code": account_code,
            "subledger_account_id": str(uuid.uuid4()),
            "journal_entry_id": journal_entry_id,
            "entry_date": entry_date,
            "description": description,
            "debit": round(debit, 2),
            "credit": round(credit, 2),
            "counterparty": counterparty,
            "reference": reference,
            "due_date": due_date,
            "company_id": company_id,
            "created_at": datetime.now().isoformat(),
        }

        if account_code not in self._sub_ledger_entries:
            self._sub_ledger_entries[account_code] = []
        self._sub_ledger_entries[account_code].append(entry)
        return entry

    def _update_all_balances(self) -> None:
        """Recalculate all account balances from entries."""
        for code in self._ledger_entries:
            self._account_balances[code] = self.compute_account_balance(code)

    @staticmethod
    def _infer_account_type(account_code: str) -> str:
        """Infer the account type from the first digit of the account code."""
        if account_code and account_code[0] in ACCOUNT_TYPES:
            return ACCOUNT_TYPES[account_code[0]]
        return "EXPENSE"

    @staticmethod
    def _get_account_name(account_code: str) -> str:
        """Return a standard account name based on code (fallback for unknown codes)."""
        names: Dict[str, str] = {
            "1010": "Cash", "1020": "Bank", "1100": "Accounts Receivable",
            "1200": "Inventory", "1300": "Prepayments", "1400": "Input VAT",
            "1600": "Property, Plant & Equipment", "1610": "Accumulated Depreciation",
            "2100": "Accounts Payable", "2300": "Tax Payable", "2310": "Output VAT",
            "2320": "CIT Payable", "2330": "Education Tax Payable", "2350": "WHT Payable",
            "2600": "Lease Liability", "2900": "Provisions",
            "3000": "Share Capital", "3100": "Retained Earnings", "3200": "Revaluation Reserve",
            "4000": "Sales Revenue", "4100": "Other Income",
            "5000": "Cost of Sales", "5100": "Operating Expenses", "5200": "Depreciation",
            "5600": "Lease Expense", "5700": "Interest Expense", "5900": "Provision/Impairment",
            "9900": "Suspense",
        }
        return names.get(account_code, f"Account {account_code}")

    def health_check(self) -> bool:
        return True
