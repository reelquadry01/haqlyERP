# Author: Quadri Atharu
"""Automotive KPI engine — vehicle turnover, margin per unit."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class AutomotiveKpiEngine:
    """Automotive KPI computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all automotive KPIs."""
        vehicles_sold = int(data.get("vehicles_sold", 0))
        total_revenue = float(data.get("total_revenue", 0))
        total_cogs = float(data.get("total_cogs", 0))
        average_inventory_units = int(data.get("average_inventory_units", 1))
        showroom_count = int(data.get("showroom_count", 1))
        service_revenue = float(data.get("service_revenue", 0))
        parts_revenue = float(data.get("parts_revenue", 0))
        finance_income = float(data.get("finance_income", 0))

        margin_per_unit = round((total_revenue - total_cogs) / vehicles_sold, 2) if vehicles_sold > 0 else None
        vehicle_turnover = round(vehicles_sold / average_inventory_units, 2) if average_inventory_units > 0 else None
        revenue_per_showroom = round(total_revenue / showroom_count, 2) if showroom_count > 0 else None
        after_sales_revenue = round(service_revenue + parts_revenue, 2)
        after_sales_pct = round(after_sales_revenue / total_revenue, 4) if total_revenue > 0 else None
        finance_penetration = round(finance_income / total_revenue, 4) if total_revenue > 0 else None
        avg_selling_price = round(total_revenue / vehicles_sold, 2) if vehicles_sold > 0 else None

        return {
            "margin_per_unit": margin_per_unit,
            "vehicle_turnover": vehicle_turnover,
            "revenue_per_showroom": revenue_per_showroom,
            "after_sales_revenue": after_sales_revenue,
            "after_sales_pct": after_sales_pct,
            "finance_penetration_rate": finance_penetration,
            "average_selling_price": avg_selling_price,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
