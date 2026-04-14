# Author: Quadri Atharu
"""Nigerian inflation rate data and projection engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class InflationRatesEngine:
    """Nigerian inflation rate data and projection engine."""

    def get_inflation_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Retrieve and analyze inflation rate data."""
        historical: List[Dict[str, Any]] = data.get("historical_rates", [])
        base_year = int(data.get("base_year", 2024))

        if not historical:
            return {"message": "No historical inflation data provided"}

        rates = [float(h.get("rate", 0)) for h in historical]
        avg_rate = round(sum(rates) / len(rates), 2) if rates else 0
        max_rate = max(rates) if rates else 0
        min_rate = min(rates) if rates else 0
        latest_rate = rates[-1] if rates else 0

        return {
            "base_year": base_year,
            "historical_rates": historical,
            "latest_rate": latest_rate,
            "average_rate": avg_rate,
            "max_rate": max_rate,
            "min_rate": min_rate,
            "trend": "increasing" if len(rates) >= 2 and rates[-1] > rates[-2] else "decreasing",
        }

    def project_inflation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Project future inflation rates."""
        current_rate = float(data.get("current_rate", 28.0))
        target_rate = float(data.get("target_rate", 15.0))
        projection_years = int(data.get("projection_years", 5))
        convergence_speed = float(data.get("convergence_speed", 0.3))

        projections: List[Dict[str, Any]] = []
        rate = current_rate

        for year in range(1, projection_years + 1):
            projected = rate + (target_rate - rate) * convergence_speed
            rate = round(projected, 2)
            projections.append({"year": year, "projected_rate": rate})

        return {
            "current_rate": current_rate,
            "target_rate": target_rate,
            "convergence_speed": convergence_speed,
            "projections": projections,
        }

    def adjust_for_inflation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Adjust financial figures for inflation."""
        nominal_amount = float(data.get("nominal_amount", 0))
        inflation_rate = float(data.get("inflation_rate", 0))
        periods = int(data.get("periods", 1))

        real_amount = round(nominal_amount / (1 + inflation_rate) ** periods, 2) if inflation_rate != 0 else nominal_amount
        inflation_premium = round(nominal_amount - real_amount, 2)

        return {
            "nominal_amount": round(nominal_amount, 2),
            "inflation_rate": inflation_rate,
            "periods": periods,
            "real_amount": real_amount,
            "inflation_premium": inflation_premium,
        }

    def health_check(self) -> bool:
        return True
