# Author: Quadri Atharu
"""Liquidity risk stress testing engine."""

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


class LiquidityRiskEngine:
    """Liquidity risk stress testing engine."""

    def stress_test_liquidity(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Stress test liquidity position under various scenarios."""
        current_cash = Decimal(str(data.get("current_cash", 0)))
        monthly_burn_rate = Decimal(str(data.get("monthly_burn_rate", 0)))
        committed_outflows = Decimal(str(data.get("committed_outflows", 0)))
        undrawn_facilities = Decimal(str(data.get("undrawn_facilities", 0)))
        liquid_assets = Decimal(str(data.get("liquid_assets", 0)))

        total_liquid = _money_round(current_cash + liquid_assets + undrawn_facilities)
        total_obligations = _money_round(monthly_burn_rate + committed_outflows)

        scenarios = {
            "normal": {"cash_shock": 0, "burn_shock": 0, "facility_shock": 0},
            "moderate": {"cash_shock": -0.10, "burn_shock": 0.20, "facility_shock": -0.10},
            "severe": {"cash_shock": -0.20, "burn_shock": 0.50, "facility_shock": -0.30},
            "extreme": {"cash_shock": -0.30, "burn_shock": 0.80, "facility_shock": -0.50},
        }

        results: Dict[str, Any] = {}
        for name, shock in scenarios.items():
            stressed_cash = _money_round(current_cash * (1 + shock["cash_shock"]))
            stressed_burn = _money_round(monthly_burn_rate * (1 + shock["burn_shock"]))
            stressed_facilities = _money_round(undrawn_facilities * (1 + shock["facility_shock"]))
            stressed_liquid = _money_round(stressed_cash + liquid_assets + stressed_facilities)
            months_survival = _money_round(stressed_liquid / stressed_burn) if stressed_burn > 0 else 999
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
                "months_of_survival": _money_round(total_liquid / total_obligations) if total_obligations > 0 else 999,
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
