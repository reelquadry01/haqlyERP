# Author: Quadri Atharu
"""Annual budget creation and management engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class AnnualBudgetEngine:
    """Annual budget creation and management engine."""

    def create_budget(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create an annual budget with line items."""
        company_id = data.get("company_id", "")
        fiscal_year = int(data.get("fiscal_year", datetime.now().year + 1))
        name = data.get("name", f"FY{fiscal_year} Budget")
        lines: List[Dict[str, Any]] = data.get("lines", [])
        description = data.get("description", "")

        if not lines:
            raise AccountingError("Budget must have at least one line item")

        total_budgeted = round(sum(float(l.get("budgeted_amount", 0)) for l in lines), 2)

        revenue_lines = [l for l in lines if str(l.get("account_code", "")).startswith("4")]
        expense_lines = [l for l in lines if str(l.get("account_code", "")).startswith("5")]
        total_revenue = round(sum(float(l.get("budgeted_amount", 0)) for l in revenue_lines), 2)
        total_expenses = round(sum(float(l.get("budgeted_amount", 0)) for l in expense_lines), 2)
        projected_surplus = _money_round(total_revenue - total_expenses)

        return {
            "company_id": company_id,
            "fiscal_year": fiscal_year,
            "name": name,
            "description": description,
            "lines": lines,
            "total_budgeted": total_budgeted,
            "total_revenue_budgeted": total_revenue,
            "total_expense_budgeted": total_expenses,
            "projected_surplus_deficit": projected_surplus,
            "margin_pct": round(projected_surplus / total_revenue, 4) if total_revenue > 0 else 0,
            "status": "draft",
            "created_at": datetime.now().isoformat(),
        }

    def allocate_budget_by_periods(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Allocate annual budget amounts across months/quarters."""
        annual_amount = Decimal(str(data.get("annual_amount", 0)))
        method = data.get("allocation_method", "even").lower()
        periods = int(data.get("periods", 12))
        seasonal_weights: List[float] = data.get("seasonal_weights", [])

        if annual_amount <= 0:
            raise AccountingError("Annual budget amount must be positive")

        if method == "even":
            per_period = _money_round(annual_amount / periods)
            remainder = _money_round(annual_amount - per_period * periods)
            allocation = [per_period + (remainder if i == 0 else 0) for i in range(periods)]
        elif method == "seasonal" and len(seasonal_weights) == periods:
            total_weight = sum(seasonal_weights)
            if total_weight <= 0:
                raise AccountingError("Seasonal weights must sum to a positive number")
            allocation = [_money_round(annual_amount * w / total_weight) for w in seasonal_weights]
        elif method == "front_loaded":
            weights = [max(periods - i, 1) for i in range(periods)]
            total_weight = sum(weights)
            allocation = [_money_round(annual_amount * w / total_weight) for w in weights]
        elif method == "back_loaded":
            weights = [i + 1 for i in range(periods)]
            total_weight = sum(weights)
            allocation = [_money_round(annual_amount * w / total_weight) for w in weights]
        else:
            per_period = _money_round(annual_amount / periods)
            allocation = [per_period] * periods

        cumulative: List[float] = []
        running = 0.0
        for a in allocation:
            running += a
            cumulative.append(_money_round(running))

        return {
            "annual_amount": _money_round(annual_amount),
            "periods": periods,
            "allocation_method": method,
            "period_allocation": allocation,
            "cumulative_allocation": cumulative,
            "total_allocated": _money_round(sum(allocation)),
        }

    def apply_growth_rate(self, base_budget: List[Dict[str, Any]], growth_rate: float, exclude_accounts: List[str] | None = None) -> List[Dict[str, Any]]:
        """Apply a growth rate to a base budget for next year planning."""
        exclude = set(exclude_accounts or [])
        adjusted: List[Dict[str, Any]] = []

        for line in base_budget:
            code = str(line.get("account_code", ""))
            amount = float(line.get("budgeted_amount", 0))
            if code in exclude:
                adjusted.append({**line, "growth_applied": 0.0})
            else:
                new_amount = _money_round(amount * (1 + growth_rate))
                adjusted.append({**line, "budgeted_amount": new_amount, "growth_applied": growth_rate})

        return adjusted

    def compare_budget_versions(self, v1: List[Dict[str, Any]], v2: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Compare two budget versions and show differences."""
        v1_map = {str(l.get("account_code", "")): l for l in v1}
        v2_map = {str(l.get("account_code", "")): l for l in v2}

        all_codes = sorted(set(v1_map.keys()) | set(v2_map.keys()))
        differences: List[Dict[str, Any]] = []

        for code in all_codes:
            v1_amt = float(v1_map.get(code, {}).get("budgeted_amount", 0))
            v2_amt = float(v2_map.get(code, {}).get("budgeted_amount", 0))
            diff = _money_round(v2_amt - v1_amt)
            pct = round(diff / v1_amt, 4) if v1_amt != 0 else None

            differences.append({
                "account_code": code,
                "v1_amount": v1_amt,
                "v2_amount": v2_amt,
                "difference": diff,
                "change_pct": pct,
            })

        return {
            "total_v1": round(sum(float(l.get("budgeted_amount", 0)) for l in v1), 2),
            "total_v2": round(sum(float(l.get("budgeted_amount", 0)) for l in v2), 2),
            "total_difference": round(sum(float(l.get("budgeted_amount", 0)) for l in v2) - sum(float(l.get("budgeted_amount", 0)) for l in v1), 2),
            "line_differences": differences,
        }

    def health_check(self) -> bool:
        return True
