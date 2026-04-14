# Author: Quadri Atharu
"""Reporting Agent — generates income statement, balance sheet, cash flow, and ratio analysis."""

from __future__ import annotations

from typing import Any, Dict

from ..reporting import (
    AnnualReportingEngine,
    BalanceSheetEngine,
    CashFlowStatementEngine,
    IncomeStatementEngine,
    MonthlyReportingEngine,
    QuarterlyReportingEngine,
)
from ..financial_analysis import (
    EfficiencyRatiosEngine,
    LeverageRatiosEngine,
    LiquidityRatiosEngine,
    ProfitabilityRatiosEngine,
    TrendAnalysisEngine,
)
from .base_agent import BaseAgent


class ReportingAgent(BaseAgent):
    """Agent responsible for financial statement generation and ratio analysis."""

    agent_name = "reporting_agent"

    def __init__(self) -> None:
        super().__init__()
        self._income_stmt = IncomeStatementEngine()
        self._balance_sheet = BalanceSheetEngine()
        self._cash_flow = CashFlowStatementEngine()
        self._liquidity = LiquidityRatiosEngine()
        self._profitability = ProfitabilityRatiosEngine()
        self._leverage = LeverageRatiosEngine()
        self._efficiency = EfficiencyRatiosEngine()
        self._trend = TrendAnalysisEngine()
        self._monthly = MonthlyReportingEngine()
        self._quarterly = QuarterlyReportingEngine()
        self._annual = AnnualReportingEngine()
        self.register_skill("generate_income_statement", self._generate_income_statement)
        self.register_skill("generate_balance_sheet", self._generate_balance_sheet)
        self.register_skill("generate_cash_flow", self._generate_cash_flow)
        self.register_skill("generate_ratio_analysis", self._generate_ratio_analysis)

    def _generate_income_statement(self, data: Dict[str, Any]) -> Dict[str, Any]:
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        revenue_accounts = data.get("revenue_accounts", [])
        expense_accounts = data.get("expense_accounts", [])
        cogs_accounts = data.get("cogs_accounts", [])
        other_income_accounts = data.get("other_income_accounts", [])

        result = self._income_stmt.generate(
            company_id=company_id,
            period_start=period_start,
            period_end=period_end,
            revenue_accounts=revenue_accounts,
            cogs_accounts=cogs_accounts,
            expense_accounts=expense_accounts,
            other_income_accounts=other_income_accounts,
        )
        return {"success": True, "income_statement": result}

    def _generate_balance_sheet(self, data: Dict[str, Any]) -> Dict[str, Any]:
        company_id = data.get("company_id", "")
        as_of_date = data.get("as_of_date", "")
        asset_accounts = data.get("asset_accounts", [])
        liability_accounts = data.get("liability_accounts", [])
        equity_accounts = data.get("equity_accounts", [])

        result = self._balance_sheet.generate(
            company_id=company_id,
            as_of_date=as_of_date,
            asset_accounts=asset_accounts,
            liability_accounts=liability_accounts,
            equity_accounts=equity_accounts,
        )
        return {"success": True, "balance_sheet": result}

    def _generate_cash_flow(self, data: Dict[str, Any]) -> Dict[str, Any]:
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        operating_data = data.get("operating_data", {})
        investing_data = data.get("investing_data", {})
        financing_data = data.get("financing_data", {})

        result = self._cash_flow.generate(
            company_id=company_id,
            period_start=period_start,
            period_end=period_end,
            operating_data=operating_data,
            investing_data=investing_data,
            financing_data=financing_data,
        )
        return {"success": True, "cash_flow_statement": result}

    def _generate_ratio_analysis(self, data: Dict[str, Any]) -> Dict[str, Any]:
        balance_sheet = data.get("balance_sheet", {})
        income_statement = data.get("income_statement", {})
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")

        liquidity = self._liquidity.compute(balance_sheet, income_statement)
        profitability = self._profitability.compute(balance_sheet, income_statement)
        leverage = self._leverage.compute(balance_sheet, income_statement)
        efficiency = self._efficiency.compute(balance_sheet, income_statement)

        trend_data = data.get("historical_periods")
        trend = None
        if trend_data:
            trend = self._trend.analyze(trend_data)

        return {
            "success": True,
            "liquidity_ratios": liquidity,
            "profitability_ratios": profitability,
            "leverage_ratios": leverage,
            "efficiency_ratios": efficiency,
            "trend_analysis": trend,
            "period_start": period_start,
            "period_end": period_end,
        }
