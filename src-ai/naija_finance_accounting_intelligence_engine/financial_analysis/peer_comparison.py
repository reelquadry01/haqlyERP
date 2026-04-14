# Author: Quadri Atharu
"""Cross-company peer comparison and benchmarking engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class PeerComparisonEngine:
    """Cross-company peer comparison and benchmarking engine."""

    def compare_against_peers(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compare a company's financials against peer companies."""
        company = data.get("company", {})
        peers: List[Dict[str, Any]] = data.get("peers", [])
        metrics = data.get("metrics", ["revenue", "net_margin", "roe", "current_ratio", "debt_to_equity"])

        if not peers:
            return {"message": "No peer data provided"}

        comparisons: Dict[str, Any] = {}

        for metric in metrics:
            company_val = float(company.get(metric, 0))
            peer_values = [float(p.get(metric, 0)) for p in peers if p.get(metric) is not None]

            if not peer_values:
                comparisons[metric] = {"company": company_val, "peer_avg": None, "percentile": None}
                continue

            peer_avg = round(sum(peer_values) / len(peer_values), 4)
            peer_min = min(peer_values)
            peer_max = max(peer_values)
            peer_median = sorted(peer_values)[len(peer_values) // 2]

            sorted_vals = sorted(peer_values + [company_val])
            rank = sorted_vals.index(company_val) + 1
            percentile = round(rank / len(sorted_vals), 4)

            deviation = round((company_val - peer_avg) / peer_avg, 4) if peer_avg != 0 else None

            comparisons[metric] = {
                "company": company_val,
                "peer_avg": peer_avg,
                "peer_median": peer_median,
                "peer_min": peer_min,
                "peer_max": peer_max,
                "percentile": percentile,
                "deviation_from_avg": deviation,
                "above_avg": company_val > peer_avg,
            }

        return {
            "comparisons": comparisons,
            "peer_count": len(peers),
            "metrics_compared": len(metrics),
            "overall_position": self._overall_position(comparisons),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_industry_benchmarks(self, companies: List[Dict[str, Any]], metrics: List[str] | None = None) -> Dict[str, Any]:
        """Compute industry benchmarks from a set of companies."""
        if metrics is None:
            metrics = ["revenue", "net_margin", "roe", "current_ratio", "debt_to_equity", "asset_turnover"]

        benchmarks: Dict[str, Any] = {}
        for metric in metrics:
            values = [float(c.get(metric, 0)) for c in companies if c.get(metric) is not None]
            if not values:
                continue

            sorted_vals = sorted(values)
            benchmarks[metric] = {
                "count": len(values),
                "avg": round(sum(values) / len(values), 4),
                "median": sorted_vals[len(sorted_vals) // 2],
                "p25": sorted_vals[len(sorted_vals) // 4],
                "p75": sorted_vals[3 * len(sorted_vals) // 4],
                "min": sorted_vals[0],
                "max": sorted_vals[-1],
            }

        return {
            "industry_benchmarks": benchmarks,
            "sample_size": len(companies),
            "computed_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _overall_position(comparisons: Dict[str, Any]) -> str:
        """Determine overall competitive position."""
        above = sum(1 for v in comparisons.values() if v.get("above_avg"))
        total = len(comparisons)
        if total == 0:
            return "unknown"
        pct = above / total
        if pct >= 0.75:
            return "industry_leader"
        elif pct >= 0.50:
            return "above_average"
        elif pct >= 0.25:
            return "below_average"
        return "laggard"

    def health_check(self) -> bool:
        return True
