# Author: Quadri Atharu
"""Multi-period trend analysis with anomaly detection."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)


class TrendAnalysisEngine:
    """Multi-period trend analysis engine with anomaly detection."""

    def analyze_trend(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze trend for a metric across multiple periods."""
        periods: List[Dict[str, Any]] = data.get("periods", [])
        metric = data.get("metric", "revenue")

        if len(periods) < 2:
            return {"trend": "insufficient_data", "message": "At least 2 periods required for trend analysis"}

        values = [float(p.get(metric, 0)) for p in periods]
        labels = [p.get("label", f"Period {i+1}") for i, p in enumerate(periods)]

        growth_rates = self._compute_growth_rates(values)
        avg_growth = round(sum(g for g in growth_rates if g is not None) / max(len([g for g in growth_rates if g is not None]), 1), 4) if growth_rates else 0

        direction = "increasing" if avg_growth > 0.01 else ("decreasing" if avg_growth < -0.01 else "stable")
        volatility = self._compute_volatility(growth_rates)

        anomalies = self._detect_anomalies(values, growth_rates, labels)

        return {
            "metric": metric,
            "periods_analyzed": len(periods),
            "direction": direction,
            "average_growth_rate": avg_growth,
            "growth_rates": growth_rates,
            "volatility": round(volatility, 4),
            "values": values,
            "labels": labels,
            "anomalies_detected": anomalies,
            "anomaly_count": len(anomalies),
            "computed_at": datetime.now().isoformat(),
        }

    def analyze_multiple_metrics(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze trends for multiple metrics simultaneously."""
        periods: List[Dict[str, Any]] = data.get("periods", [])
        metrics = data.get("metrics", ["revenue", "net_income", "total_assets"])

        results: Dict[str, Any] = {}
        for metric in metrics:
            results[metric] = self.analyze_trend({"periods": periods, "metric": metric})

        overall_direction = "mixed"
        directions = [r.get("direction") for r in results.values()]
        if all(d == "increasing" for d in directions):
            overall_direction = "all_increasing"
        elif all(d == "decreasing" for d in directions):
            overall_direction = "all_decreasing"

        return {
            "metrics_analyzed": len(metrics),
            "results": results,
            "overall_direction": overall_direction,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_cagr(self, start_value: float, end_value: float, years: int) -> Dict[str, Any]:
        """Compute Compound Annual Growth Rate."""
        if years <= 0:
            return {"cagr": None, "message": "Years must be positive"}
        if start_value <= 0:
            return {"cagr": None, "message": "Start value must be positive for CAGR"}

        cagr = round((end_value / start_value) ** (1 / years) - 1, 6)
        return {
            "start_value": round(start_value, 2),
            "end_value": round(end_value, 2),
            "years": years,
            "cagr": cagr,
            "cagr_pct": f"{cagr * 100:.2f}%",
        }

    @staticmethod
    def _compute_growth_rates(values: List[float]) -> List[Optional[float]]:
        """Compute period-over-period growth rates."""
        rates: List[Optional[float]] = [None]
        for i in range(1, len(values)):
            if values[i - 1] != 0:
                rates.append(round((values[i] - values[i - 1]) / abs(values[i - 1]), 4))
            else:
                rates.append(None)
        return rates

    @staticmethod
    def _compute_volatility(growth_rates: List[Optional[float]]) -> float:
        """Compute volatility (standard deviation) of growth rates."""
        valid = [g for g in growth_rates if g is not None]
        if len(valid) < 2:
            return 0.0
        avg = sum(valid) / len(valid)
        variance = sum((g - avg) ** 2 for g in valid) / (len(valid) - 1)
        return variance ** 0.5

    @staticmethod
    def _detect_anomalies(values: List[float], growth_rates: List[Optional[float]], labels: List[str]) -> List[Dict[str, Any]]:
        """Detect anomalies in trend data using statistical deviation."""
        valid_rates = [g for g in growth_rates if g is not None]
        if len(valid_rates) < 2:
            return []

        avg = sum(valid_rates) / len(valid_rates)
        std = (sum((g - avg) ** 2 for g in valid_rates) / (len(valid_rates) - 1)) ** 0.5

        anomalies: List[Dict[str, Any]] = []
        for i, rate in enumerate(growth_rates):
            if rate is not None and std > 0 and abs(rate - avg) > 2 * std:
                anomalies.append({
                    "period": labels[i] if i < len(labels) else f"Period {i}",
                    "growth_rate": rate,
                    "expected_range": f"{round(avg - 2 * std, 4)} to {round(avg + 2 * std, 4)}",
                    "deviation": round(abs(rate - avg) / std, 2),
                    "type": "spike" if rate > avg else "dip",
                })
        return anomalies

    def health_check(self) -> bool:
        return True
