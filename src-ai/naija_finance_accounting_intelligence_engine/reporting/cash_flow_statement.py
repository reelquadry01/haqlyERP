# Author: Quadri Atharu
"""Cash Flow Statement generation engine — indirect and direct methods."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class CashFlowStatementEngine:
    """Cash Flow Statement generation engine."""

    def generate_indirect(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate Cash Flow Statement using the indirect method."""
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        currency = data.get("currency", "NGN")

        net_income = float(data.get("net_income", 0))
        depreciation = float(data.get("depreciation", 0))
        amortisation = float(data.get("amortisation", 0))
        impairment = float(data.get("impairment", 0))
        gain_on_disposal = float(data.get("gain_on_disposal", 0))
        loss_on_disposal = float(data.get("loss_on_disposal", 0))
        finance_costs = float(data.get("finance_costs", 0))
        fx_gain_loss = float(data.get("fx_gain_loss", 0))

        increase_receivables = float(data.get("increase_in_trade_receivables", 0))
        decrease_payables = float(data.get("decrease_in_trade_payables", 0))
        increase_inventory = float(data.get("increase_in_inventory", 0))
        other_working_capital = float(data.get("other_working_capital_changes", 0))

        operating_before_wc = round(net_income + depreciation + amortisation + impairment - gain_on_disposal + loss_on_disposal + finance_costs - fx_gain_loss, 2)
        working_capital_adj = round(-increase_receivables + decrease_payables - increase_inventory + other_working_capital, 2)
        net_cash_operating = round(operating_before_wc + working_capital_adj, 2)

        ppe_purchase = float(data.get("ppe_purchases", 0))
        ppe_disposal = float(data.get("ppe_disposal_proceeds", 0))
        investment_purchase = float(data.get("investment_purchases", 0))
        investment_disposal = float(data.get("investment_disposal_proceeds", 0))
        interest_received = float(data.get("interest_received", 0))
        dividends_received = float(data.get("dividends_received", 0))
        net_cash_investing = round(-ppe_purchase + ppe_disposal - investment_purchase + investment_disposal + interest_received + dividends_received, 2)

        share_issuance = float(data.get("share_issuance_proceeds", 0))
        loan_proceeds = float(data.get("loan_proceeds", 0))
        loan_repayment = float(data.get("loan_repayment", 0))
        dividends_paid = float(data.get("dividends_paid", 0))
        interest_paid = float(data.get("interest_paid", 0))
        net_cash_financing = round(share_issuance + loan_proceeds - loan_repayment - dividends_paid - interest_paid, 2)

        net_change = round(net_cash_operating + net_cash_investing + net_cash_financing, 2)
        cash_beginning = float(data.get("cash_at_beginning", 0))
        cash_end = round(cash_beginning + net_change, 2)

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
            "cash_at_beginning": round(cash_beginning, 2),
            "cash_at_end": cash_end,
            "generated_at": datetime.now().isoformat(),
        }

    def generate_direct(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate Cash Flow Statement using the direct method."""
        cash_from_customers = float(data.get("cash_received_from_customers", 0))
        cash_paid_to_suppliers = float(data.get("cash_paid_to_suppliers", 0))
        cash_paid_to_employees = float(data.get("cash_paid_to_employees", 0))
        other_operating_receipts = float(data.get("other_operating_receipts", 0))
        other_operating_payments = float(data.get("other_operating_payments", 0))
        interest_paid = float(data.get("interest_paid", 0))
        tax_paid = float(data.get("tax_paid", 0))

        net_cash_operating = round(cash_from_customers - cash_paid_to_suppliers - cash_paid_to_employees + other_operating_receipts - other_operating_payments - interest_paid - tax_paid, 2)

        net_cash_investing = float(data.get("net_cash_from_investing", 0))
        net_cash_financing = float(data.get("net_cash_from_financing", 0))
        net_change = round(net_cash_operating + net_cash_investing + net_cash_financing, 2)
        cash_beginning = float(data.get("cash_at_beginning", 0))
        cash_end = round(cash_beginning + net_change, 2)

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
            "cash_at_beginning": round(cash_beginning, 2),
            "cash_at_end": cash_end,
        }

    def health_check(self) -> bool:
        return True
