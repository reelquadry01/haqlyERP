# Author: Quadri Atharu
"""Liquidity risk stress testing engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class LiquidityRiskEngine:
    """Liquidity risk stress testing engine."""

    def stress_test_liquidity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Stress test liquidity position under various scenarios."""
        current_cash = float(data.get("current_cash", 0))
        monthly_burn_rate = float(data.get("monthly_burn_rate", 0))
        committed_outflows = float(data.get("committed_outflows", 0))
        undrawn_facilities = float(data.get("undrawn_facilities", 0))
        liquid_assets = float(data.get("liquid_assets", 0))

        total_liquid = round(current_cash + liquid_assets + undrawn_facilities, 2)
        total_obligations = round(monthly_burn_rate + committed_outflows, 2)

        scenarios = {
            "normal": {"cash_shock": 0, "burn_shock": 0, "facility_shock": 0},
            "moderate": {"cash_shock": -0.10, "burn_shock": 0.20, "facility_shock": -0.10},
            "severe": {"cash_shock": -0.20, "burn_shock": 0.50, "facility_shock": -0.30},
            "extreme": {"cash_shock": -0.30, "burn_shock": 0.80, "facility_shock": -0.50},
        }

        results: Dict[str, Any] = {}
        for name, shock in scenarios.items():
            stressed_cash = round(current_cash * (1 + shock["cash_shock"]), 2)
            stressed_burn = round(monthly_burn_rate * (1 + shock["burn_shock"]), 2)
            stressed_facilities = round(undrawn_facilities * (1 + shock["facility_shock"]), 2)
            stressed_liquid = round(stressed_cash + liquid_assets + stressed_facilities, 2)
            months_survival = round(stressed_liquid / stressed_burn, 2) if stressed_burn > 0 else 999
            lcr = round(stressed_liquid / (stressed_burn + committed_outflows), 4) if (stressed_burn + committed_outflows) > 0 else 999

            results[name] = {
                "stressed_cash": stressed_cash,
                "stressed_burn_rate": stressed_burn,
                "stressed_liquid_resources": stressed_liquid,
                "months_of_survival": months_survival,
                "liquidity_coverage_ratio": lcr,
                "passes_lcr_threshold": lcr >= 1.0,
            }

        return {
            "current_position": {
                "total_liquid_resources": total_liquid,
                "monthly_obligations": total_obligations,
                "months_of_survival": round(total_liquid / total_obligations, 2) if total_obligations > 0 else 999,
            },
            "stress_scenarios": results,
            "overall_risk": self._assess_risk(results),
            "computed_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _assess_risk(results: Dict[str, Any]) -> str:
        """Assess overall liquidity risk from stress test results."""
        severe = results.get("severe", {})
        months = severe.get("months_of_survival", 0)
        if months < 3:
            return "critical"
        elif months < 6:
            return "high"
        elif months < 12:
            return "moderate"
        return "low"

    def health_check(self) -> bool:
        return True
