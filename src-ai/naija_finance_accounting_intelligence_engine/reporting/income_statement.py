# Author: Quadri Atharu
"""Income Statement generation engine with comparative period support."""

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


class IncomeStatementEngine:
    """Income Statement generation engine."""

    def generate(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate an Income Statement (Statement of Profit or Loss)."""
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        currency = data.get("currency", "NGN")
        comparative = data.get("comparative", False)

        revenue = Decimal(str(data.get("revenue", 0)))
        other_income = Decimal(str(data.get("other_income", 0)))
        cogs = Decimal(str(data.get("cogs", 0)))
        gross_profit = _money_round(revenue - cogs)

        selling_expenses = Decimal(str(data.get("selling_expenses", 0)))
        admin_expenses = Decimal(str(data.get("admin_expenses", 0)))
        depreciation = Decimal(str(data.get("depreciation", 0)))
        amortisation = Decimal(str(data.get("amortisation", 0)))
        other_operating = Decimal(str(data.get("other_operating_expenses", 0)))
        total_opex = _money_round(selling_expenses + admin_expenses + depreciation + amortisation + other_operating)

        operating_income = _money_round(gross_profit - total_opex)

        finance_costs = Decimal(str(data.get("finance_costs", 0)))
        finance_income = Decimal(str(data.get("finance_income", 0)))
        net_finance_costs = _money_round(finance_costs - finance_income)

        share_of_associate_profit = Decimal(str(data.get("share_of_associate_profit", 0)))
        profit_before_tax = _money_round(operating_income - net_finance_costs + share_of_associate_profit + other_income)

        tax_expense = Decimal(str(data.get("tax_expense", 0)))
        net_income = _money_round(profit_before_tax - tax_expense)

        other_comprehensive = Decimal(str(data.get("other_comprehensive_income", 0)))
        total_comprehensive = _money_round(net_income + other_comprehensive)

        shares_outstanding = Decimal(str(data.get("shares_outstanding", 1)))
        eps = _money_round(net_income / shares_outstanding) if shares_outstanding > 0 else None

        lines: List[Dict[str, Any]] = [
            {"label": "Revenue", "amount": revenue, "note_ref": "1"},
            {"label": "Cost of Sales", "amount": -cogs, "note_ref": "2"},
            {"label": "Gross Profit", "amount": gross_profit, "is_subtotal": True},
            {"label": "Selling Expenses", "amount": -selling_expenses, "note_ref": "3"},
            {"label": "Administrative Expenses", "amount": -admin_expenses, "note_ref": "3"},
            {"label": "Depreciation and Amortisation", "amount": -(depreciation + amortisation), "note_ref": "4"},
            {"label": "Other Operating Expenses", "amount": -other_operating, "note_ref": "5"},
            {"label": "Operating Profit", "amount": operating_income, "is_subtotal": True},
            {"label": "Finance Costs", "amount": -finance_costs, "note_ref": "6"},
            {"label": "Finance Income", "amount": finance_income, "note_ref": "6"},
            {"label": "Other Income", "amount": other_income, "note_ref": "7"},
            {"label": "Profit Before Tax", "amount": profit_before_tax, "is_subtotal": True},
            {"label": "Income Tax Expense", "amount": -tax_expense, "note_ref": "8"},
            {"label": "Profit for the Year", "amount": net_income, "is_subtotal": True},
            {"label": "Other Comprehensive Income", "amount": other_comprehensive, "note_ref": "9"},
            {"label": "Total Comprehensive Income", "amount": total_comprehensive, "is_total": True},
        ]

        result: Dict[str, Any] = {
            "report_type": "income_statement",
            "statement_name": "Statement of Profit or Loss and Other Comprehensive Income",
            "company_id": company_id,
            "period_start": period_start,
            "period_end": period_end,
            "currency": currency,
            "lines": lines,
            "totals": {
                "revenue": revenue,
                "gross_profit": gross_profit,
                "operating_profit": operating_income,
                "profit_before_tax": profit_before_tax,
                "net_income": net_income,
                "total_comprehensive_income": total_comprehensive,
                "eps": eps,
            },
            "generated_at": datetime.now().isoformat(),
        }

        if comparative:
            result["comparative"] = data.get("comparative_data")

        logger.info("income_statement_generated", company_id=company_id, net_income=net_income)
        return result

    def health_check(self) -> bool:
        return True
