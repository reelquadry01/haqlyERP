# Author: Quadri Atharu
"""Investment feasibility analysis — payback, profitability index, and feasibility report."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class InvestmentFeasibilityEngine:
    """Investment feasibility analysis engine."""

    def compute_feasibility(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute full investment feasibility report."""
        initial_investment = float(data.get("initial_investment", 0))
        cash_flows: List[float] = [float(cf) for cf in data.get("cash_flows", [])]
        discount_rate = float(data.get("discount_rate", 0.10))
        project_name = data.get("project_name", "Unnamed Project")

        payback = self._compute_payback(initial_investment, cash_flows)
        discounted_payback = self._compute_discounted_payback(initial_investment, cash_flows, discount_rate)
        npv = self._compute_npv(initial_investment, cash_flows, discount_rate)
        irr = self._compute_irr(initial_investment, cash_flows)
        pi = self._compute_pi(initial_investment, cash_flows, discount_rate)

        total_inflow = round(sum(cash_flows), 2)
        roi = round((total_inflow - initial_investment) / initial_investment, 4) if initial_investment > 0 else None

        feasible = npv > 0 and (irr is not None and irr > discount_rate) and (payback is not None)

        return {
            "project_name": project_name,
            "initial_investment": round(initial_investment, 2),
            "discount_rate": discount_rate,
            "cash_flows": cash_flows,
            "payback_period_years": payback,
            "discounted_payback_years": discounted_payback,
            "npv": npv,
            "irr": irr,
            "irr_pct": f"{irr * 100:.2f}%" if irr is not None else None,
            "profitability_index": pi,
            "roi": roi,
            "feasible": feasible,
            "verdict": "FEASIBLE — proceed with investment" if feasible else "NOT FEASIBLE — reconsider or reject",
            "computed_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _compute_payback(initial: float, cash_flows: List[float]) -> float | None:
        """Compute payback period in years."""
        cumulative = 0.0
        for i, cf in enumerate(cash_flows):
            cumulative += cf
            if cumulative >= initial:
                fraction = 1 - (cumulative - initial) / cf if cf > 0 else 0
                return round(i + fraction, 2) if cf > 0 else round(i + 1, 2)
        return None

    @staticmethod
    def _compute_discounted_payback(initial: float, cash_flows: List[float], rate: float) -> float | None:
        """Compute discounted payback period."""
        cumulative = 0.0
        for i, cf in enumerate(cash_flows):
            pv = cf / (1 + rate) ** (i + 1)
            prev_cumulative = cumulative
            cumulative += pv
            if cumulative >= initial:
                remaining = initial - prev_cumulative
                fraction = remaining / pv if pv > 0 else 1
                return round(i + fraction, 2)
        return None

    @staticmethod
    def _compute_npv(initial: float, cash_flows: List[float], rate: float) -> float:
        """Compute NPV."""
        pv = sum(cf / (1 + rate) ** (i + 1) for i, cf in enumerate(cash_flows))
        return round(pv - initial, 2)

    @staticmethod
    def _compute_irr(initial: float, cash_flows: List[float]) -> float | None:
        """Compute IRR using Newton-Raphson."""
        all_flows = [-initial] + cash_flows
        rate = 0.1
        for _ in range(1000):
            npv = sum(cf / (1 + rate) ** i for i, cf in enumerate(all_flows))
            dnpv = sum(-i * cf / (1 + rate) ** (i + 1) for i, cf in enumerate(all_flows))
            if abs(dnpv) < 1e-12:
                break
            new_rate = rate - npv / dnpv
            if abs(new_rate - rate) < 1e-8:
                return round(new_rate, 6)
            rate = new_rate
            rate = max(min(rate, 10), -0.5)
        return None

    @staticmethod
    def _compute_pi(initial: float, cash_flows: List[float], rate: float) -> float | None:
        """Compute Profitability Index."""
        if initial <= 0:
            return None
        pv = sum(cf / (1 + rate) ** (i + 1) for i, cf in enumerate(cash_flows))
        return round(pv / initial, 4)

    def health_check(self) -> bool:
        return True
