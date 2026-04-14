# Author: Quadri Atharu
"""Leverage ratio computation — debt/equity, interest coverage, etc."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class LeverageRatiosEngine:
    """Leverage ratio computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all leverage ratios."""
        total_debt = float(data.get("total_debt", 0))
        total_equity = float(data.get("total_equity", 0))
        total_assets = float(data.get("total_assets", 0))
        total_liabilities = float(data.get("total_liabilities", 0))
        ebit = float(data.get("ebit", 0))
        interest_expense = float(data.get("interest_expense", 0))
        current_liabilities = float(data.get("current_liabilities", 0))
        long_term_debt = float(data.get("long_term_debt", 0))
        cash = float(data.get("cash", 0))

        debt_to_equity = round(total_debt / total_equity, 4) if total_equity > 0 else None
        debt_to_assets = round(total_debt / total_assets, 4) if total_assets > 0 else None
        interest_coverage = round(ebit / interest_expense, 4) if interest_expense > 0 else None
        equity_ratio = round(total_equity / total_assets, 4) if total_assets > 0 else None
        debt_to_tangible_assets = round(total_debt / (total_assets - float(data.get("intangible_assets", 0))), 4) if (total_assets - float(data.get("intangible_assets", 0))) > 0 else None
        net_debt = round(total_debt - cash, 2)
        net_debt_to_equity = round(net_debt / total_equity, 4) if total_equity > 0 else None
        long_term_debt_to_equity = round(long_term_debt / total_equity, 4) if total_equity > 0 else None
        fixed_charge_coverage = round((ebit + float(data.get("lease_expense", 0))) / (interest_expense + float(data.get("lease_expense", 0))), 4) if (interest_expense + float(data.get("lease_expense", 0))) > 0 else None

        return {
            "debt_to_equity": debt_to_equity,
            "debt_to_assets": debt_to_assets,
            "interest_coverage": interest_coverage,
            "equity_ratio": equity_ratio,
            "net_debt": net_debt,
            "net_debt_to_equity": net_debt_to_equity,
            "long_term_debt_to_equity": long_term_debt_to_equity,
            "fixed_charge_coverage": fixed_charge_coverage,
            "debt_to_tangible_assets": debt_to_tangible_assets,
            "computed_at": datetime.now().isoformat(),
        }

    def assess_leverage_risk(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Assess overall leverage risk level."""
        dte = float(data.get("debt_to_equity", 0))
        icr = float(data.get("interest_coverage", 100))

        if dte > 3.0 or icr < 1.0:
            risk = "high"
        elif dte > 2.0 or icr < 1.5:
            risk = "moderate"
        elif dte > 1.0 or icr < 3.0:
            risk = "low_moderate"
        else:
            risk = "low"

        return {
            "debt_to_equity": dte,
            "interest_coverage": icr,
            "overall_risk": risk,
            "covenant_breach_risk": dte > 2.0 or icr < 1.5,
            "recommendation": "Reduce debt levels or renegotiate terms" if risk in ("high", "moderate") else "Leverage is within acceptable range",
        }

    def health_check(self) -> bool:
        return True
