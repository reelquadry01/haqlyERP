# Author: Quadri Atharu
"""Import dependency computation and FX impact on COGS for Nigerian businesses."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)


class ImportDependencyEngine:
    """Compute import dependency ratios and FX impact on cost of goods sold."""

    def compute_import_dependency_ratio(
        self,
        imports_value: float,
        total_revenue: float,
    ) -> float:
        if total_revenue <= 0:
            logger.warning("import_dependency_zero_revenue", imports_value=imports_value)
            return 0.0
        ratio = round(imports_value / total_revenue, 4)
        logger.info("import_dependency_computed", imports_value=imports_value, total_revenue=total_revenue, ratio=ratio)
        return ratio

    def assess_fx_impact_on_cogs(
        self,
        import_ratio: float,
        fx_change_pct: float,
        cogs_base: float = 0,
        pass_through_rate: float = 1.0,
    ) -> Dict[str, Any]:
        if import_ratio < 0:
            import_ratio = 0.0
        if import_ratio > 1.0:
            import_ratio = 1.0

        cogs_impact_pct = round(import_ratio * fx_change_pct * pass_through_rate, 4)
        additional_cost = round(cogs_base * cogs_impact_pct, 2) if cogs_base > 0 else 0.0
        revised_cogs = round(cogs_base + additional_cost, 2) if cogs_base > 0 else 0.0

        risk_level = "low"
        if abs(cogs_impact_pct) > 0.10:
            risk_level = "critical"
        elif abs(cogs_impact_pct) > 0.05:
            risk_level = "high"
        elif abs(cogs_impact_pct) > 0.02:
            risk_level = "medium"

        recommendations: List[str] = []
        if import_ratio > 0.5:
            recommendations.append("High import dependency — consider local sourcing alternatives")
        if abs(fx_change_pct) > 0.10:
            recommendations.append("Significant FX movement — hedge foreign currency exposure")
        if abs(cogs_impact_pct) > 0.05:
            recommendations.append("COGS materially impacted — review pricing strategy")
        if pass_through_rate < 1.0:
            recommendations.append("Partial pass-through — margin erosion risk detected")
        if not recommendations:
            recommendations.append("FX impact on COGS within acceptable range")

        result: Dict[str, Any] = {
            "import_ratio": import_ratio,
            "fx_change_pct": fx_change_pct,
            "pass_through_rate": pass_through_rate,
            "cogs_impact_pct": cogs_impact_pct,
            "additional_cost": additional_cost,
            "cogs_base": cogs_base,
            "revised_cogs": revised_cogs,
            "risk_level": risk_level,
            "recommendations": recommendations,
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("fx_impact_on_cogs_computed", cogs_impact_pct=cogs_impact_pct, risk_level=risk_level)
        return result

    def compute_import_breakdown(
        self,
        import_categories: List[Dict[str, Any]],
        total_revenue: float,
    ) -> Dict[str, Any]:
        if not import_categories or total_revenue <= 0:
            return {"categories": [], "total_import_ratio": 0.0, "computed_at": datetime.now().isoformat()}

        total_imports = 0.0
        breakdown: List[Dict[str, Any]] = []
        for cat in import_categories:
            name = cat.get("category", "uncategorized")
            value = float(cat.get("value", 0))
            ratio = round(value / total_revenue, 4) if total_revenue > 0 else 0
            total_imports += value
            breakdown.append({
                "category": name,
                "value": round(value, 2),
                "ratio": ratio,
                "fx_sensitive": cat.get("fx_sensitive", True),
            })

        return {
            "categories": breakdown,
            "total_imports": round(total_imports, 2),
            "total_import_ratio": round(total_imports / total_revenue, 4) if total_revenue > 0 else 0,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True


engine = ImportDependencyEngine()


def compute_import_dependency_ratio(imports_value: float, total_revenue: float) -> float:
    return engine.compute_import_dependency_ratio(imports_value, total_revenue)


def assess_fx_impact_on_cogs(import_ratio: float, fx_change_pct: float, cogs_base: float = 0, pass_through_rate: float = 1.0) -> Dict[str, Any]:
    return engine.assess_fx_impact_on_cogs(import_ratio, fx_change_pct, cogs_base, pass_through_rate)
