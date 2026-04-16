# Author: Quadri Atharu
"""Cash Flow Statement generation engine — indirect and direct methods."""

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


class CashFlowStatementEngine:
    """Cash Flow Statement generation engine."""

    def generate_indirect(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate Cash Flow Statement using the indirect method."""
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        currency = data.get("currency", "NGN")

        net_income = Decimal(str(data.get("net_income", 0)))
        depreciation = Decimal(str(data.get("depreciation", 0)))
        amortisation = Decimal(str(data.get("amortisation", 0)))
        impairment = Decimal(str(data.get("impairment", 0)))
        gain_on_disposal = Decimal(str(data.get("gain_on_disposal", 0)))
        loss_on_disposal = Decimal(str(data.get("loss_on_disposal", 0)))
        finance_costs = Decimal(str(data.get("finance_costs", 0)))
        fx_gain_loss = Decimal(str(data.get("fx_gain_loss", 0)))

        increase_receivables = Decimal(str(data.get("increase_in_trade_receivables", 0)))
        decrease_payables = Decimal(str(data.get("decrease_in_trade_payables", 0)))
        increase_inventory = Decimal(str(data.get("increase_in_inventory", 0)))
        other_working_capital = Decimal(str(data.get("other_working_capital_changes", 0)))

        operating_before_wc = _money_round(net_income + depreciation + amortisation + impairment - gain_on_disposal + loss_on_disposal + finance_costs - fx_gain_loss)
        working_capital_adj = _money_round(-increase_receivables + decrease_payables - increase_inventory + other_working_capital)
        net_cash_operating = _money_round(operating_before_wc + working_capital_adj)

        ppe_purchase = Decimal(str(data.get("ppe_purchases", 0)))
        ppe_disposal = Decimal(str(data.get("ppe_disposal_proceeds", 0)))
        investment_purchase = Decimal(str(data.get("investment_purchases", 0)))
        investment_disposal = Decimal(str(data.get("investment_disposal_proceeds", 0)))
        interest_received = Decimal(str(data.get("interest_received", 0)))
        dividends_received = Decimal(str(data.get("dividends_received", 0)))
        net_cash_investing = _money_round(-ppe_purchase + ppe_disposal - investment_purchase + investment_disposal + interest_received + dividends_received)

        share_issuance = Decimal(str(data.get("share_issuance_proceeds", 0)))
        loan_proceeds = Decimal(str(data.get("loan_proceeds", 0)))
        loan_repayment = Decimal(str(data.get("loan_repayment", 0)))
        dividends_paid = Decimal(str(data.get("dividends_paid", 0)))
        interest_paid = Decimal(str(data.get("interest_paid", 0)))
        net_cash_financing = _money_round(share_issuance + loan_proceeds - loan_repayment - dividends_paid - interest_paid)

        net_change = _money_round(net_cash_operating + net_cash_investing + net_cash_financing)
        cash_beginning = Decimal(str(data.get("cash_at_beginning", 0)))
        cash_end = _money_round(cash_beginning + net_change)

        return {
            "report_type": "cash_flow_statement_indirect",
            "company_id": company_id,
            "period_start": period_start,
            "period_end": period_end,
            "currency": currency,
            "operating_activities": {
                "profit_before_working_capital": operating_before_wc,
                "working_capital_adjustments": working_capital_adj,
                "net_cash_from_operating": net_cash_operating,
            },
            "investing_activities": {
                "ppe_purchases": -ppe_purchase,
                "ppe_disposal_proceeds": ppe_disposal,
                "investment_purchases": -investment_purchase,
                "investment_disposal_proceeds": investment_disposal,
                "interest_received": interest_received,
                "dividends_received": dividends_received,
                "net_cash_from_investing": net_cash_investing,
            },
            "financing_activities": {
                "share_issuance": share_issuance,
                "loan_proceeds": loan_proceeds,
                "loan_repayment": -loan_repayment,
                "dividends_paid": -dividends_paid,
                "interest_paid": -interest_paid,
                "net_cash_from_financing": net_cash_financing,
            },
            "net_change_in_cash": net_change,
            "cash_at_beginning": _money_round(cash_beginning),
            "cash_at_end": cash_end,
            "generated_at": datetime.now().isoformat(),
        }

    def generate_direct(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate Cash Flow Statement using the direct method."""
        cash_from_customers = Decimal(str(data.get("cash_received_from_customers", 0)))
        cash_paid_to_suppliers = Decimal(str(data.get("cash_paid_to_suppliers", 0)))
        cash_paid_to_employees = Decimal(str(data.get("cash_paid_to_employees", 0)))
        other_operating_receipts = Decimal(str(data.get("other_operating_receipts", 0)))
        other_operating_payments = Decimal(str(data.get("other_operating_payments", 0)))
        interest_paid = Decimal(str(data.get("interest_paid", 0)))
        tax_paid = Decimal(str(data.get("tax_paid", 0)))

        net_cash_operating = _money_round(cash_from_customers - cash_paid_to_suppliers - cash_paid_to_employees + other_operating_receipts - other_operating_payments - interest_paid - tax_paid)

        net_cash_investing = Decimal(str(data.get("net_cash_from_investing", 0)))
        net_cash_financing = Decimal(str(data.get("net_cash_from_financing", 0)))
        net_change = _money_round(net_cash_operating + net_cash_investing + net_cash_financing)
        cash_beginning = Decimal(str(data.get("cash_at_beginning", 0)))
        cash_end = _money_round(cash_beginning + net_change)

        return {
            "report_type": "cash_flow_statement_direct",
            "operating_activities": {
                "cash_from_customers": cash_from_customers,
                "cash_paid_to_suppliers": -cash_paid_to_suppliers,
                "cash_paid_to_employees": -cash_paid_to_employees,
                "other_receipts": other_operating_receipts,
                "other_payments": -other_operating_payments,
                "interest_paid": -interest_paid,
                "tax_paid": -tax_paid,
                "net_cash_from_operating": net_cash_operating,
            },
            "net_cash_from_investing": net_cash_investing,
            "net_cash_from_financing": net_cash_financing,
            "net_change_in_cash": net_change,
            "cash_at_beginning": _money_round(cash_beginning),
            "cash_at_end": cash_end,
        }

    def health_check(self) -> bool:
        return True
