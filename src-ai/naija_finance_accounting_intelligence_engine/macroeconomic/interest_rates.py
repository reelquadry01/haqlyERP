# Author: Quadri Atharu
"""Nigerian interest rate data engine — MPR, Treasury Bill rates."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class InterestRatesEngine:
    """Nigerian interest rate data and analysis engine."""

    def get_mpr_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Retrieve Monetary Policy Rate (MPR) data."""
        historical: List[Dict[str, Any]] = data.get("mpr_history", [])

        if not historical:
            return {"message": "No MPR history provided", "current_mpr": None}

        rates = [float(h.get("rate", 0)) for h in historical]
        latest = rates[-1] if rates else None

        return {
            "current_mpr": latest,
            "mpr_history": historical,
            "average_mpr": round(sum(rates) / len(rates), 2) if rates else None,
            "trend": "tightening" if len(rates) >= 2 and rates[-1] > rates[-2] else "easing",
        }

    def get_tbill_rates(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Retrieve Treasury Bill rates across tenors."""
        rates: Dict[str, float] = data.get("tbill_rates", {})

        return {
            "tbill_91_day": rates.get("91_day", 0),
            "tbill_182_day": rates.get("182_day", 0),
            "tbill_364_day": rates.get("364_day", 0),
            "yield_curve_slope": round(rates.get("364_day", 0) - rates.get("91_day", 0), 4) if rates.get("91_day") else None,
            "normal_yield_curve": rates.get("364_day", 0) > rates.get("91_day", 0) if rates else None,
        }

    def compute_real_interest_rate(self, nominal_rate: float, inflation_rate: float) -> Dict[str, Any]:
        """Compute real interest rate using Fisher equation."""
        real_rate = round((1 + nominal_rate / 100) / (1 + inflation_rate / 100) - 1, 6)
        return {
            "nominal_rate_pct": nominal_rate,
            "inflation_rate_pct": inflation_rate,
            "real_rate": real_rate,
            "real_rate_pct": f"{real_rate * 100:.2f}%",
            "positive_real_rate": real_rate > 0,
        }

    def health_check(self) -> bool:
        return True
