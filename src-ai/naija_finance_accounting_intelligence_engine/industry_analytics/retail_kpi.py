# Author: Quadri Atharu
"""Retail KPI engine — sales/sqm, basket size, conversion rate."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class RetailKpiEngine:
    """Retail KPI computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all retail KPIs."""
        total_revenue = float(data.get("total_revenue", 0))
        total_sqm = float(data.get("total_sqm", 1))
        total_transactions = int(data.get("total_transactions", 1))
        total_visitors = int(data.get("total_visitors", 0))
        cogs = float(data.get("cogs", 0))
        inventory_at_cost = float(data.get("average_inventory", 0))
        footfall = int(data.get("footfall", 0))

        sales_per_sqm = round(total_revenue / total_sqm, 2) if total_sqm > 0 else None
        avg_basket_size = round(total_revenue / total_transactions, 2) if total_transactions > 0 else None
        conversion_rate = round(total_transactions / footfall, 4) if footfall > 0 else None
        gross_margin = round((total_revenue - cogs) / total_revenue, 4) if total_revenue > 0 else None
        inventory_turnover = round(cogs / inventory_at_cost, 2) if inventory_at_cost > 0 else None
        avg_transaction_value = round(total_revenue / total_transactions, 2) if total_transactions > 0 else None
        items_per_transaction = round(float(data.get("items_sold", 0)) / total_transactions, 2) if total_transactions > 0 else None

        return {
            "sales_per_sqm": sales_per_sqm,
            "average_basket_size": avg_basket_size,
            "conversion_rate": conversion_rate,
            "gross_margin": gross_margin,
            "inventory_turnover": inventory_turnover,
            "avg_transaction_value": avg_transaction_value,
            "items_per_transaction": items_per_transaction,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
