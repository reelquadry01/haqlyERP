# Author: Quadri Atharu
"""Inflation pressure computation and projection adjustment for Nigerian businesses."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)

CATEGORY_INFLATION_SENSITIVITY: Dict[str, float] = {
    "raw_materials": 0.85,
    "labor": 0.65,
    "rent": 0.40,
    "utilities": 0.70,
    "transport": 0.75,
    "administrative": 0.30,
    "marketing": 0.20,
    "depreciation": 0.10,
    "interest_expense": 0.50,
    "professional_fees": 0.35,
    "insurance": 0.25,
    "maintenance": 0.55,
    "technology": 0.40,
    "training": 0.30,
}


class InflationPressureEngine:
    """Compute inflation impact on expenses and adjust financial projections."""

    def compute_inflation_impact(
        self,
        expenses_by_category: Dict[str, float],
        inflation_rate: float,
    ) -> Dict[str, Any]:
        if not expenses_by_category:
            return {
                "inflation_rate": inflation_rate,
                "total_expenses": 0.0,
                "total_inflation_impact": 0.0,
                "adjusted_expenses": {},
                "category_breakdown": [],
                "computed_at": datetime.now().isoformat(),
            }

        total_expenses = sum(expenses_by_category.values())
        category_breakdown: List[Dict[str, Any]] = []
        adjusted_expenses: Dict[str, float] = {}
        total_inflation_impact = 0.0

        for category, amount in expenses_by_category.items():
            sensitivity = CATEGORY_INFLATION_SENSITIVITY.get(category.lower(), 0.50)
            category_impact = round(amount * inflation_rate * sensitivity, 2)
            adjusted_amount = round(amount + category_impact, 2)

            total_inflation_impact += category_impact
            adjusted_expenses[category] = adjusted_amount

            category_breakdown.append({
                "category": category,
                "base_amount": round(amount, 2),
                "inflation_sensitivity": sensitivity,
                "inflation_impact": category_impact,
                "adjusted_amount": adjusted_amount,
                "impact_pct": round(category_impact / amount, 4) if amount > 0 else 0,
            })

        category_breakdown.sort(key=lambda x: x["inflation_impact"], reverse=True)

        overall_impact_pct = round(total_inflation_impact / total_expenses, 4) if total_expenses > 0 else 0

        risk_level = "low"
        if overall_impact_pct > 0.15:
            risk_level = "critical"
        elif overall_impact_pct > 0.10:
            risk_level = "high"
        elif overall_impact_pct > 0.05:
            risk_level = "medium"

        recommendations: List[str] = []
        high_impact = [c for c in category_breakdown if c["inflation_sensitivity"] > 0.6]
        if high_impact:
            categories_str = ", ".join(c["category"] for c in high_impact[:3])
            recommendations.append(f"High-sensitivity categories ({categories_str}) — negotiate fixed-price contracts")
        if overall_impact_pct > 0.10:
            recommendations.append("Material inflation impact — consider price adjustments to maintain margins")
        if inflation_rate > 0.20:
            recommendations.append("Hyperinflation threshold — apply IAS 29 hyperinflation accounting")
        if not recommendations:
            recommendations.append("Inflation impact within manageable range")

        result: Dict[str, Any] = {
            "inflation_rate": inflation_rate,
            "total_base_expenses": round(total_expenses, 2),
            "total_inflation_impact": round(total_inflation_impact, 2),
            "adjusted_expenses": adjusted_expenses,
            "total_adjusted_expenses": round(total_expenses + total_inflation_impact, 2),
            "overall_impact_pct": overall_impact_pct,
            "risk_level": risk_level,
            "category_breakdown": category_breakdown,
            "recommendations": recommendations,
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("inflation_impact_computed", overall_impact_pct=overall_impact_pct, risk_level=risk_level)
        return result

    def adjust_projections(
        self,
        baseline_projections: List[Dict[str, Any]],
        inflation_forecast: List[Dict[str, Any]],
    ) -> List[Dict[str, Any]]:
        if not baseline_projections:
            return []

        if not inflation_forecast:
            return baseline_projections

        inflation_map: Dict[str, float] = {}
        for entry in inflation_forecast:
            period = str(entry.get("period", entry.get("month", "")))
            rate = float(entry.get("inflation_rate", entry.get("rate", 0)))
            inflation_map[period] = rate

        default_inflation = 0.15
        if inflation_map:
            default_inflation = sum(inflation_map.values()) / len(inflation_map)

        adjusted: List[Dict[str, Any]] = []
        for projection in baseline_projections:
            period = str(projection.get("period", projection.get("month", "")))
            inflation = inflation_map.get(period, default_inflation)

            adjusted_projection: Dict[str, Any] = {"period": period, "baseline": {}}

            for key, value in projection.items():
                if key in ("period", "month"):
                    continue
                if isinstance(value, (int, float)) and value > 0:
                    sensitivity = CATEGORY_INFLATION_SENSITIVITY.get(key.lower(), 0.50)
                    impact = value * inflation * sensitivity
                    adjusted_val = round(value + impact, 2)
                    adjusted_projection["baseline"][key] = round(value, 2)
                    adjusted_projection[key] = adjusted_val
                    adjusted_projection[f"{key}_inflation_impact"] = round(impact, 2)
                elif isinstance(value, (int, float)):
                    adjusted_projection["baseline"][key] = round(value, 2)
                    adjusted_projection[key] = round(value, 2)
                    adjusted_projection[f"{key}_inflation_impact"] = 0.0
                else:
                    adjusted_projection[key] = value

            adjusted_projection["applied_inflation_rate"] = inflation
            adjusted_projection["computed_at"] = datetime.now().isoformat()
            adjusted.append(adjusted_projection)

        logger.info("projections_adjusted", periods=len(adjusted))
        return adjusted

    def compute_weighted_inflation(
        self,
        expenses_by_category: Dict[str, float],
        inflation_by_category: Dict[str, float],
    ) -> Dict[str, Any]:
        total_expenses = sum(expenses_by_category.values())
        if total_expenses <= 0:
            return {"weighted_inflation": 0.0, "computed_at": datetime.now().isoformat()}

        weighted_sum = 0.0
        details: List[Dict[str, Any]] = []

        for category, amount in expenses_by_category.items():
            weight = amount / total_expenses
            cat_inflation = inflation_by_category.get(category.lower(), 0.15)
            weighted_contribution = weight * cat_inflation
            weighted_sum += weighted_contribution
            details.append({
                "category": category,
                "amount": round(amount, 2),
                "weight": round(weight, 4),
                "category_inflation": cat_inflation,
                "weighted_contribution": round(weighted_contribution, 4),
            })

        return {
            "weighted_inflation": round(weighted_sum, 4),
            "total_expenses": round(total_expenses, 2),
            "category_details": details,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True


engine = InflationPressureEngine()


def compute_inflation_impact(expenses_by_category: Dict[str, float], inflation_rate: float) -> Dict[str, Any]:
    return engine.compute_inflation_impact(expenses_by_category, inflation_rate)


def adjust_projections(baseline_projections: List[Dict[str, Any]], inflation_forecast: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    return engine.adjust_projections(baseline_projections, inflation_forecast)
