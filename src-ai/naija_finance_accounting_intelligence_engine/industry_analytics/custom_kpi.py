# Author: Quadri Atharu
"""Custom KPI definition and computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class CustomKpiEngine:
    """Custom KPI definition and computation engine."""

    def __init__(self) -> None:
        self._definitions: Dict[str, Dict[str, Any]] = {}

    def define_kpi(self, name: str, numerator: str, denominator: str, unit: str = "ratio", target: float | None = None, description: str = "") -> Dict[str, Any]:
        """Define a custom KPI."""
        kpi = {
            "name": name,
            "numerator_field": numerator,
            "denominator_field": denominator,
            "unit": unit,
            "target": target,
            "description": description,
        }
        self._definitions[name] = kpi
        return kpi

    def compute_kpi(self, name: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute a defined custom KPI."""
        kpi = self._definitions.get(name)
        if kpi is None:
            return {"error": f"KPI '{name}' not defined"}

        numerator = float(data.get(kpi["numerator_field"], 0))
        denominator = float(data.get(kpi["denominator_field"], 0))

        if denominator == 0:
            value = None
        else:
            value = round(numerator / denominator, 4)

        on_target = None
        if value is not None and kpi.get("target") is not None:
            on_target = value >= kpi["target"]

        return {
            "name": name,
            "value": value,
            "unit": kpi["unit"],
            "target": kpi.get("target"),
            "on_target": on_target,
            "numerator": round(numerator, 2),
            "denominator": round(denominator, 2),
            "computed_at": datetime.now().isoformat(),
        }

    def compute_all_kpis(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all defined custom KPIs."""
        results = {}
        for name in self._definitions:
            results[name] = self.compute_kpi(name, data)
        return {"kpis": results, "total_computed": len(results)}

    def list_definitions(self) -> Dict[str, Any]:
        """List all defined custom KPIs."""
        return {"definitions": self._definitions, "count": len(self._definitions)}

    def health_check(self) -> bool:
        return True
