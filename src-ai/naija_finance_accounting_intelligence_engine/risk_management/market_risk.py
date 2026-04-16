# Author: Quadri Atharu
"""Market risk engine — VaR and CVaR computation."""

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


class MarketRiskEngine:
    """Market risk engine with Value at Risk and Conditional VaR."""

    def compute_var(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Value at Risk (VaR) using parametric method."""
        portfolio_value = Decimal(str(data.get("portfolio_value", 0)))
        daily_volatility = float(data.get("daily_volatility", 0.02))
        confidence_level = float(data.get("confidence_level", 0.95))
        horizon_days = int(data.get("horizon_days", 1))

        z_scores = {0.90: 1.28, 0.95: 1.645, 0.99: 2.33}
        z = z_scores.get(confidence_level, 1.645)

        var_daily = _money_round(portfolio_value * Decimal(str(daily_volatility)) * Decimal(str(z)))
        var_horizon = _money_round(var_daily * Decimal(str(horizon_days ** 0.5)))

        return {
            "portfolio_value": _money_round(portfolio_value),
            "daily_volatility": daily_volatility,
            "confidence_level": confidence_level,
            "z_score": z,
            "horizon_days": horizon_days,
            "var_daily": var_daily,
            "var_horizon": var_horizon,
            "var_pct_of_portfolio": round(float(var_horizon) / float(portfolio_value), 4) if portfolio_value > 0 else 0,
            "method": "parametric",
        }

    def compute_historical_var(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute VaR using historical simulation method."""
        returns: List[float] = [float(r) for r in data.get("returns", [])]
        portfolio_value = Decimal(str(data.get("portfolio_value", 0)))
        confidence_level = float(data.get("confidence_level", 0.95))
        horizon_days = int(data.get("horizon_days", 1))

        if len(returns) < 30:
            return {"var": None, "message": "At least 30 return observations required"}

        sorted_returns = sorted(returns)
        idx = int(len(sorted_returns) * (1 - confidence_level))
        var_return = sorted_returns[idx]
        var_amount = _money_round(abs(Decimal(str(var_return)) * portfolio_value))
        var_horizon = _money_round(var_amount * Decimal(str(horizon_days ** 0.5)))

        cvar_returns = sorted_returns[:idx + 1]
        cvar_return = round(sum(cvar_returns) / len(cvar_returns), 6) if cvar_returns else 0
        cvar_amount = _money_round(abs(Decimal(str(cvar_return)) * portfolio_value))

        return {
            "method": "historical_simulation",
            "portfolio_value": _money_round(portfolio_value),
            "confidence_level": confidence_level,
            "horizon_days": horizon_days,
            "var_daily": var_amount,
            "var_horizon": var_horizon,
            "cvar_daily": cvar_amount,
            "observations": len(returns),
        }

    def compute_cvar(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Conditional VaR (Expected Shortfall)."""
        var_result = self.compute_var(data)
        var = var_result.get("var_daily", 0)
        portfolio_value = Decimal(str(data.get("portfolio_value", 0)))
        daily_vol = float(data.get("daily_volatility", 0.02))
        confidence = float(data.get("confidence_level", 0.95))

        z_scores = {0.90: 1.28, 0.95: 1.645, 0.99: 2.33}
        z = z_scores.get(confidence, 1.645)

        from math import exp, pi
        if confidence > 0 and z > 0:
            pdf_z = round(exp(-z ** 2 / 2) / (2 * pi) ** 0.5, 6)
            cvar_factor = pdf_z / (1 - confidence)
        else:
            cvar_factor = 1.5

        cvar = _money_round(portfolio_value * Decimal(str(daily_vol * cvar_factor))

        return {
            "var": var,
            "cvar": cvar,
            "cvar_var_ratio": round(float(cvar) / float(var), 4) if float(var) > 0 else None,
            "confidence_level": confidence,
        }

    def health_check(self) -> bool:
        return True
