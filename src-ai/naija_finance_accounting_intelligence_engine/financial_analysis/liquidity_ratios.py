# Author: Quadri Atharu
"""Liquidity ratio computation engine — current, quick, and cash ratios."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class LiquidityRatiosEngine:
    """Liquidity ratio computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all liquidity ratios."""
        current_assets = Decimal(str(data.get("current_assets", 0)))
        current_liabilities = Decimal(str(data.get("current_liabilities", 0)))
        inventory = Decimal(str(data.get("inventory", 0)))
        cash = Decimal(str(data.get("cash", 0)))
        marketable_securities = Decimal(str(data.get("marketable_securities", 0)))
        operating_cash_flow = Decimal(str(data.get("operating_cash_flow", 0)))

        current_ratio = round(current_assets / current_liabilities, 4) if current_liabilities > 0 else None
        quick_ratio = round((current_assets - inventory) / current_liabilities, 4) if current_liabilities > 0 else None
        cash_ratio = round((cash + marketable_securities) / current_liabilities, 4) if current_liabilities > 0 else None
        operating_cash_flow_ratio = round(operating_cash_flow / current_liabilities, 4) if current_liabilities > 0 else None

        def _assess_current(r: float | None) -> str:
            if r is None: return "unknown"
            if r >= 2.0: return "strong"
            if r >= 1.5: return "adequate"
            if r >= 1.0: return "tight"
            return "distressed"

        def _assess_quick(r: float | None) -> str:
            if r is None: return "unknown"
            if r >= 1.0: return "healthy"
            if r >= 0.7: return "adequate"
            return "concerning"

        return {
            "current_ratio": current_ratio,
            "current_ratio_assessment": _assess_current(current_ratio),
            "quick_ratio": quick_ratio,
            "quick_ratio_assessment": _assess_quick(quick_ratio),
            "cash_ratio": cash_ratio,
            "operating_cash_flow_ratio": operating_cash_flow_ratio,
            "working_capital": _money_round(current_assets - current_liabilities),
            "working_capital_pct_revenue": round((current_assets - current_liabilities) / data.get("revenue", 1), 4) if data.get("revenue", 0) > 0 else None,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_current_ratio(self, current_assets: float, current_liabilities: float) -> Dict[str, Any]:
        """Compute current ratio."""
        if current_liabilities <= 0:
            return {"ratio": None, "message": "Current liabilities must be positive"}
        ratio = round(current_assets / current_liabilities, 4)
        return {"ratio": ratio, "assessment": "healthy" if ratio >= 1.5 else ("tight" if ratio >= 1.0 else "distressed")}

    def compute_quick_ratio(self, current_assets: float, inventory: float, current_liabilities: float) -> Dict[str, Any]:
        """Compute quick (acid-test) ratio."""
        if current_liabilities <= 0:
            return {"ratio": None, "message": "Current liabilities must be positive"}
        ratio = round((current_assets - inventory) / current_liabilities, 4)
        return {"ratio": ratio, "assessment": "healthy" if ratio >= 1.0 else ("adequate" if ratio >= 0.7 else "concerning")}

    def compute_cash_ratio(self, cash: float, marketable_securities: float, current_liabilities: float) -> Dict[str, Any]:
        """Compute cash ratio."""
        if current_liabilities <= 0:
            return {"ratio": None, "message": "Current liabilities must be positive"}
        ratio = round((cash + marketable_securities) / current_liabilities, 4)
        return {"ratio": ratio}

    def health_check(self) -> bool:
        return True
