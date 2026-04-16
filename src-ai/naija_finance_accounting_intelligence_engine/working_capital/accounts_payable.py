# Author: Quadri Atharu
"""Accounts Payable aging and DPO computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

AP_AGING_BUCKETS = [
    ("current", 0, 30),
    ("31_60_days", 31, 60),
    ("61_90_days", 61, 90),
    ("91_180_days", 91, 180),
    ("over_180_days", 181, 99999),
]


class AccountsPayableEngine:
    """Accounts Payable aging and DPO computation engine."""

    def compute_aging(self, payables: List[Dict[str, Any]], as_of_date: str = "") -> Dict[str, Any]:
        """Compute AP aging report."""
        try:
            ref_date = datetime.fromisoformat(as_of_date) if as_of_date else datetime.now()
        except (ValueError, TypeError):
            ref_date = datetime.now()

        buckets: Dict[str, List[Dict[str, Any]]] = {name: [] for name, _, _ in AP_AGING_BUCKETS}
        bucket_totals: Dict[str, float] = {name: 0.0 for name, _, _ in AP_AGING_BUCKETS}

        total_ap = 0.0
        total_overdue = 0.0

        for pay in payables:
            amount = float(pay.get("amount", 0))
            due_date_str = pay.get("due_date", "")
            try:
                due_date = datetime.fromisoformat(due_date_str) if due_date_str else ref_date
            except (ValueError, TypeError):
                due_date = ref_date

            days_overdue = max((ref_date - due_date).days, 0)
            total_ap += amount

            for name, low, high in AP_AGING_BUCKETS:
                if low <= days_overdue <= high:
                    buckets[name].append({
                        "vendor": pay.get("vendor", ""),
                        "invoice_id": pay.get("id", ""),
                        "amount": _money_round(amount),
                        "due_date": due_date_str,
                        "days_overdue": days_overdue,
                    })
                    bucket_totals[name] = _money_round(bucket_totals[name] + amount)
                    break

            if days_overdue > 0:
                total_overdue += amount

        return {
            "as_of_date": ref_date.isoformat(),
            "total_ap": _money_round(total_ap),
            "total_overdue": _money_round(total_overdue),
            "overdue_pct": round(total_overdue / total_ap, 4) if total_ap > 0 else 0,
            "aging_buckets": {name: {"total": bucket_totals[name], "count": len(buckets[name]), "items": buckets[name]} for name, _, _ in AP_AGING_BUCKETS},
            "bucket_totals": bucket_totals,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dpo(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Days Payable Outstanding (DPO)."""
        accounts_payable = Decimal(str(data.get("accounts_payable", 0)))
        annual_cogs = Decimal(str(data.get("annual_cogs", 0)))
        period_days = int(data.get("period_days", 365))

        if annual_cogs <= 0:
            return {"dpo": None, "message": "COGS must be positive for DPO computation"}

        dpo = _money_round(accounts_payable / (annual_cogs / period_days))

        if dpo <= 30:
            health = "fast_payment"
        elif dpo <= 45:
            health = "moderate"
        elif dpo <= 60:
            health = "good_utilization"
        elif dpo <= 90:
            health = "stretching"
        else:
            health = "potential_risk"

        return {
            "accounts_payable": _money_round(accounts_payable),
            "annual_cogs": _money_round(annual_cogs),
            "dpo": dpo,
            "health": health,
            "recommendation": "Optimize payment timing to maximize float" if dpo < 30 else ("Risk of vendor disputes — consider accelerating payments" if dpo > 90 else "DPO within acceptable range"),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_payment_priority(self, payables: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Rank payables by payment priority based on due date, amount, and vendor importance."""
        scored: List[Dict[str, Any]] = []

        for pay in payables:
            amount = float(pay.get("amount", 0))
            due_date_str = pay.get("due_date", "")
            vendor_criticality = int(pay.get("vendor_criticality", 5))
            early_discount_pct = float(pay.get("early_discount_pct", 0))

            try:
                due_date = datetime.fromisoformat(due_date_str) if due_date_str else datetime.now()
                days_until_due = max((due_date - datetime.now()).days, 0)
            except (ValueError, TypeError):
                days_until_due = 30

            urgency_score = max(100 - days_until_due, 0) + vendor_criticality * 5 + (20 if early_discount_pct > 0 else 0)
            discount_value = _money_round(amount * early_discount_pct) if early_discount_pct > 0 else 0

            scored.append({
                "vendor": pay.get("vendor", ""),
                "invoice_id": pay.get("id", ""),
                "amount": _money_round(amount),
                "due_date": due_date_str,
                "days_until_due": days_until_due,
                "urgency_score": urgency_score,
                "early_discount_available": early_discount_pct > 0,
                "discount_value": discount_value,
            })

        scored.sort(key=lambda x: x["urgency_score"], reverse=True)

        return {
            "total_payables": _money_round(sum(s["amount"] for s in scored)),
            "payment_priority_list": scored,
            "early_discount_opportunities": _money_round(sum(s["discount_value"] for s in scored if s["early_discount_available"])),
        }

    def health_check(self) -> bool:
        return True
