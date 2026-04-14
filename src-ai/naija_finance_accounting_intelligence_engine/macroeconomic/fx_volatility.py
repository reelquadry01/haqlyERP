# Author: Quadri Atharu
"""FX volatility analysis and forecast engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class FxVolatilityEngine:
    """FX volatility analysis and forecast engine for NGN."""

    def compute_volatility(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute FX rate volatility from historical rates."""
        rates: List[float] = [float(r) for r in data.get("rates", [])]
        base_currency = data.get("base_currency", "NGN")
        quote_currency = data.get("quote_currency", "USD")

        if len(rates) < 2:
            return {"volatility": None, "message": "At least 2 rates required"}

        log_returns = []
        for i in range(1, len(rates)):
            if rates[i - 1] > 0 and rates[i] > 0:
                log_returns.append((rates[i] / rates[i - 1]))

        if not log_returns:
            return {"volatility": None, "message": "Invalid rate data"}

        avg = sum(log_returns) / len(log_returns)
        variance = sum((r - avg) ** 2 for r in log_returns) / (len(log_returns) - 1)
        daily_vol = variance ** 0.5
        annualized_vol = round(daily_vol * (252 ** 0.5), 4)

        max_rate = max(rates)
        min_rate = min(rates)
        latest = rates[-1]
        depreciation = round((rates[-1] - rates[0]) / rates[0], 4) if rates[0] > 0 else 0

        return {
            "pair": f"{base_currency}/{quote_currency}",
            "data_points": len(rates),
            "latest_rate": latest,
            "period_high": max_rate,
            "period_low": min_rate,
            "period_depreciation": depreciation,
            "depreciation_pct": f"{depreciation * 100:.2f}%",
            "daily_volatility": round(daily_vol, 6),
            "annualized_volatility": annualized_vol,
            "annualized_volatility_pct": f"{annualized_vol * 100:.2f}%",
        }

    def forecast_fx(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Simple FX rate forecast."""
        current_rate = float(data.get("current_rate", 0))
        trend_rate = float(data.get("trend_rate", 0))
        periods = int(data.get("periods", 12))
        volatility = float(data.get("volatility", 0.15))

        projections: List[Dict[str, Any]] = []
        rate = current_rate

        for p in range(1, periods + 1):
            rate = rate * (1 + trend_rate)
            upper = rate * (1 + volatility * (p ** 0.5))
            lower = rate * (1 - volatility * (p ** 0.5))
            projections.append({"period": p, "central": round(rate, 2), "upper": round(upper, 2), "lower": round(lower, 2)})

        return {
            "current_rate": current_rate,
            "monthly_trend_rate": trend_rate,
            "assumed_volatility": volatility,
            "projections": projections,
        }

    def health_check(self) -> bool:
        return True
