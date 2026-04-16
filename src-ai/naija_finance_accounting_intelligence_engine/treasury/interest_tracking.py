# Author: Quadri Atharu
"""Interest tracking and accrual engine."""

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


class InterestTrackingEngine:
    """Interest accrual tracking engine for loans, deposits, and overdrafts."""

    def compute_interest_accrual(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute interest accrual for a period."""
        principal = Decimal(str(data.get("principal", 0)))
        annual_rate = Decimal(str(data.get("annual_rate", 0)))
        days_in_period = int(data.get("days_in_period", 30))
        day_count_convention = data.get("day_count_convention", "actual_365").lower()
        interest_type = data.get("interest_type", "simple").lower()

        if principal <= 0:
            return {"interest_accrued": 0.0, "message": "No principal to accrue interest on"}

        if day_count_convention == "actual_360":
            daily_rate = annual_rate / 360
        elif day_count_convention == "actual_365":
            daily_rate = annual_rate / 365
        elif day_count_convention == "30_360":
            daily_rate = annual_rate / 360
            days_in_period = 30
        else:
            daily_rate = annual_rate / 365

        if interest_type == "simple":
            interest = _money_round(principal * daily_rate * days_in_period)
        elif interest_type == "compound":
            interest = _money_round(principal * ((1 + daily_rate) ** days_in_period - 1))
        else:
            interest = _money_round(principal * daily_rate * days_in_period)

        return {
            "principal": _money_round(principal),
            "annual_rate": annual_rate,
            "daily_rate": round(daily_rate, 8),
            "days_in_period": days_in_period,
            "day_count_convention": day_count_convention,
            "interest_type": interest_type,
            "interest_accrued": interest,
            "period_start": data.get("period_start", ""),
            "period_end": data.get("period_end", ""),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_compound_interest_schedule(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a compound interest accrual schedule over multiple periods."""
        principal = Decimal(str(data.get("principal", 0)))
        annual_rate = Decimal(str(data.get("annual_rate", 0)))
        periods = int(data.get("periods", 12))
        compounding_frequency = data.get("compounding_frequency", "monthly").lower()

        freq_map = {"daily": 365, "monthly": 12, "quarterly": 4, "semi_annually": 2, "annually": 1}
        n = freq_map.get(compounding_frequency, 12)
        periodic_rate = annual_rate / n

        schedule: List[Dict[str, Any]] = []
        balance = principal

        for p in range(1, periods + 1):
            interest = _money_round(balance * periodic_rate)
            balance = _money_round(balance + interest)
            schedule.append({
                "period": p,
                "opening_balance": _money_round(balance - interest),
                "interest_accrued": interest,
                "closing_balance": balance,
            })

        total_interest = _money_round(sum(s["interest_accrued"] for s in schedule))

        return {
            "principal": _money_round(principal),
            "annual_rate": annual_rate,
            "compounding_frequency": compounding_frequency,
            "periodic_rate": round(periodic_rate, 8),
            "total_periods": periods,
            "total_interest_accrued": total_interest,
            "maturity_amount": _money_round(principal + total_interest),
            "effective_annual_rate": round((1 + periodic_rate) ** n - 1, 6),
            "schedule": schedule,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_overdraft_interest(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute interest on overdraft facility based on daily balances."""
        daily_balances: List[Dict[str, Any]] = data.get("daily_balances", [])
        annual_rate = Decimal(str(data.get("annual_rate", 0.25)))
        facility_limit = Decimal(str(data.get("facility_limit", 0)))

        if not daily_balances:
            return {"interest_accrued": 0.0, "message": "No daily balances provided"}

        daily_rate = annual_rate / 365
        total_interest = 0.0
        over_limit_days = 0
        details: List[Dict[str, Any]] = []

        for db in daily_balances:
            balance = float(db.get("balance", 0))
            if balance > 0:
                continue
            utilized = abs(balance)
            day_interest = _money_round(utilized * daily_rate)
            total_interest += day_interest

            if facility_limit > 0 and utilized > facility_limit:
                over_limit_days += 1

            details.append({
                "date": db.get("date", ""),
                "utilized_amount": _money_round(utilized),
                "daily_interest": day_interest,
            })

        return {
            "annual_rate": annual_rate,
            "daily_rate": round(daily_rate, 8),
            "total_interest_accrued": _money_round(total_interest),
            "over_limit_days": over_limit_days,
            "facility_limit": _money_round(facility_limit),
            "penalty_note": "Penalty interest may apply for over-limit usage" if over_limit_days > 0 else None,
            "daily_details": details,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_deposit_interest(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute interest earned on fixed deposits."""
        principal = Decimal(str(data.get("principal", 0)))
        annual_rate = Decimal(str(data.get("annual_rate", 0.10)))
        term_days = int(data.get("term_days", 365))
        withholding_tax_rate = Decimal(str(data.get("wht_rate", 0.10)))

        interest_gross = _money_round(principal * annual_rate * term_days / 365)
        wht_amount = _money_round(interest_gross * withholding_tax_rate)
        interest_net = _money_round(interest_gross - wht_amount)

        return {
            "principal": _money_round(principal),
            "annual_rate": annual_rate,
            "term_days": term_days,
            "interest_gross": interest_gross,
            "wht_on_interest": wht_amount,
            "wht_rate": withholding_tax_rate,
            "interest_net_of_wht": interest_net,
            "maturity_amount": _money_round(principal + interest_gross),
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
