# Author: Quadri Atharu
"""Efficiency ratio computation — asset turnover, receivables, payables turnover."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class EfficiencyRatiosEngine:
    """Efficiency ratio computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all efficiency ratios."""
        revenue = float(data.get("revenue", 0))
        total_assets = float(data.get("total_assets", 0))
        fixed_assets = float(data.get("fixed_assets", 0))
        accounts_receivable = float(data.get("accounts_receivable", 0))
        accounts_payable = float(data.get("accounts_payable", 0))
        inventory = float(data.get("inventory", 0))
        cogs = float(data.get("cogs", 0))
        period_days = int(data.get("period_days", 365))

        asset_turnover = round(revenue / total_assets, 4) if total_assets > 0 else None
        fixed_asset_turnover = round(revenue / fixed_assets, 4) if fixed_assets > 0 else None
        receivables_turnover = round(revenue / accounts_receivable, 4) if accounts_receivable > 0 else None
        payables_turnover = round(cogs / accounts_payable, 4) if accounts_payable > 0 else None
        inventory_turnover = round(cogs / inventory, 4) if inventory > 0 else None

        dso = round(period_days / receivables_turnover, 2) if receivables_turnover and receivables_turnover > 0 else None
        dpo = round(period_days / payables_turnover, 2) if payables_turnover and payables_turnover > 0 else None
        dio = round(period_days / inventory_turnover, 2) if inventory_turnover and inventory_turnover > 0 else None
        ccc = round(dso + dio - dpo, 2) if all(v is not None for v in [dso, dio, dpo]) else None

        return {
            "asset_turnover": asset_turnover,
            "fixed_asset_turnover": fixed_asset_turnover,
            "receivables_turnover": receivables_turnover,
            "payables_turnover": payables_turnover,
            "inventory_turnover": inventory_turnover,
            "dso": dso,
            "dpo": dpo,
            "dio": dio,
            "cash_conversion_cycle": ccc,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
