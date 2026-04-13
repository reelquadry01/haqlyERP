# Author: Quadri Atharu
"""Main Finance Intelligence Engine orchestrator."""

from __future__ import annotations

import time
from typing import Any, Dict, Optional

from ..core.config import settings
from ..core.exceptions import AccountingError, TaxError, AnalysisError
from ..core.logging import get_logger
from ..core.registry import ModuleRegistry

logger = get_logger(__name__)


class FinanceIntelligenceEngine:
    """Top-level orchestrator that loads and routes to all sub-modules."""

    def __init__(self) -> None:
        self._registry = ModuleRegistry()
        self._started_at: Optional[float] = None
        self._accounting_engine: Any = None
        self._tax_engine: Any = None
        self._analysis_engine: Any = None
        self._reporting_engine: Any = None
        self._audit_engine: Any = None
        self._treasury_engine: Any = None
        self._ocr_engine: Any = None

    async def initialize(self) -> None:
        """Load all sub-modules and register them."""
        from ..accounting.lifecycle import AccountingLifecycle
        from ..tax_engine.tax_schedules import TaxScheduleGenerator
        from ..tax_engine.tax_returns import TaxReturnGenerator
        from ..tax_engine.tax_risk_flags import TaxRiskDetector

        self._accounting_engine = AccountingLifecycle()
        self._registry.register("accounting", self._accounting_engine)

        self._tax_engine = _TaxEngineFacade()
        self._registry.register("tax", self._tax_engine)

        self._analysis_engine = _AnalysisEngineFacade()
        self._registry.register("analysis", self._analysis_engine)

        self._reporting_engine = _ReportingEngineFacade()
        self._registry.register("reporting", self._reporting_engine)

        self._audit_engine = _AuditEngineFacade()
        self._registry.register("audit", self._audit_engine)

        self._treasury_engine = _TreasuryEngineFacade()
        self._registry.register("treasury", self._treasury_engine)

        self._started_at = time.monotonic()
        logger.info("finance_intelligence_engine_initialized", modules=list(self._registry.list_modules()))

    def register_module(self, name: str, module: Any) -> None:
        """Dynamically register a new module into the engine."""
        self._registry.register(name, module)
        logger.info("module_registered", name=name)

    async def process_transaction(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Route a transaction to the accounting engine for full lifecycle processing."""
        if self._accounting_engine is None:
            raise AccountingError("Accounting engine not initialized")
        logger.info("processing_transaction", transaction_id=data.get("transaction_id"))
        result = await self._accounting_engine.run_full_lifecycle(data)
        return result

    async def compute_tax(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Route tax computation requests to the tax engine."""
        if self._tax_engine is None:
            raise TaxError("Tax engine not initialized")
        logger.info("computing_tax", tax_type=data.get("tax_type"))
        result = await self._tax_engine.compute(data)
        return result

    async def analyze_financials(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Route financial analysis requests to the analysis engine."""
        if self._analysis_engine is None:
            raise AnalysisError("Analysis engine not initialized")
        logger.info("analyzing_financials", analysis_type=data.get("analysis_type"))
        result = await self._analysis_engine.analyze(data)
        return result

    async def generate_report(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Route report generation requests to the reporting engine."""
        if self._reporting_engine is None:
            raise AnalysisError("Reporting engine not initialized")
        logger.info("generating_report", report_type=data.get("report_type"))
        result = await self._reporting_engine.generate(data)
        return result

    def get_status(self) -> Dict[str, Any]:
        """Return health/status of all registered modules."""
        uptime = time.monotonic() - self._started_at if self._started_at else 0
        modules = {}
        for name in self._registry.list_modules():
            module = self._registry.get(name)
            healthy = True
            if hasattr(module, "health_check"):
                healthy = getattr(module, "health_check")()
            modules[name] = {"registered": True, "healthy": healthy}
        return {
            "engine": "naija_finance_accounting_intelligence_engine",
            "version": settings.api_port and "1.0.0",
            "uptime_seconds": round(uptime, 2),
            "modules": modules,
            "currency": settings.default_currency,
        }


class _TaxEngineFacade:
    """Facade that unifies all tax sub-modules behind a single compute method."""

    def __init__(self) -> None:
        from ..tax_engine.vat import VatEngine
        from ..tax_engine.wht import WhtEngine
        from ..tax_engine.cit import CitEngine
        from ..tax_engine.education_tax import EducationTaxEngine
        from ..tax_engine.capital_gains_tax import CapitalGainsTaxEngine
        from ..tax_engine.stamp_duties import StampDutyEngine
        from ..tax_engine.tax_schedules import TaxScheduleGenerator
        from ..tax_engine.tax_returns import TaxReturnGenerator
        from ..tax_engine.tax_risk_flags import TaxRiskDetector

        self.vat = VatEngine()
        self.wht = WhtEngine()
        self.cit = CitEngine()
        self.edu = EducationTaxEngine()
        self.cgt = CapitalGainsTaxEngine()
        self.stamp = StampDutyEngine()
        self.schedules = TaxScheduleGenerator()
        self.returns = TaxReturnGenerator()
        self.risk = TaxRiskDetector()

    async def compute(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Dispatch tax computation based on tax_type field."""
        tax_type = data.get("tax_type", "").upper()
        results: Dict[str, Any] = {}

        if tax_type == "VAT" or tax_type == "ALL":
            results["vat"] = self.vat.compute_output_vat(
                taxable_amount=data.get("taxable_amount", 0), rate=data.get("vat_rate", settings.vat_rate)
            )
        if tax_type == "WHT" or tax_type == "ALL":
            results["wht"] = self.wht.compute_wht(
                payment_amount=data.get("payment_amount", 0), category=data.get("wht_category", "consultancy")
            )
        if tax_type == "CIT" or tax_type == "ALL":
            results["cit"] = self.cit.compute_cit(
                profit_before_tax=data.get("profit_before_tax", 0),
                turnover=data.get("turnover", 0),
                industry=data.get("industry", "general"),
            )
        if tax_type == "EDU_TAX" or tax_type == "ALL":
            results["edu_tax"] = self.edu.compute_education_tax(
                assessable_profit=data.get("assessable_profit", 0)
            )
        if tax_type == "CGT" or tax_type == "ALL":
            results["cgt"] = self.cgt.compute_cgt(
                disposal_proceeds=data.get("disposal_proceeds", 0),
                cost_basis=data.get("cost_basis", 0),
                allowable_deductions=data.get("allowable_deductions", 0),
            )
        if tax_type == "STAMP_DUTY" or tax_type == "ALL":
            results["stamp_duty"] = self.stamp.compute_stamp_duty(
                document_type=data.get("document_type", "general"),
                value=data.get("document_value", 0),
            )
        return results

    def health_check(self) -> bool:
        return True


class _AnalysisEngineFacade:
    """Facade for financial analysis engine."""

    async def analyze(self, data: Dict[str, Any]) -> Dict[str, Any]:
        analysis_type = data.get("analysis_type", "ratios")
        financials = data.get("financials", {})

        if analysis_type == "ratios":
            return self._compute_ratios(financials)
        elif analysis_type == "trend":
            return self._compute_trend(financials)
        elif analysis_type == "peer_comparison":
            return self._compute_peer_comparison(financials, data.get("peers", []))
        elif analysis_type == "valuation":
            return self._compute_valuation(financials, data.get("valuation_params", {}))
        return {"error": f"Unknown analysis type: {analysis_type}"}

    @staticmethod
    def _compute_ratios(financials: Dict[str, Any]) -> Dict[str, Any]:
        revenue = financials.get("revenue", 0) or 0
        net_income = financials.get("net_income", 0) or 0
        total_assets = financials.get("total_assets", 0) or 0
        total_liabilities = financials.get("total_liabilities", 0) or 0
        current_assets = financials.get("current_assets", 0) or 0
        current_liabilities = financials.get("current_liabilities", 0) or 0
        inventory = financials.get("inventory", 0) or 0
        equity = total_assets - total_liabilities if total_assets else 0

        return {
            "profitability": {
                "gross_margin": round(net_income / revenue, 4) if revenue else None,
                "net_margin": round(net_income / revenue, 4) if revenue else None,
                "roa": round(net_income / total_assets, 4) if total_assets else None,
                "roe": round(net_income / equity, 4) if equity else None,
            },
            "liquidity": {
                "current_ratio": round(current_assets / current_liabilities, 4) if current_liabilities else None,
                "quick_ratio": round((current_assets - inventory) / current_liabilities, 4) if current_liabilities else None,
            },
            "leverage": {
                "debt_to_equity": round(total_liabilities / equity, 4) if equity else None,
                "debt_to_assets": round(total_liabilities / total_assets, 4) if total_assets else None,
            },
        }

    @staticmethod
    def _compute_trend(financials: Dict[str, Any]) -> Dict[str, Any]:
        periods = financials.get("periods", [])
        if not periods:
            return {"trend": "insufficient_data"}
        metric = financials.get("metric", "revenue")
        values = [p.get(metric, 0) for p in periods if p.get(metric) is not None]
        if len(values) < 2:
            return {"trend": "insufficient_data"}
        growth_rates = []
        for i in range(1, len(values)):
            if values[i - 1] != 0:
                growth_rates.append(round((values[i] - values[i - 1]) / abs(values[i - 1]), 4))
            else:
                growth_rates.append(None)
        avg_growth = sum(g for g in growth_rates if g is not None) / len([g for g in growth_rates if g is not None]) if growth_rates else 0
        direction = "increasing" if avg_growth > 0 else ("decreasing" if avg_growth < 0 else "stable")
        return {"metric": metric, "growth_rates": growth_rates, "average_growth": round(avg_growth, 4), "direction": direction}

    @staticmethod
    def _compute_peer_comparison(financials: Dict[str, Any], peers: list) -> Dict[str, Any]:
        company_revenue = financials.get("revenue", 0)
        company_margin = financials.get("net_margin", 0)
        peer_revenues = [p.get("revenue", 0) for p in peers]
        peer_margins = [p.get("net_margin", 0) for p in peers]
        avg_peer_revenue = sum(peer_revenues) / len(peer_revenues) if peer_revenues else 0
        avg_peer_margin = sum(peer_margins) / len(peer_margins) if peer_margins else 0
        return {
            "revenue_vs_peer_avg": round(company_revenue / avg_peer_revenue, 4) if avg_peer_revenue else None,
            "margin_vs_peer_avg": round(company_margin / avg_peer_margin, 4) if avg_peer_margin else None,
            "peer_count": len(peers),
        }

    @staticmethod
    def _compute_valuation(financials: Dict[str, Any], params: Dict[str, Any]) -> Dict[str, Any]:
        net_income = financials.get("net_income", 0) or 0
        book_value = financials.get("total_equity", 0) or 0
        pe_ratio = params.get("pe_ratio", 10)
        pb_ratio = params.get("pb_ratio", 1.5)
        shares_outstanding = params.get("shares_outstanding", 1)

        eps = net_income / shares_outstanding if shares_outstanding else 0
        bvps = book_value / shares_outstanding if shares_outstanding else 0
        market_value_pe = eps * pe_ratio
        market_value_pb = bvps * pb_ratio

        return {
            "eps": round(eps, 2),
            "bvps": round(bvps, 2),
            "market_value_pe_estimate": round(market_value_pe * shares_outstanding, 2),
            "market_value_pb_estimate": round(market_value_pb * shares_outstanding, 2),
            "pe_ratio_used": pe_ratio,
            "pb_ratio_used": pb_ratio,
        }

    def health_check(self) -> bool:
        return True


class _ReportingEngineFacade:
    """Facade for financial reporting engine."""

    async def generate(self, data: Dict[str, Any]) -> Dict[str, Any]:
        report_type = data.get("report_type", "income_statement")
        if report_type == "income_statement":
            return self._income_statement(data)
        elif report_type == "balance_sheet":
            return self._balance_sheet(data)
        elif report_type == "cash_flow":
            return self._cash_flow(data)
        elif report_type == "ratio_analysis":
            facade = _AnalysisEngineFacade()
            return await facade.analyze({"analysis_type": "ratios", "financials": data.get("financials", {})})
        return {"error": f"Unknown report type: {report_type}"}

    @staticmethod
    def _income_statement(data: Dict[str, Any]) -> Dict[str, Any]:
        revenue = data.get("revenue", 0)
        cogs = data.get("cogs", 0)
        gross_profit = revenue - cogs
        operating_expenses = data.get("operating_expenses", 0)
        operating_income = gross_profit - operating_expenses
        interest_expense = data.get("interest_expense", 0)
        other_income = data.get("other_income", 0)
        profit_before_tax = operating_income - interest_expense + other_income
        tax_expense = data.get("tax_expense", 0)
        net_income = profit_before_tax - tax_expense

        return {
            "report_type": "income_statement",
            "period": data.get("period", ""),
            "currency": data.get("currency", "NGN"),
            "lines": [
                {"label": "Revenue", "amount": revenue},
                {"label": "Cost of Goods Sold", "amount": -cogs},
                {"label": "Gross Profit", "amount": gross_profit},
                {"label": "Operating Expenses", "amount": -operating_expenses},
                {"label": "Operating Income", "amount": operating_income},
                {"label": "Interest Expense", "amount": -interest_expense},
                {"label": "Other Income", "amount": other_income},
                {"label": "Profit Before Tax", "amount": profit_before_tax},
                {"label": "Tax Expense", "amount": -tax_expense},
                {"label": "Net Income", "amount": net_income},
            ],
            "totals": {
                "gross_profit": gross_profit,
                "operating_income": operating_income,
                "profit_before_tax": profit_before_tax,
                "net_income": net_income,
            },
        }

    @staticmethod
    def _balance_sheet(data: Dict[str, Any]) -> Dict[str, Any]:
        assets = data.get("assets", {})
        liabilities = data.get("liabilities", {})
        equity = data.get("equity", {})
        total_assets = sum(v for v in assets.values() if isinstance(v, (int, float)))
        total_liabilities = sum(v for v in liabilities.values() if isinstance(v, (int, float)))
        total_equity = sum(v for v in equity.values() if isinstance(v, (int, float)))

        return {
            "report_type": "balance_sheet",
            "as_of": data.get("as_of", ""),
            "currency": data.get("currency", "NGN"),
            "assets": assets,
            "liabilities": liabilities,
            "equity": equity,
            "totals": {
                "total_assets": total_assets,
                "total_liabilities": total_liabilities,
                "total_equity": total_equity,
                "balance_check": total_assets == total_liabilities + total_equity,
            },
        }

    @staticmethod
    def _cash_flow(data: Dict[str, Any]) -> Dict[str, Any]:
        operating = data.get("operating_activities", {})
        investing = data.get("investing_activities", {})
        financing = data.get("financing_activities", {})
        net_operating = sum(v for v in operating.values() if isinstance(v, (int, float)))
        net_investing = sum(v for v in investing.values() if isinstance(v, (int, float)))
        net_financing = sum(v for v in financing.values() if isinstance(v, (int, float)))
        net_change = net_operating + net_investing + net_financing
        beginning_cash = data.get("beginning_cash", 0)
        ending_cash = beginning_cash + net_change

        return {
            "report_type": "cash_flow_statement",
            "period": data.get("period", ""),
            "currency": data.get("currency", "NGN"),
            "operating_activities": operating,
            "investing_activities": investing,
            "financing_activities": financing,
            "totals": {
                "net_operating": net_operating,
                "net_investing": net_investing,
                "net_financing": net_financing,
                "net_change_in_cash": net_change,
                "beginning_cash": beginning_cash,
                "ending_cash": ending_cash,
            },
        }

    def health_check(self) -> bool:
        return True


class _AuditEngineFacade:
    """Facade for audit engine."""

    async def trail(self, data: Dict[str, Any]) -> Dict[str, Any]:
        return {"audit_trail": data.get("entries", []), "total_entries": len(data.get("entries", []))}

    async def sampling(self, data: Dict[str, Any]) -> Dict[str, Any]:
        population = data.get("population", [])
        sample_size = data.get("sample_size", min(25, len(population)))
        method = data.get("method", "random")
        sampled = population[:sample_size] if method == "sequential" else population[:sample_size]
        return {"sample": sampled, "sample_size": len(sampled), "method": method}

    async def exceptions(self, data: Dict[str, Any]) -> Dict[str, Any]:
        entries = data.get("entries", [])
        flagged = [e for e in entries if e.get("amount", 0) > data.get("threshold", 1_000_000)]
        return {"exceptions": flagged, "count": len(flagged)}

    def health_check(self) -> bool:
        return True


class _TreasuryEngineFacade:
    """Facade for treasury engine."""

    async def cash_position(self, data: Dict[str, Any]) -> Dict[str, Any]:
        bank_balances = data.get("bank_balances", {})
        petty_cash = data.get("petty_cash", 0)
        total = sum(v for v in bank_balances.values() if isinstance(v, (int, float))) + petty_cash
        return {"bank_balances": bank_balances, "petty_cash": petty_cash, "total_available": total}

    async def bank_reconciliation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        book_balance = data.get("book_balance", 0)
        bank_balance = data.get("bank_balance", 0)
        outstanding_deposits = data.get("outstanding_deposits", 0)
        outstanding_cheques = data.get("outstanding_cheques", 0)
        bank_charges = data.get("bank_charges", 0)
        adjusted_bank = bank_balance + outstanding_deposits - outstanding_cheques - bank_charges
        difference = book_balance - adjusted_bank
        return {
            "book_balance": book_balance,
            "bank_balance": bank_balance,
            "adjusted_bank_balance": adjusted_bank,
            "difference": round(difference, 2),
            "reconciled": abs(difference) < 0.01,
        }

    async def loan_schedule(self, data: Dict[str, Any]) -> Dict[str, Any]:
        principal = data.get("principal", 0)
        annual_rate = data.get("annual_rate", 0.20)
        months = data.get("months", 12)
        monthly_rate = annual_rate / 12
        if monthly_rate == 0:
            emi = principal / months if months else 0
        else:
            emi = principal * monthly_rate * (1 + monthly_rate) ** months / ((1 + monthly_rate) ** months - 1)
        schedule = []
        balance = principal
        for m in range(1, months + 1):
            interest = balance * monthly_rate
            repayment = min(emi, balance + interest)
            principal_part = repayment - interest
            balance -= principal_part
            schedule.append({"month": m, "emi": round(repayment, 2), "interest": round(interest, 2), "principal": round(principal_part, 2), "balance": round(max(balance, 0), 2)})
        return {"emi": round(emi, 2), "total_payment": round(emi * months, 2), "total_interest": round(emi * months - principal, 2), "schedule": schedule}

    def health_check(self) -> bool:
        return True
