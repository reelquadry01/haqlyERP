# Author: Quadri Atharu
"""Profitability ratio computation — ROE, ROA, ROCE, margins."""

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


class ProfitabilityRatiosEngine:
    """Profitability ratio computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all profitability ratios."""
        revenue = Decimal(str(data.get("revenue", 0)))
        cogs = Decimal(str(data.get("cogs", 0)))
        gross_profit = Decimal(str(data.get("gross_profit", revenue - cogs)))
        operating_income = Decimal(str(data.get("operating_income", 0)))
        net_income = Decimal(str(data.get("net_income", 0)))
        total_assets = Decimal(str(data.get("total_assets", 0)))
        total_equity = Decimal(str(data.get("total_equity", 0)))
        total_liabilities = Decimal(str(data.get("total_liabilities", 0)))
        interest_expense = Decimal(str(data.get("interest_expense", 0)))
        tax_expense = Decimal(str(data.get("tax_expense", 0)))

        ebit = operating_income if operating_income else gross_profit - Decimal(str(data.get("operating_expenses", 0)))

        capital_employed = total_equity + (total_liabilities - Decimal(str(data.get("current_liabilities", 0))))

        gross_margin = round(gross_profit / revenue, 4) if revenue > 0 else None
        operating_margin = round(operating_income / revenue, 4) if revenue > 0 else None
        net_margin = round(net_income / revenue, 4) if revenue > 0 else None
        ebitda_margin = round((ebit + Decimal(str(data.get("depreciation", 0)))) / revenue, 4) if revenue > 0 else None
        roa = round(net_income / total_assets, 4) if total_assets > 0 else None
        roe = round(net_income / total_equity, 4) if total_equity > 0 else None
        roce = round(ebit / capital_employed, 4) if capital_employed > 0 else None

        return {
            "gross_margin": gross_margin,
            "operating_margin": operating_margin,
            "net_margin": net_margin,
            "ebitda_margin": ebitda_margin,
            "roa": roa,
            "roe": roe,
            "roce": roce,
            "gross_profit": _money_round(gross_profit),
            "ebit": _money_round(ebit),
            "capital_employed": _money_round(capital_employed),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dupont_roe(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Decompose ROE using the DuPont model into margin, turnover, and leverage."""
        net_income = Decimal(str(data.get("net_income", 0)))
        revenue = Decimal(str(data.get("revenue", 0)))
        total_assets = Decimal(str(data.get("total_assets", 0)))
        total_equity = Decimal(str(data.get("total_equity", 0)))

        net_margin = round(net_income / revenue, 4) if revenue > 0 else 0
        asset_turnover = round(revenue / total_assets, 4) if total_assets > 0 else 0
        equity_multiplier = round(total_assets / total_equity, 4) if total_equity > 0 else 0
        roe = round(net_margin * asset_turnover * equity_multiplier, 4)

        return {
            "roe": roe,
            "net_margin": net_margin,
            "asset_turnover": asset_turnover,
            "equity_multiplier": equity_multiplier,
            "decomposition": f"ROE = {net_margin:.4f} x {asset_turnover:.4f} x {equity_multiplier:.4f} = {roe:.4f}",
            "strongest_driver": max(
                [("net_margin", net_margin), ("asset_turnover", asset_turnover), ("leverage", equity_multiplier - 1)],
                key=lambda x: x[1],
            )[0],
        }

    def health_check(self) -> bool:
        return True
