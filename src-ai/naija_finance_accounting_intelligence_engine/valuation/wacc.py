# Author: Quadri Atharu
"""Weighted Average Cost of Capital (WACC) computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class WaccEngine:
    """Weighted Average Cost of Capital computation engine."""

    def compute_wacc(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute WACC from cost of equity, cost of debt, and capital structure."""
        market_cap = float(data.get("market_cap", 0))
        total_debt = float(data.get("total_debt", 0))
        cost_of_equity = float(data.get("cost_of_equity", 0))
        cost_of_debt_pretax = float(data.get("cost_of_debt_pretax", 0))
        tax_rate = float(data.get("tax_rate", 0.30))
        cash = float(data.get("cash", 0))

        enterprise_value = market_cap + total_debt - cash
        equity_weight = round(market_cap / enterprise_value, 4) if enterprise_value > 0 else 0
        debt_weight = round(total_debt / enterprise_value, 4) if enterprise_value > 0 else 0

        cost_of_debt_aftertax = round(cost_of_debt_pretax * (1 - tax_rate), 6)
        wacc = round(equity_weight * cost_of_equity + debt_weight * cost_of_debt_aftertax, 6)

        return {
            "market_cap": round(market_cap, 2),
            "total_debt": round(total_debt, 2),
            "cash": round(cash, 2),
            "enterprise_value": round(enterprise_value, 2),
            "equity_weight": equity_weight,
            "debt_weight": debt_weight,
            "cost_of_equity": cost_of_equity,
            "cost_of_debt_pretax": cost_of_debt_pretax,
            "tax_rate": tax_rate,
            "cost_of_debt_aftertax": cost_of_debt_aftertax,
            "wacc": wacc,
            "wacc_pct": f"{wacc * 100:.2f}%",
            "computed_at": datetime.now().isoformat(),
        }

    def compute_cost_of_equity_capm(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute cost of equity using Capital Asset Pricing Model."""
        risk_free_rate = float(data.get("risk_free_rate", 0.15))
        beta = float(data.get("beta", 1.0))
        market_return = float(data.get("market_return", 0.20))
        size_premium = float(data.get("size_premium", 0.02))
        country_risk_premium = float(data.get("country_risk_premium", 0.04))

        equity_risk_premium = round(market_return - risk_free_rate, 4)
        cost_of_equity = round(risk_free_rate + beta * equity_risk_premium + size_premium + country_risk_premium, 6)

        return {
            "model": "CAPM + Adjustments",
            "risk_free_rate": risk_free_rate,
            "beta": beta,
            "market_return": market_return,
            "equity_risk_premium": equity_risk_premium,
            "size_premium": size_premium,
            "country_risk_premium": country_risk_premium,
            "cost_of_equity": cost_of_equity,
            "cost_of_equity_pct": f"{cost_of_equity * 100:.2f}%",
            "note": "Nigerian risk-free rate typically based on 10-year FGN bond yield",
        }

    def health_check(self) -> bool:
        return True
