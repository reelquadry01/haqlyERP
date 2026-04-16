# Author: Quadri Atharu
"""Net Present Value (NPV) computation engine."""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal, ROUND_HALF_UP
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


class NpvEngine:
    """Net Present Value computation engine."""

    def compute_npv(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute NPV given cash flows and discount rate."""
        cash_flows = [Decimal(str(cf)) for cf in data.get("cash_flows", [])]
        discount_rate = float(data.get("discount_rate", 0.10))
        initial_investment = Decimal(str(data.get("initial_investment", 0)))

        if not cash_flows:
            return {"npv": None, "message": "No cash flows provided"}

        pv_of_flows = Decimal('0')
        pv_details: List[Dict[str, Any]] = []

        for i, cf in enumerate(cash_flows):
            discount_factor = Decimal(str((1 + discount_rate) ** (i + 1)))
            pv = _money_round(cf / discount_factor)
            pv_of_flows += pv
            pv_details.append({"period": i + 1, "cash_flow": float(cf), "present_value": pv, "discount_factor": round(1 / (1 + discount_rate) ** (i + 1), 6)})

        npv = _money_round(pv_of_flows - initial_investment)

        return {
            "initial_investment": _money_round(initial_investment),
            "discount_rate": discount_rate,
            "total_periods": len(cash_flows),
            "pv_of_cash_flows": _money_round(pv_of_flows),
            "npv": npv,
            "recommendation": "ACCEPT — NPV is positive" if npv > 0 else "REJECT — NPV is negative",
            "profitability_index": round(float(pv_of_flows) / float(initial_investment), 4) if initial_investment > 0 else None,
            "pv_details": pv_details,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_npv_sensitivity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute NPV sensitivity to discount rate changes."""
        cash_flows = [Decimal(str(cf)) for cf in data.get("cash_flows", [])]
        initial_investment = Decimal(str(data.get("initial_investment", 0)))
        base_rate = float(data.get("discount_rate", 0.10))
        rate_range = float(data.get("rate_range", 0.05))

        sensitivities: List[Dict[str, Any]] = []
        steps = 11
        for step in range(steps):
            rate = round(base_rate - rate_range + (2 * rate_range * step / (steps - 1)), 6)
            pv = Decimal('0')
            for i, cf in enumerate(cash_flows):
                pv += cf / Decimal(str((1 + rate) ** (i + 1)))
            npv = _money_round(pv - initial_investment)
            sensitivities.append({"discount_rate": rate, "npv": npv})

        return {
            "base_rate": base_rate,
            "rate_range": rate_range,
            "sensitivities": sensitivities,
            "break_even_rate_approx": self._approximate_break_even(sensitivities),
        }

    @staticmethod
    def _approximate_break_even(sensitivities: List[Dict[str, Any]]) -> float | None:
        """Approximate the break-even discount rate where NPV = 0."""
        for i in range(len(sensitivities) - 1):
            npv1 = float(sensitivities[i]["npv"])
            npv2 = float(sensitivities[i + 1]["npv"])
            if (npv1 >= 0 and npv2 < 0) or (npv1 <= 0 and npv2 > 0):
                r1 = sensitivities[i]["discount_rate"]
                r2 = sensitivities[i + 1]["discount_rate"]
                return round(r1 + (0 - npv1) * (r2 - r1) / (npv2 - npv1), 6)
        return None

    def health_check(self) -> bool:
        return True
