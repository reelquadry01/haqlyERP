# Author: Quadri Atharu
"""Rolling forecast engine for continuous budget updates."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class RollingForecastEngine:
    """Rolling forecast engine that continuously updates projections."""

    def generate_rolling_forecast(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a rolling forecast from actual data and budget."""
        actuals: List[Dict[str, Any]] = data.get("actuals", [])
        budget: List[Dict[str, Any]] = data.get("budget", [])
        remaining_periods = int(data.get("remaining_periods", 6))
        trend_method = data.get("trend_method", "linear").lower()
        company_id = data.get("company_id", "")

        if not actuals and not budget:
            return {"message": "No actual or budget data provided for forecasting"}

        forecast_lines: List[Dict[str, Any]] = []
        all_accounts = sorted(set(
            [str(a.get("account_code", "")) for a in actuals] +
            [str(b.get("account_code", "")) for b in budget]
        ))

        for code in all_accounts:
            actual_values = [float(a.get("amount", 0)) for a in actuals if str(a.get("account_code", "")) == code]
            budget_values = [float(b.get("budgeted_amount", 0)) for b in budget if str(b.get("account_code", "")) == code]

            if actual_values and len(actual_values) >= 2:
                trend = self._compute_trend(actual_values, remaining_periods, trend_method)
            elif budget_values:
                avg = round(sum(budget_values) / len(budget_values), 2) if budget_values else 0
                trend = [avg] * remaining_periods
            else:
                trend = [0.0] * remaining_periods

            forecast_lines.append({
                "account_code": code,
                "actual_data_points": len(actual_values),
                "forecast_periods": remaining_periods,
                "forecast_values": trend,
                "trend_method": trend_method,
            })

        return {
            "company_id": company_id,
            "forecast_type": "rolling",
            "remaining_periods": remaining_periods,
            "trend_method": trend_method,
            "forecast_lines": forecast_lines,
            "generated_at": datetime.now().isoformat(),
        }

    def update_forecast_with_actuals(self, existing_forecast: List[Dict[str, Any]], new_actuals: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Update an existing rolling forecast with new actual data."""
        actual_map: Dict[str, float] = {}
        for a in new_actuals:
            code = str(a.get("account_code", ""))
            actual_map[code] = actual_map.get(code, 0) + float(a.get("amount", 0))

        updated: List[Dict[str, Any]] = []
        for line in existing_forecast:
            code = str(line.get("account_code", ""))
            if code in actual_map:
                new_forecast = line.get("forecast_values", [])[1:] if line.get("forecast_values") else []
                if len(new_forecast) < 1:
                    new_forecast = [actual_map[code]]
                updated.append({**line, "forecast_values": new_forecast, "latest_actual": actual_map[code]})
            else:
                updated.append(line)

        return updated

    def _compute_trend(self, values: List[float], periods: int, method: str) -> List[float]:
        """Compute trend-based forecast values."""
        if not values:
            return [0.0] * periods

        if method == "linear":
            return self._linear_trend(values, periods)
        elif method == "moving_average":
            return self._moving_average_trend(values, periods)
        elif method == "exponential_smoothing":
            return self._exponential_smoothing_trend(values, periods)
        return self._linear_trend(values, periods)

    @staticmethod
    def _linear_trend(values: List[float], periods: int) -> List[float]:
        """Simple linear trend extrapolation."""
        n = len(values)
        if n < 2:
            return [values[0]] * periods
        x_avg = (n - 1) / 2
        y_avg = sum(values) / n
        numerator = sum((i - x_avg) * (v - y_avg) for i, v in enumerate(values))
        denominator = sum((i - x_avg) ** 2 for i in range(n))
        slope = numerator / denominator if denominator != 0 else 0
        intercept = y_avg - slope * x_avg

        return [round(max(intercept + slope * (n + p), 0), 2) for p in range(periods)]

    @staticmethod
    def _moving_average_trend(values: List[float], periods: int) -> List[float]:
        """Moving average forecast."""
        window = min(3, len(values))
        avg = round(sum(values[-window:]) / window, 2)
        return [avg] * periods

    @staticmethod
    def _exponential_smoothing_trend(values: List[float], periods: int) -> List[float]:
        """Simple exponential smoothing forecast."""
        alpha = 0.3
        smoothed = values[0]
        for v in values[1:]:
            smoothed = alpha * v + (1 - alpha) * smoothed
        return [round(smoothed, 2)] * periods

    def health_check(self) -> bool:
        return True
