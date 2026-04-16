# Author: Quadri Atharu
"""Variance analysis engine with detection and categorization."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

VARIANCE_THRESHOLDS = {
    "favourable_attention": 0.10,
    "adverse_attention": 0.10,
    "critical": 0.25,
}


class VarianceAnalysisEngine:
    """Variance detection and categorization engine."""

    def analyze_variance(self, budgeted: List[Dict[str, Any]], actual: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Perform variance analysis between budgeted and actual amounts."""
        budget_map: Dict[str, float] = {}
        actual_map: Dict[str, float] = {}

        for b in budgeted:
            budget_map[str(b.get("account_code", ""))] = float(b.get("budgeted_amount", 0))

        for a in actual:
            actual_map[str(a.get("account_code", ""))] = float(a.get("actual_amount", 0))

        all_codes = sorted(set(budget_map.keys()) | set(actual_map.keys()))
        variances: List[Dict[str, Any]] = []

        total_budgeted = 0.0
        total_actual = 0.0

        for code in all_codes:
            bud = budget_map.get(code, 0)
            act = actual_map.get(code, 0)
            variance = _money_round(act - bud)
            variance_pct = round(variance / bud, 4) if bud != 0 else None

            is_revenue = code.startswith("4")
            is_expense = code.startswith("5")

            if is_revenue:
                direction = "favourable" if variance > 0 else ("adverse" if variance < 0 else "on_budget")
            elif is_expense:
                direction = "favourable" if variance < 0 else ("adverse" if variance > 0 else "on_budget")
            else:
                direction = "favourable" if variance > 0 else ("adverse" if variance < 0 else "on_budget")

            severity = self._classify_severity(abs(variance_pct) if variance_pct is not None else 0, code)

            variances.append({
                "account_code": code,
                "budgeted_amount": bud,
                "actual_amount": act,
                "variance": variance,
                "variance_pct": variance_pct,
                "direction": direction,
                "severity": severity,
                "category": self._categorize_variance(code, variance, variance_pct),
            })

            total_budgeted += bud
            total_actual += act

        total_variance = _money_round(total_actual - total_budgeted)
        total_variance_pct = round(total_variance / total_budgeted, 4) if total_budgeted != 0 else None

        favourable_variances = [v for v in variances if v["direction"] == "favourable" and abs(v.get("variance", 0)) > 0]
        adverse_variances = [v for v in variances if v["direction"] == "adverse" and abs(v.get("variance", 0)) > 0]
        critical_variances = [v for v in variances if v["severity"] == "critical"]

        result: Dict[str, Any] = {
            "variance_lines": variances,
            "summary": {
                "total_budgeted": _money_round(total_budgeted),
                "total_actual": _money_round(total_actual),
                "total_variance": total_variance,
                "total_variance_pct": total_variance_pct,
                "favourable_count": len(favourable_variances),
                "adverse_count": len(adverse_variances),
                "critical_count": len(critical_variances),
                "favourable_total": _money_round(sum(v["variance"] for v in favourable_variances)),
                "adverse_total": _money_round(sum(abs(v["variance"]) for v in adverse_variances)),
            },
            "critical_variances": critical_variances,
            "recommendations": self._generate_recommendations(critical_variances, adverse_variances),
            "analyzed_at": datetime.now().isoformat(),
        }

        logger.info("variance_analysis_complete", total_variance=total_variance, critical=len(critical_variances))
        return result

    def _classify_severity(self, abs_pct: float, code: str) -> str:
        """Classify variance severity."""
        if abs_pct >= VARIANCE_THRESHOLDS["critical"]:
            return "critical"
        elif abs_pct >= VARIANCE_THRESHOLDS["adverse_attention"]:
            return "significant"
        elif abs_pct >= 0.05:
            return "moderate"
        return "minor"

    @staticmethod
    def _categorize_variance(code: str, variance: float, variance_pct: float | None) -> str:
        """Categorize the type of variance."""
        if code.startswith("4"):
            return "revenue_variance"
        elif code.startswith("5"):
            if "1" in code[1:3]:
                return "cost_of_sales_variance"
            return "operating_expense_variance"
        elif code.startswith("1"):
            return "balance_sheet_variance"
        return "other_variance"

    @staticmethod
    def _generate_recommendations(critical: List[Dict[str, Any]], adverse: List[Dict[str, Any]]) -> List[str]:
        """Generate actionable recommendations from variance analysis."""
        recs: List[str] = []
        if critical:
            recs.append(f"URGENT: {len(critical)} critical variances require immediate management attention")
        if adverse:
            revenue_adverse = [v for v in adverse if v.get("category") == "revenue_variance"]
            expense_adverse = [v for v in adverse if v.get("category") in ("cost_of_sales_variance", "operating_expense_variance")]
            if revenue_adverse:
                recs.append("Revenue shortfall detected — review pricing strategy and sales pipeline")
            if expense_adverse:
                recs.append("Expense overrun detected — implement cost control measures immediately")
        if not critical and not adverse:
            recs.append("Performance is on track — no significant variances detected")
        return recs

    def health_check(self) -> bool:
        return True
