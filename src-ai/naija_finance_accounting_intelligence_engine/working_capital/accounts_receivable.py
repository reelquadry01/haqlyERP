# Author: Quadri Atharu
"""Accounts Receivable aging and DSO computation engine."""

from __future__ import annotations

from datetime import datetime, timedelta
from typing import Any, Dict, List

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

AGING_BUCKETS = [
    ("current", 0, 30),
    ("31_60_days", 31, 60),
    ("61_90_days", 61, 90),
    ("91_180_days", 91, 180),
    ("181_365_days", 181, 365),
    ("over_365_days", 366, 99999),
]

PROVISION_RATES = {
    "current": 0.0,
    "31_60_days": 0.0,
    "61_90_days": 0.05,
    "91_180_days": 0.10,
    "181_365_days": 0.25,
    "over_365_days": 0.50,
}


class AccountsReceivableEngine:
    """Accounts Receivable aging, DSO, and provision engine."""

    def compute_aging(self, invoices: List[Dict[str, Any]], as_of_date: str = "") -> Dict[str, Any]:
        """Compute AR aging report."""
        try:
            ref_date = datetime.fromisoformat(as_of_date) if as_of_date else datetime.now()
        except (ValueError, TypeError):
            ref_date = datetime.now()

        buckets: Dict[str, List[Dict[str, Any]]] = {name: [] for name, _, _ in AGING_BUCKETS}
        bucket_totals: Dict[str, float] = {name: 0.0 for name, _, _ in AGING_BUCKETS}

        total_ar = 0.0
        total_overdue = 0.0

        for inv in invoices:
            amount = float(inv.get("amount", 0))
            due_date_str = inv.get("due_date", "")
            try:
                due_date = datetime.fromisoformat(due_date_str) if due_date_str else ref_date
            except (ValueError, TypeError):
                due_date = ref_date

            days_overdue = max((ref_date - due_date).days, 0)
            total_ar += amount

            placed = False
            for name, low, high in AGING_BUCKETS:
                if low <= days_overdue <= high:
                    buckets[name].append({
                        "invoice_id": inv.get("id", ""),
                        "customer": inv.get("customer", ""),
                        "amount": _money_round(amount),
                        "due_date": due_date_str,
                        "days_overdue": days_overdue,
                    })
                    bucket_totals[name] = _money_round(bucket_totals[name] + amount)
                    placed = True
                    break
            if not placed:
                buckets["over_365_days"].append({
                    "invoice_id": inv.get("id", ""),
                    "customer": inv.get("customer", ""),
                    "amount": _money_round(amount),
                    "due_date": due_date_str,
                    "days_overdue": days_overdue,
                })
                bucket_totals["over_365_days"] = _money_round(bucket_totals["over_365_days"] + amount)

            if days_overdue > 0:
                total_overdue += amount

        provision = self._compute_provision(bucket_totals)

        return {
            "as_of_date": ref_date.isoformat(),
            "total_ar": _money_round(total_ar),
            "total_overdue": _money_round(total_overdue),
            "overdue_pct": round(total_overdue / total_ar, 4) if total_ar > 0 else 0,
            "aging_buckets": {name: {"total": bucket_totals[name], "count": len(buckets[name]), "invoices": buckets[name]} for name, _, _ in AGING_BUCKETS},
            "bucket_totals": bucket_totals,
            "provision_for_bad_debts": provision,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dso(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Days Sales Outstanding (DSO)."""
        accounts_receivable = Decimal(str(data.get("accounts_receivable", 0)))
        annual_revenue = Decimal(str(data.get("annual_revenue", 0)))
        period_days = int(data.get("period_days", 365))

        if annual_revenue <= 0:
            return {"dso": None, "message": "Revenue must be positive for DSO computation"}

        dso = _money_round(accounts_receivable / (annual_revenue / period_days))

        if dso <= 30:
            health = "excellent"
        elif dso <= 45:
            health = "good"
        elif dso <= 60:
            health = "adequate"
        elif dso <= 90:
            health = "poor"
        else:
            health = "critical"

        return {
            "accounts_receivable": _money_round(accounts_receivable),
            "annual_revenue": _money_round(annual_revenue),
            "dso": dso,
            "health": health,
            "recommendation": "Improve collection processes" if dso > 60 else "DSO within acceptable range",
            "computed_at": datetime.now().isoformat(),
        }

    def compute_bad_debt_provision(self, aging_data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute IFRS 9 expected credit loss provision."""
        bucket_totals = aging_data.get("bucket_totals", {})

        ifrs9_provision = 0.0
        details: List[Dict[str, Any]] = []

        for name, _, _ in AGING_BUCKETS:
            amount = bucket_totals.get(name, 0)
            rate = PROVISION_RATES.get(name, 0)
            provision = _money_round(amount * rate)
            ifrs9_provision += provision
            details.append({"bucket": name, "amount": amount, "provision_rate": rate, "provision": provision})

        simplified_provision = round(aging_data.get("total_ar", 0) * 0.05, 2)

        return {
            "standard": "IFRS 9",
            "aging_based_provision": _money_round(ifrs9_provision),
            "simplified_approach_provision": simplified_provision,
            "recommended_provision": max(ifrs9_provision, simplified_provision),
            "details": details,
            "journal_entry": {
                "debit_account": "5900",
                "credit_account": "1105",
                "description": "Bad debt provision per IFRS 9 ECL",
                "amount": max(ifrs9_provision, simplified_provision),
            },
        }

    @staticmethod
    def _compute_provision(bucket_totals: Dict[str, float]) -> Dict[str, Any]:
        """Compute provision for bad debts based on aging."""
        total_provision = 0.0
        details: List[Dict[str, Any]] = []
        for name, _, _ in AGING_BUCKETS:
            amount = bucket_totals.get(name, 0)
            rate = PROVISION_RATES.get(name, 0)
            provision = _money_round(amount * rate)
            total_provision += provision
            details.append({"bucket": name, "amount": amount, "rate": rate, "provision": provision})

        return {"total_provision": _money_round(total_provision), "details": details}

    def health_check(self) -> bool:
        return True
