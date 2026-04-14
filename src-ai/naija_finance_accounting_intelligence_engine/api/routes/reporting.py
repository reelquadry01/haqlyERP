# Author: Quadri Atharu
"""Reporting routes — Income Statement, Balance Sheet, Cash Flow, Ratio Analysis."""

from __future__ import annotations

from typing import Any, Dict, Optional

from fastapi import APIRouter, HTTPException, status

from ...reporting.income_statement import IncomeStatementEngine
from ...reporting.balance_sheet import BalanceSheetEngine
from ...reporting.cash_flow_statement import CashFlowStatementEngine
from ...financial_analysis.profitability_ratios import ProfitabilityRatiosEngine
from ...financial_analysis.liquidity_ratios import LiquidityRatiosEngine
from ...financial_analysis.leverage_ratios import LeverageRatiosEngine
from ...financial_analysis.efficiency_ratios import EfficiencyRatiosEngine

router = APIRouter(prefix="/reporting", tags=["Reporting"])

_income_stmt = IncomeStatementEngine()
_balance_sheet = BalanceSheetEngine()
_cash_flow = CashFlowStatementEngine()
_profitability = ProfitabilityRatiosEngine()
_liquidity = LiquidityRatiosEngine()
_leverage = LeverageRatiosEngine()
_efficiency = EfficiencyRatiosEngine()


@router.post("/income-statement")
async def generate_income_statement(body: Dict[str, Any]) -> Dict[str, Any]:
    """Generate an Income Statement (Statement of Profit or Loss)."""
    company_id = body.get("company_id", "")
    period_id = body.get("period_id", "")
    if not company_id or not period_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id and period_id are required")
    try:
        data = {
            "company_id": company_id,
            "period_start": body.get("period_start", period_id),
            "period_end": period_id,
            "currency": body.get("currency", "NGN"),
            "revenue": body.get("revenue", 0),
            "other_income": body.get("other_income", 0),
            "cogs": body.get("cogs", 0),
            "selling_expenses": body.get("selling_expenses", 0),
            "admin_expenses": body.get("admin_expenses", 0),
            "depreciation": body.get("depreciation", 0),
            "amortisation": body.get("amortisation", 0),
            "other_operating_expenses": body.get("other_operating_expenses", 0),
            "finance_costs": body.get("finance_costs", 0),
            "finance_income": body.get("finance_income", 0),
            "share_of_associate_profit": body.get("share_of_associate_profit", 0),
            "tax_expense": body.get("tax_expense", 0),
            "other_comprehensive_income": body.get("other_comprehensive_income", 0),
            "shares_outstanding": body.get("shares_outstanding", 1),
            "comparative": body.get("comparative", False),
            "comparative_data": body.get("comparative_data"),
        }
        result = _income_stmt.generate(data)
        return {"status": "success", "report": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/balance-sheet")
async def generate_balance_sheet(body: Dict[str, Any]) -> Dict[str, Any]:
    """Generate a classified Balance Sheet (Statement of Financial Position)."""
    company_id = body.get("company_id", "")
    as_of_date = body.get("as_of_date", "")
    if not company_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id is required")
    try:
        data = {
            "company_id": company_id,
            "as_of": as_of_date,
            "currency": body.get("currency", "NGN"),
            "property_plant_equipment": body.get("property_plant_equipment", 0),
            "intangible_assets": body.get("intangible_assets", 0),
            "investment_in_associates": body.get("investment_in_associates", 0),
            "other_non_current_assets": body.get("other_non_current_assets", 0),
            "inventory": body.get("inventory", 0),
            "trade_receivables": body.get("trade_receivables", 0),
            "other_receivables": body.get("other_receivables", 0),
            "cash": body.get("cash", 0),
            "short_term_investments": body.get("short_term_investments", 0),
            "other_current_assets": body.get("other_current_assets", 0),
            "share_capital": body.get("share_capital", 0),
            "share_premium": body.get("share_premium", 0),
            "retained_earnings": body.get("retained_earnings", 0),
            "revaluation_reserve": body.get("revaluation_reserve", 0),
            "other_reserves": body.get("other_reserves", 0),
            "long_term_loans": body.get("long_term_loans", 0),
            "deferred_tax_liability": body.get("deferred_tax_liability", 0),
            "other_non_current_liabilities": body.get("other_non_current_liabilities", 0),
            "trade_payables": body.get("trade_payables", 0),
            "short_term_loans": body.get("short_term_loans", 0),
            "tax_payable": body.get("tax_payable", 0),
            "other_current_liabilities": body.get("other_current_liabilities", 0),
        }
        result = _balance_sheet.generate(data)
        return {"status": "success", "report": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/cash-flow")
async def generate_cash_flow(body: Dict[str, Any]) -> Dict[str, Any]:
    """Generate a Cash Flow Statement using indirect or direct method."""
    company_id = body.get("company_id", "")
    period_id = body.get("period_id", "")
    if not company_id or not period_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id and period_id are required")
    method = body.get("method", "indirect").lower()
    try:
        data = {
            "company_id": company_id,
            "period_start": body.get("period_start", period_id),
            "period_end": period_id,
            "currency": body.get("currency", "NGN"),
            **body,
        }
        if method == "direct":
            result = _cash_flow.generate_direct(data)
        else:
            result = _cash_flow.generate_indirect(data)
        return {"status": "success", "report": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/ratio-analysis")
async def generate_ratio_analysis(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute a full set of financial ratios."""
    try:
        profitability = _profitability.compute_all(body)
        liquidity = _liquidity.compute_all(body)
        leverage = _leverage.compute_all(body)
        efficiency = _efficiency.compute_all(body)
        return {
            "status": "success",
            "ratios": {
                "profitability": profitability,
                "liquidity": liquidity,
                "leverage": leverage,
                "efficiency": efficiency,
            },
        }
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))
