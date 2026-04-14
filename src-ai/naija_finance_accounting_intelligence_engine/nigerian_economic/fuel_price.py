# Author: Quadri Atharu
"""Nigerian fuel price analysis and impact engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class FuelPriceEngine:
    """Nigerian fuel price analysis and business impact engine."""

    def analyze_fuel_impact(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze the impact of fuel price changes on business operations."""
        current_price = float(data.get("current_price", 620))
        previous_price = float(data.get("previous_price", 195))
        fuel_consumption_monthly = float(data.get("fuel_consumption_monthly", 1000))
        transport_cost_pct = float(data.get("transport_cost_pct_of_cogs", 0.15))
        cogs = float(data.get("cogs", 0))

        price_change = round(current_price - previous_price, 2)
        price_change_pct = round(price_change / previous_price, 4) if previous_price > 0 else 0

        monthly_cost_increase = round(fuel_consumption_monthly * price_change, 2)
        annual_cost_increase = round(monthly_cost_increase * 12, 2)
        cogs_impact = round(cogs * transport_cost_pct * price_change_pct, 2) if cogs > 0 else 0

        return {
            "current_price": current_price,
            "previous_price": previous_price,
            "price_change": price_change,
            "price_change_pct": price_change_pct,
            "monthly_fuel_cost_increase": monthly_cost_increase,
            "annual_fuel_cost_increase": annual_cost_increase,
            "estimated_cogs_impact": cogs_impact,
            "recommendation": "Review pricing strategy and consider fuel surcharges" if price_change_pct > 0.5 else "Monitor fuel price trends",
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
