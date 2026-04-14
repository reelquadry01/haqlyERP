# Author: Quadri Atharu
"""Profitability ratio computation — ROE, ROA, ROCE, margins."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class ProfitabilityRatiosEngine:
    """Profitability ratio computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all profitability ratios."""
        revenue = float(data.get("revenue", 0))
        cogs = float(data.get("cogs", 0))
        gross_profit = float(data.get("gross_profit", revenue - cogs))
        operating_income = float(data.get("operating_income", 0))
        net_income = float(data.get("net_income", 0))
        total_assets = float(data.get("total_assets", 0))
        total_equity = float(data.get("total_equity", 0))
        total_liabilities = float(data.get("total_liabilities", 0))
        interest_expense = float(data.get("interest_expense", 0))
        tax_expense = float(data.get("tax_expense", 0))

        ebit = operating_income if operating_income else gross_profit - float(data.get("operating_expenses", 0))

        capital_employed = total_equity + (total_liabilities - float(data.get("current_liabilities", 0)))

        gross_margin = round(gross_profit / revenue, 4) if revenue > 0 else None
        operating_margin = round(operating_income / revenue, 4) if revenue > 0 else None
        net_margin = round(net_income / revenue, 4) if revenue > 0 else None
        ebitda_margin = round((ebit + float(data.get("depreciation", 0))) / revenue, 4) if revenue > 0 else None
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
            "gross_profit": round(gross_profit, 2),
            "ebit": round(ebit, 2),
            "capital_employed": round(capital_employed, 2),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_dupont_roe(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Decompose ROE using the DuPont model into margin, turnover, and leverage."""
        net_income = float(data.get("net_income", 0))
        revenue = float(data.get("revenue", 0))
        total_assets = float(data.get("total_assets", 0))
        total_equity = float(data.get("total_equity", 0))

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
