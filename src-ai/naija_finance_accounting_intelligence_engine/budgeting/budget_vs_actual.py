# Author: Quadri Atharu
"""Budget vs Actual reporting engine."""

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


class BudgetVsActualEngine:
    """Budget vs Actual (BvA) reporting engine."""

    def generate_bva_report(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a Budget vs Actual report."""
        company_id = data.get("company_id", "")
        fiscal_year = int(data.get("fiscal_year", datetime.now().year))
        period = data.get("period", "")
        budget_lines: List[Dict[str, Any]] = data.get("budget_lines", [])
        actual_lines: List[Dict[str, Any]] = data.get("actual_lines", [])
        budget_name = data.get("budget_name", "")

        budget_map: Dict[str, float] = {str(l.get("account_code", "")): float(l.get("budgeted_amount", 0)) for l in budget_lines}
        actual_map: Dict[str, float] = {str(l.get("account_code", "")): float(l.get("actual_amount", 0)) for l in actual_lines}

        all_codes = sorted(set(budget_map.keys()) | set(actual_map.keys()))
        report_lines: List[Dict[str, Any]] = []

        for code in all_codes:
            budgeted = budget_map.get(code, 0)
            actual = actual_map.get(code, 0)
            variance = _money_round(actual - budgeted)
            variance_pct = round(variance / budgeted, 4) if budgeted != 0 else None
            attainment_pct = round(actual / budgeted, 4) if budgeted != 0 else None

            report_lines.append({
                "account_code": code,
                "account_name": f"Account {code}",
                "budgeted": _money_round(budgeted),
                "actual": _money_round(actual),
                "variance": variance,
                "variance_pct": variance_pct,
                "attainment_pct": attainment_pct,
            })

        total_budgeted = _money_round(sum(budget_map.values()))
        total_actual = _money_round(sum(actual_map.values()))
        total_variance = _money_round(total_actual - total_budgeted)
        overall_attainment = round(total_actual / total_budgeted, 4) if total_budgeted > 0 else None

        return {
            "company_id": company_id,
            "budget_name": budget_name,
            "fiscal_year": fiscal_year,
            "period": period,
            "report_type": "budget_vs_actual",
            "currency": "NGN",
            "lines": report_lines,
            "totals": {
                "total_budgeted": total_budgeted,
                "total_actual": total_actual,
                "total_variance": total_variance,
                "overall_attainment_pct": overall_attainment,
            },
            "generated_at": datetime.now().isoformat(),
        }

    def generate_cumulative_bva(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate cumulative Budget vs Actual over multiple periods."""
        period_budgets: List[Dict[str, Any]] = data.get("period_budgets", [])
        period_actuals: List[Dict[str, Any]] = data.get("period_actuals", [])

        cum_budget: Dict[str, float] = {}
        cum_actual: Dict[str, float] = {}

        for pb in period_budgets:
            for line in pb.get("lines", []):
                code = str(line.get("account_code", ""))
                cum_budget[code] = cum_budget.get(code, 0) + float(line.get("budgeted_amount", 0))

        for pa in period_actuals:
            for line in pa.get("lines", []):
                code = str(line.get("account_code", ""))
                cum_actual[code] = cum_actual.get(code, 0) + float(line.get("actual_amount", 0))

        all_codes = sorted(set(cum_budget.keys()) | set(cum_actual.keys()))
        lines: List[Dict[str, Any]] = []

        for code in all_codes:
            bud = cum_budget.get(code, 0)
            act = cum_actual.get(code, 0)
            lines.append({
                "account_code": code,
                "cumulative_budgeted": _money_round(bud),
                "cumulative_actual": _money_round(act),
                "cumulative_variance": _money_round(act - bud),
                "attainment_pct": round(act / bud, 4) if bud > 0 else None,
            })

        return {
            "report_type": "cumulative_budget_vs_actual",
            "periods_covered": max(len(period_budgets), len(period_actuals)),
            "lines": lines,
            "generated_at": datetime.now().isoformat(),
        }

    def forecast_year_end(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Forecast year-end position based on current BvA trends."""
        annual_budget = Decimal(str(data.get("annual_budget", 0)))
        ytd_actual = Decimal(str(data.get("ytd_actual", 0)))
        periods_elapsed = int(data.get("periods_elapsed", 0))
        total_periods = int(data.get("total_periods", 12))

        if periods_elapsed <= 0 or total_periods <= 0:
            return {"message": "Invalid period data"}

        periods_remaining = total_periods - periods_elapsed
        run_rate = _money_round(ytd_actual / periods_elapsed)
        projected_year_end = _money_round(ytd_actual + run_rate * periods_remaining)
        projected_variance = _money_round(projected_year_end - annual_budget)

        return {
            "annual_budget": _money_round(annual_budget),
            "ytd_actual": _money_round(ytd_actual),
            "periods_elapsed": periods_elapsed,
            "periods_remaining": periods_remaining,
            "monthly_run_rate": run_rate,
            "projected_year_end": projected_year_end,
            "projected_variance": projected_variance,
            "on_track": abs(projected_variance) / annual_budget < 0.05 if annual_budget > 0 else True,
            "recommendation": "Accelerate revenue activities" if projected_variance < 0 else ("Control expenses" if projected_variance > 0 else "On track"),
        }

    def health_check(self) -> bool:
        return True
