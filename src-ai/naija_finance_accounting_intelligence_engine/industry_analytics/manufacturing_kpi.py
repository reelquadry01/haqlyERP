# Author: Quadri Atharu
"""Manufacturing KPI engine — capacity utilization, OEE, cost per unit."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class ManufacturingKpiEngine:
    """Manufacturing KPI computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all manufacturing KPIs."""
        actual_output = float(data.get("actual_output", 0))
        maximum_capacity = float(data.get("maximum_capacity", 0))
        good_units = float(data.get("good_units", 0))
        total_units = float(data.get("total_units", 0))
        planned_production_time = float(data.get("planned_production_time", 0))
        operating_time = float(data.get("operating_time", 0))
        ideal_cycle_time = float(data.get("ideal_cycle_time", 0))
        total_count = float(data.get("total_count", 0))
        total_manufacturing_cost = float(data.get("total_manufacturing_cost", 0))
        units_produced = float(data.get("units_produced", 0))
        scrap_cost = float(data.get("scrap_cost", 0))
        rework_cost = float(data.get("rework_cost", 0))
        total_revenue = float(data.get("total_revenue", 0))

        capacity_utilization = round(actual_output / maximum_capacity, 4) if maximum_capacity > 0 else None
        quality_rate = round(good_units / total_units, 4) if total_units > 0 else None
        availability = round(operating_time / planned_production_time, 4) if planned_production_time > 0 else None
        performance = round((ideal_cycle_time * total_count) / operating_time, 4) if operating_time > 0 else None

        oee = round(availability * performance * quality_rate, 4) if all(v is not None and v > 0 for v in [availability, performance, quality_rate]) else None

        cost_per_unit = round(total_manufacturing_cost / units_produced, 2) if units_produced > 0 else None
        scrap_rate = round(scrap_cost / total_manufacturing_cost, 4) if total_manufacturing_cost > 0 else None
        rework_rate = round(rework_cost / total_manufacturing_cost, 4) if total_manufacturing_cost > 0 else None
        manufacturing_margin = round((total_revenue - total_manufacturing_cost) / total_revenue, 4) if total_revenue > 0 else None

        return {
            "capacity_utilization": capacity_utilization,
            "capacity_utilization_pct": f"{capacity_utilization * 100:.1f}%" if capacity_utilization else None,
            "oee": oee,
            "oee_pct": f"{oee * 100:.1f}%" if oee else None,
            "availability": availability,
            "performance": performance,
            "quality_rate": quality_rate,
            "cost_per_unit": cost_per_unit,
            "scrap_rate": scrap_rate,
            "rework_rate": rework_rate,
            "manufacturing_margin": manufacturing_margin,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
