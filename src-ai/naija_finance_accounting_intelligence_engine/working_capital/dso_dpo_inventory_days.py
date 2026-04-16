# Author: Quadri Atharu
"""Cash Conversion Cycle computation — DSO, DPO, and Inventory Days."""

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


class CashConversionCycleEngine:
    """Cash Conversion Cycle (CCC) computation engine."""

    def compute_ccc(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute the Cash Conversion Cycle from DSO, DPO, and DIO."""
        dso = Decimal(str(data.get("dso", 0)))
        dpo = Decimal(str(data.get("dpo", 0)))
        dio = Decimal(str(data.get("dio", 0)))

        ccc = _money_round(dso + dio - dpo)

        if ccc <= 30:
            health = "excellent"
        elif ccc <= 60:
            health = "good"
        elif ccc <= 90:
            health = "adequate"
        elif ccc <= 120:
            health = "poor"
        else:
            health = "critical"

        return {
            "dso": dso,
            "dio": dio,
            "dpo": dpo,
            "cash_conversion_cycle": ccc,
            "health": health,
            "interpretation": self._interpret_ccc(ccc, dso, dpo, dio),
            "recommendations": self._recommend_ccc(ccc, dso, dpo, dio),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_ccc_from_financials(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute CCC from raw financial data."""
        accounts_receivable = Decimal(str(data.get("accounts_receivable", 0)))
        accounts_payable = Decimal(str(data.get("accounts_payable", 0)))
        inventory = Decimal(str(data.get("inventory", 0)))
        revenue = Decimal(str(data.get("revenue", 0)))
        cogs = Decimal(str(data.get("cogs", 0)))
        period_days = int(data.get("period_days", 365))

        dso = _money_round(accounts_receivable / (revenue / period_days)) if revenue > 0 else 0
        dpo = _money_round(accounts_payable / (cogs / period_days)) if cogs > 0 else 0
        dio = _money_round(inventory / (cogs / period_days)) if cogs > 0 else 0

        ccc_data = self.compute_ccc({"dso": dso, "dpo": dpo, "dio": dio})
        ccc_data["source_data"] = {
            "accounts_receivable": _money_round(accounts_receivable),
            "accounts_payable": _money_round(accounts_payable),
            "inventory": _money_round(inventory),
            "revenue": _money_round(revenue),
            "cogs": _money_round(cogs),
            "period_days": period_days,
        }
        return ccc_data

    def compute_working_capital_requirement(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute working capital requirement based on CCC."""
        ccc = Decimal(str(data.get("ccc", 0)))
        daily_operating_cost = Decimal(str(data.get("daily_operating_cost", 0)))
        revenue_growth_rate = Decimal(str(data.get("revenue_growth_rate", 0)))

        base_requirement = _money_round(ccc * daily_operating_cost)
        growth_requirement = _money_round(base_requirement * revenue_growth_rate)
        total_requirement = _money_round(base_requirement + growth_requirement)

        return {
            "cash_conversion_cycle": ccc,
            "daily_operating_cost": _money_round(daily_operating_cost),
            "base_working_capital": base_requirement,
            "growth_working_capital": growth_requirement,
            "total_working_capital_requirement": total_requirement,
            "funding_gap_days": ccc,
            "recommendation": f"Maintain minimum working capital of NGN {total_requirement:,.2f}",
        }

    @staticmethod
    def _interpret_ccc(ccc: float, dso: float, dpo: float, dio: float) -> str:
        """Interpret the CCC value."""
        if ccc < 0:
            return f"Negative CCC ({ccc} days) — the company is financed by its suppliers. Excellent cash flow management."
        parts = []
        if dso > 60:
            parts.append(f"DSO of {dso} days suggests slow collections")
        if dio > 90:
            parts.append(f"DIO of {dio} days indicates slow inventory turnover")
        if dpo < 30:
            parts.append(f"DPO of {dpo} days suggests paying suppliers too quickly")
        if parts:
            return f"CCC of {ccc} days. {'; '.join(parts)}."
        return f"CCC of {ccc} days — within acceptable range for Nigerian businesses."

    @staticmethod
    def _recommend_ccc(ccc: float, dso: float, dpo: float, dio: float) -> list:
        """Generate recommendations for CCC improvement."""
        recs: list = []
        if dso > 45:
            recs.append("Improve collections: implement automated reminders and early payment discounts")
        if dpo < 30:
            recs.append("Negotiate longer payment terms with suppliers (target 45-60 days)")
        if dio > 60:
            recs.append("Optimize inventory: implement JIT or reduce safety stock levels")
        if ccc > 90:
            recs.append("Consider supply chain financing or factoring to reduce funding gap")
        return recs or ["CCC is well-managed — maintain current practices"]

    def health_check(self) -> bool:
        return True
