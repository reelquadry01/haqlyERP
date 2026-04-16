# Author: Quadri Atharu
"""Weighted Average Cost of Capital (WACC) computation engine."""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal, ROUND_HALF_UP
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


class WaccEngine:
    """Weighted Average Cost of Capital computation engine."""

    def compute_wacc(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute WACC from cost of equity, cost of debt, and capital structure."""
        market_cap = Decimal(str(data.get("market_cap", 0)))
        total_debt = Decimal(str(data.get("total_debt", 0)))
        cost_of_equity = float(data.get("cost_of_equity", 0))
        cost_of_debt_pretax = float(data.get("cost_of_debt_pretax", 0))
        tax_rate = float(data.get("tax_rate", 0.25))
        cash = Decimal(str(data.get("cash", 0)))

        enterprise_value = market_cap + total_debt - cash
        equity_weight = round(float(market_cap) / float(enterprise_value), 4) if enterprise_value > 0 else 0
        debt_weight = round(float(total_debt) / float(enterprise_value), 4) if enterprise_value > 0 else 0

        cost_of_debt_aftertax = round(cost_of_debt_pretax * (1 - tax_rate), 6)
        wacc = round(equity_weight * cost_of_equity + debt_weight * cost_of_debt_aftertax, 6)

        return {
            "market_cap": _money_round(market_cap),
            "total_debt": _money_round(total_debt),
            "cash": _money_round(cash),
            "enterprise_value": _money_round(enterprise_value),
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
