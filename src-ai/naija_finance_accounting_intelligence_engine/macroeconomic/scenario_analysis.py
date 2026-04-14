# Author: Quadri Atharu
"""Scenario analysis engine — bull, base, and bear scenarios."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class ScenarioAnalysisEngine:
    """Scenario analysis engine for financial planning."""

    def run_scenario_analysis(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Run bull/base/bear scenario analysis on financial projections."""
        base_financials: Dict[str, float] = data.get("base_financials", {})
        shocks: Dict[str, Dict[str, float]] = data.get("shocks", {})

        default_shocks = {
            "bull": {"revenue_growth": 0.15, "cost_inflation": 0.05, "fx_impact": 0.02, "interest_rate_delta": -0.02},
            "base": {"revenue_growth": 0.08, "cost_inflation": 0.10, "fx_impact": 0.05, "interest_rate_delta": 0.00},
            "bear": {"revenue_growth": -0.05, "cost_inflation": 0.20, "fx_impact": 0.15, "interest_rate_delta": 0.03},
        }

        if not shocks:
            shocks = default_shocks

        scenarios: Dict[str, Dict[str, Any]] = {}

        for scenario_name, shock_params in shocks.items():
            revenue = float(base_financials.get("revenue", 0)) * (1 + shock_params.get("revenue_growth", 0))
            cogs = float(base_financials.get("cogs", 0)) * (1 + shock_params.get("cost_inflation", 0))
            gross_profit = round(revenue - cogs, 2)
            opex = float(base_financials.get("opex", 0)) * (1 + shock_params.get("cost_inflation", 0) * 0.7)
            ebit = round(gross_profit - opex, 2)
            interest = float(base_financials.get("interest", 0)) * (1 + shock_params.get("interest_rate_delta", 0))
            fx_impact_amount = round(float(base_financials.get("revenue", 0)) * shock_params.get("fx_impact", 0), 2)
            net_income = round(ebit - interest - fx_impact_amount, 2)

            scenarios[scenario_name] = {
                "revenue": round(revenue, 2),
                "cogs": round(cogs, 2),
                "gross_profit": gross_profit,
                "opex": round(opex, 2),
                "ebit": ebit,
                "interest": round(interest, 2),
                "fx_impact": fx_impact_amount,
                "net_income": net_income,
                "shock_parameters": shock_params,
            }

        base_net = scenarios.get("base", {}).get("net_income", 0)
        bull_net = scenarios.get("bull", {}).get("net_income", 0)
        bear_net = scenarios.get("bear", {}).get("net_income", 0)

        return {
            "scenarios": scenarios,
            "range": {
                "best_case_net_income": bull_net,
                "base_case_net_income": base_net,
                "worst_case_net_income": bear_net,
                "upside_vs_base": round(bull_net - base_net, 2),
                "downside_vs_base": round(bear_net - base_net, 2),
            },
            "risk_reward_ratio": round(abs(bull_net - base_net) / abs(bear_net - base_net), 2) if bear_net != base_net else None,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
