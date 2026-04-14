# Author: Quadri Atharu
"""Net Present Value (NPV) computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class NpvEngine:
    """Net Present Value computation engine."""

    def compute_npv(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute NPV given cash flows and discount rate."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        discount_rate = float(data.get("discount_rate", 0.10))
        initial_investment = float(data.get("initial_investment", 0))

        if not cash_flows:
            return {"npv": None, "message": "No cash flows provided"}

        pv_of_flows = 0.0
        pv_details: List[Dict[str, Any]] = []

        for i, cf in enumerate(cash_flows):
            pv = round(cf / (1 + discount_rate) ** (i + 1), 2)
            pv_of_flows += pv
            pv_details.append({"period": i + 1, "cash_flow": cf, "present_value": pv, "discount_factor": round(1 / (1 + discount_rate) ** (i + 1), 6)})

        npv = round(pv_of_flows - initial_investment, 2)

        return {
            "initial_investment": round(initial_investment, 2),
            "discount_rate": discount_rate,
            "total_periods": len(cash_flows),
            "pv_of_cash_flows": round(pv_of_flows, 2),
            "npv": npv,
            "recommendation": "ACCEPT — NPV is positive" if npv > 0 else "REJECT — NPV is negative",
            "profitability_index": round(pv_of_flows / initial_investment, 4) if initial_investment > 0 else None,
            "pv_details": pv_details,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_npv_sensitivity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute NPV sensitivity to discount rate changes."""
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        initial_investment = float(data.get("initial_investment", 0))
        base_rate = float(data.get("discount_rate", 0.10))
        rate_range = float(data.get("rate_range", 0.05))

        sensitivities: List[Dict[str, Any]] = []
        steps = 11
        for step in range(steps):
            rate = round(base_rate - rate_range + (2 * rate_range * step / (steps - 1)), 6)
            pv = sum(cf / (1 + rate) ** (i + 1) for i, cf in enumerate(cash_flows))
            npv = round(pv - initial_investment, 2)
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
            npv1 = sensitivities[i]["npv"]
            npv2 = sensitivities[i + 1]["npv"]
            if (npv1 >= 0 and npv2 < 0) or (npv1 <= 0 and npv2 > 0):
                r1 = sensitivities[i]["discount_rate"]
                r2 = sensitivities[i + 1]["discount_rate"]
                return round(r1 + (0 - npv1) * (r2 - r1) / (npv2 - npv1), 6)
        return None

    def health_check(self) -> bool:
        return True
