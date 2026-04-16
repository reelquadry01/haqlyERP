# Author: Quadri Atharu
"""Finance Agent — financial analysis, valuation, risk assessment, and forecasting."""

from __future__ import annotations

from typing import Any, Dict, List

from ..financial_analysis import (
    EfficiencyRatiosEngine,
    LeverageRatiosEngine,
    LiquidityRatiosEngine,
    ProfitabilityRatiosEngine,
    TrendAnalysisEngine,
    PeerComparisonEngine,
)
from ..valuation import (
    DcfEngine,
    InvestmentFeasibilityEngine,
    IrrEngine,
    NpvEngine,
    WaccEngine,
)
from ..risk_management import (
    CreditRiskEngine,
    LiquidityRiskEngine,
    MarketRiskEngine,
    RiskDashboardEngine,
)
from .base_agent import BaseAgent


class FinanceAgent(BaseAgent):
    """Agent responsible for financial analysis, valuation, risk assessment, and forecasting."""

    agent_name = "finance_agent"

    def __init__(self) -> None:
        super().__init__()
        self._liquidity = LiquidityRatiosEngine()
        self._profitability = ProfitabilityRatiosEngine()
        self._leverage = LeverageRatiosEngine()
        self._efficiency = EfficiencyRatiosEngine()
        self._trend = TrendAnalysisEngine()
        self._peer = PeerComparisonEngine()
        self._npv = NpvEngine()
        self._irr = IrrEngine()
        self._wacc = WaccEngine()
        self._dcf = DcfEngine()
        self._feasibility = InvestmentFeasibilityEngine()
        self._credit_risk = CreditRiskEngine()
        self._liquidity_risk = LiquidityRiskEngine()
        self._market_risk = MarketRiskEngine()
        self._risk_dashboard = RiskDashboardEngine()
        self.register_skill("analyze_financials", self._analyze_financials)
        self.register_skill("compute_valuation", self._compute_valuation)
        self.register_skill("assess_risk", self._assess_risk)
        self.register_skill("forecast", self._forecast)

    def _analyze_financials(self, data: Dict[str, Any]) -> Dict[str, Any]:
        balance_sheet = data.get("balance_sheet", {})
        income_statement = data.get("income_statement", {})
        company_id = data.get("company_id", "")

        liquidity = self._liquidity.compute(balance_sheet, income_statement)
        profitability = self._profitability.compute(balance_sheet, income_statement)
        leverage = self._leverage.compute(balance_sheet, income_statement)
        efficiency = self._efficiency.compute(balance_sheet, income_statement)

        peer_data = data.get("peer_data")
        peer_comparison = None
        if peer_data:
            peer_comparison = self._peer.compare(
                company_data={"balance_sheet": balance_sheet, "income_statement": income_statement},
                peer_group=peer_data,
            )

        historical = data.get("historical_periods")
        trend = None
        if historical:
            trend = self._trend.analyze(historical)

        return {
            "success": True,
            "company_id": company_id,
            "liquidity": liquidity,
            "profitability": profitability,
            "leverage": leverage,
            "efficiency": efficiency,
            "peer_comparison": peer_comparison,
            "trend_analysis": trend,
        }

    def _compute_valuation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        valuation_method = data.get("method", "dcf").lower()
        company_id = data.get("company_id", "")

        if valuation_method == "dcf":
            free_cash_flows = data.get("free_cash_flows", [])
            discount_rate = float(data.get("discount_rate", 0.12))
            terminal_growth = float(data.get("terminal_growth_rate", 0.03))
            net_debt = float(data.get("net_debt", 0))
            result = self._dcf.valuate(free_cash_flows, discount_rate, terminal_growth, net_debt)
        elif valuation_method == "npv":
            cash_flows = data.get("cash_flows", [])
            discount_rate = float(data.get("discount_rate", 0.12))
            result = self._npv.compute(cash_flows, discount_rate)
        elif valuation_method == "irr":
            cash_flows = data.get("cash_flows", [])
            result = self._irr.compute(cash_flows)
        elif valuation_method == "wacc":
            result = self._wacc.compute(
                equity_value=float(data.get("equity_value", 0)),
                debt_value=float(data.get("debt_value", 0)),
                cost_of_equity=float(data.get("cost_of_equity", 0.15)),
                cost_of_debt=float(data.get("cost_of_debt", 0.10)),
                tax_rate=float(data.get("tax_rate", 0.25)),
            )
        elif valuation_method == "feasibility":
            result = self._feasibility.assess(data)
        else:
            return {"success": False, "error": f"Unsupported valuation method: {valuation_method}"}

        return {
            "success": True,
            "method": valuation_method,
            "company_id": company_id,
            "valuation": result,
        }

    def _assess_risk(self, data: Dict[str, Any]) -> Dict[str, Any]:
        company_id = data.get("company_id", "")
        risk_types = data.get("risk_types", ["credit", "liquidity", "market"])

        assessments: Dict[str, Any] = {}

        if "credit" in risk_types:
            assessments["credit_risk"] = self._credit_risk.assess(data.get("credit_data", {}))
        if "liquidity" in risk_types:
            assessments["liquidity_risk"] = self._liquidity_risk.assess(data.get("liquidity_data", {}))
        if "market" in risk_types:
            assessments["market_risk"] = self._market_risk.assess(data.get("market_data", {}))

        dashboard = self._risk_dashboard.generate(assessments)

        return {
            "success": True,
            "company_id": company_id,
            "assessments": assessments,
            "dashboard": dashboard,
        }

    def _forecast(self, data: Dict[str, Any]) -> Dict[str, Any]:
        company_id = data.get("company_id", "")
        historical_revenue = data.get("historical_revenue", [])
        historical_expenses = data.get("historical_expenses", [])
        periods_ahead = int(data.get("periods_ahead", 12))
        growth_rate = data.get("growth_rate")
        method = data.get("method", "linear").lower()

        revenue_forecast = self._forecast_series(historical_revenue, periods_ahead, growth_rate, method)
        expense_forecast = self._forecast_series(historical_expenses, periods_ahead, growth_rate, method)

        profit_forecast: List[Dict[str, Any]] = []
        for i in range(periods_ahead):
            rev = revenue_forecast["forecast"][i] if i < len(revenue_forecast["forecast"]) else 0
            exp = expense_forecast["forecast"][i] if i < len(expense_forecast["forecast"]) else 0
            profit_forecast.append({
                "period": i + 1,
                "revenue": round(rev, 2),
                "expenses": round(exp, 2),
                "profit": round(rev - exp, 2),
            })

        return {
            "success": True,
            "company_id": company_id,
            "method": method,
            "periods_ahead": periods_ahead,
            "revenue_forecast": revenue_forecast,
            "expense_forecast": expense_forecast,
            "profit_forecast": profit_forecast,
        }

    @staticmethod
    def _forecast_series(
        historical: List[float],
        periods_ahead: int,
        growth_rate: float | None,
        method: str,
    ) -> Dict[str, Any]:
        if not historical:
            return {"forecast": [0.0] * periods_ahead, "method": method, "confidence": 0.0}

        n = len(historical)
        if method == "linear" and n >= 2:
            x_mean = sum(range(n)) / n
            y_mean = sum(historical) / n
            numerator = sum((i - x_mean) * (v - y_mean) for i, v in enumerate(historical))
            denominator = sum((i - x_mean) ** 2 for i in range(n))
            slope = numerator / denominator if denominator else 0
            intercept = y_mean - slope * x_mean
            forecast = [round(intercept + slope * (n + i), 2) for i in range(periods_ahead)]
            confidence = 0.7
        elif method == "average_growth" and n >= 2:
            growth_rates = [(historical[i] - historical[i - 1]) / abs(historical[i - 1]) for i in range(1, n) if historical[i - 1] != 0]
            avg_growth = sum(growth_rates) / len(growth_rates) if growth_rates else 0
            last = historical[-1]
            forecast = [round(last * (1 + avg_growth) ** (i + 1), 2) for i in range(periods_ahead)]
            confidence = 0.6
        elif growth_rate is not None:
            last = historical[-1] if historical else 0
            forecast = [round(last * (1 + growth_rate) ** (i + 1), 2) for i in range(periods_ahead)]
            confidence = 0.5
        else:
            avg = sum(historical) / n
            forecast = [round(avg, 2)] * periods_ahead
            confidence = 0.3

        return {"forecast": forecast, "method": method, "confidence": confidence, "historical_count": n}
