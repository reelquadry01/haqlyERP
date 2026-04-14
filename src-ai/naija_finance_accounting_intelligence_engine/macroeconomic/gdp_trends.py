# Author: Quadri Atharu
"""Nigerian GDP trends and sector breakdown engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class GdpTrendsEngine:
    """Nigerian GDP trends and sector breakdown engine."""

    def analyze_gdp(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze GDP data and trends."""
        historical: List[Dict[str, Any]] = data.get("gdp_history", [])

        if not historical:
            return {"message": "No GDP data provided"}

        growth_rates = []
        for i in range(1, len(historical)):
            prev = float(historical[i - 1].get("gdp", 0))
            curr = float(historical[i].get("gdp", 0))
            if prev > 0:
                growth_rates.append(round((curr - prev) / prev, 4))

        avg_growth = round(sum(growth_rates) / len(growth_rates), 4) if growth_rates else 0

        return {
            "gdp_data": historical,
            "latest_gdp": float(historical[-1].get("gdp", 0)) if historical else 0,
            "average_growth_rate": avg_growth,
            "growth_rates": growth_rates,
        }

    def sector_breakdown(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze GDP by sector."""
        sectors: Dict[str, float] = data.get("sectors", {})
        total = sum(sectors.values())

        if total <= 0:
            return {"message": "No sector data provided"}

        breakdown = {k: {"value": round(v, 2), "pct": round(v / total, 4)} for k, v in sectors.items()}
        oil_pct = round(sectors.get("oil_gas", 0) / total, 4)
        non_oil_pct = 1 - oil_pct

        return {
            "sectors": breakdown,
            "total_gdp": round(total, 2),
            "oil_sector_pct": oil_pct,
            "non_oil_sector_pct": round(non_oil_pct, 4),
            "largest_sector": max(sectors, key=sectors.get) if sectors else None,
        }

    def health_check(self) -> bool:
        return True
