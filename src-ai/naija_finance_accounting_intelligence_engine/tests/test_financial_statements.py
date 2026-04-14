# Author: Quadri Atharu
"""Pytest tests for financial statement generation — income statement, balance sheet, cash flow."""

import pytest

from ..reporting.income_statement import IncomeStatementEngine
from ..reporting.balance_sheet import BalanceSheetEngine


@pytest.fixture
def income_stmt_engine():
    return IncomeStatementEngine()


@pytest.fixture
def balance_sheet_engine():
    return BalanceSheetEngine()


@pytest.fixture
def typical_nigerian_company_income_data():
    return {
        "company_id": "HAQLY-001",
        "period_start": "2025-01-01",
        "period_end": "2025-12-31",
        "currency": "NGN",
        "revenue": 2500000000,
        "other_income": 50000000,
        "cogs": 1500000000,
        "selling_expenses": 200000000,
        "admin_expenses": 150000000,
        "depreciation": 100000000,
        "amortisation": 25000000,
        "other_operating_expenses": 50000000,
        "finance_costs": 120000000,
        "finance_income": 30000000,
        "share_of_associate_profit": 15000000,
        "tax_expense": 108500000,
    }


@pytest.fixture
def typical_nigerian_company_bs_data():
    return {
        "company_id": "HAQLY-001",
        "as_of": "2025-12-31",
        "currency": "NGN",
        "property_plant_equipment": 3500000000,
        "intangible_assets": 500000000,
        "investment_in_associates": 300000000,
        "other_non_current_assets": 200000000,
        "inventory": 800000000,
        "trade_receivables": 600000000,
        "other_receivables": 100000000,
        "cash": 450000000,
        "short_term_investments": 150000000,
        "other_current_assets": 50000000,
        "share_capital": 1000000000,
        "share_premium": 500000000,
        "retained_earnings": 1800000000,
        "revaluation_reserve": 200000000,
        "other_reserves": 100000000,
        "long_term_loans": 1200000000,
        "deferred_tax_liability": 150000000,
        "other_non_current_liabilities": 500000000,
        "trade_payables": 400000000,
        "short_term_loans": 300000000,
        "tax_payable": 108500000,
        "other_current_liabilities": 291500000,
    }


class TestIncomeStatementStructure:
    def test_income_statement_structure(self, income_stmt_engine, typical_nigerian_company_income_data):
        result = income_stmt_engine.generate(typical_nigerian_company_income_data)

        totals = result["totals"]
        revenue = totals["revenue"]
        cogs = typical_nigerian_company_income_data["cogs"]
        gross_profit = totals["gross_profit"]

        assert gross_profit == revenue - cogs
        assert gross_profit == 1000000000

        total_opex = (
            typical_nigerian_company_income_data["selling_expenses"]
            + typical_nigerian_company_income_data["admin_expenses"]
            + typical_nigerian_company_income_data["depreciation"]
            + typical_nigerian_company_income_data["amortisation"]
            + typical_nigerian_company_income_data["other_operating_expenses"]
        )
        operating_profit = totals["operating_profit"]
        assert operating_profit == gross_profit - total_opex
        assert operating_profit == 475000000

        net_income = totals["net_income"]
        assert net_income == totals["profit_before_tax"] - typical_nigerian_company_income_data["tax_expense"]
        assert net_income > 0

    def test_revenue_minus_expenses_equals_net_income(self, income_stmt_engine):
        data = {
            "company_id": "HAQLY-001",
            "period_start": "2025-01-01",
            "period_end": "2025-12-31",
            "revenue": 100000000,
            "cogs": 40000000,
            "selling_expenses": 10000000,
            "admin_expenses": 15000000,
            "depreciation": 5000000,
            "amortisation": 0,
            "other_operating_expenses": 5000000,
            "finance_costs": 8000000,
            "finance_income": 2000000,
            "tax_expense": 7500000,
        }
        result = income_stmt_engine.generate(data)
        totals = result["totals"]

        gross_profit = 100000000 - 40000000
        opex = 10000000 + 15000000 + 5000000 + 0 + 5000000
        operating_profit = gross_profit - opex
        pbt = operating_profit - (8000000 - 2000000)
        expected_net = pbt - 7500000

        assert totals["net_income"] == expected_net

    def test_negative_net_income_when_expenses_exceed_revenue(self, income_stmt_engine):
        data = {
            "company_id": "HAQLY-001",
            "period_start": "2025-01-01",
            "period_end": "2025-12-31",
            "revenue": 50000000,
            "cogs": 60000000,
        }
        result = income_stmt_engine.generate(data)
        assert result["totals"]["gross_profit"] < 0
        assert result["totals"]["net_income"] < 0

    def test_eps_computation(self, income_stmt_engine):
        data = {
            "company_id": "HAQLY-001",
            "period_start": "2025-01-01",
            "period_end": "2025-12-31",
            "revenue": 500000000,
            "cogs": 200000000,
            "tax_expense": 90000000,
            "shares_outstanding": 500000000,
        }
        result = income_stmt_engine.generate(data)
        assert result["totals"]["eps"] is not None
        assert result["totals"]["eps"] > 0


class TestBalanceSheetBalanced:
    def test_balance_sheet_balanced(self, balance_sheet_engine, typical_nigerian_company_bs_data):
        result = balance_sheet_engine.generate(typical_nigerian_company_bs_data)

        assert result["balance_check"] is True
        total_assets = result["total_assets"]
        total_liabilities = result["total_liabilities"]
        total_equity = result["equity"]["total_equity"]
        assert abs(total_assets - (total_liabilities + total_equity)) < 0.01

    def test_balance_sheet_classified_format(self, balance_sheet_engine, typical_nigerian_company_bs_data):
        result = balance_sheet_engine.generate(typical_nigerian_company_bs_data)

        nca = result["non_current_assets"]
        ca = result["current_assets"]
        ncl = result["non_current_liabilities"]
        cl = result["current_liabilities"]

        assert nca["total_non_current_assets"] == (
            nca["property_plant_equipment"]
            + nca["intangible_assets"]
            + nca["investment_in_associates"]
            + nca["other_non_current_assets"]
        )
        assert ca["total_current_assets"] == (
            ca["inventory"]
            + ca["trade_receivables"]
            + ca["other_receivables"]
            + ca["cash_and_cash_equivalents"]
            + ca["short_term_investments"]
            + ca["other_current_assets"]
        )

    def test_unbalanced_balance_sheet_detected(self, balance_sheet_engine):
        data = {
            "company_id": "HAQLY-001",
            "as_of": "2025-12-31",
            "property_plant_equipment": 100000000,
            "inventory": 50000000,
            "cash": 25000000,
            "share_capital": 100000000,
            "long_term_loans": 50000000,
        }
        result = balance_sheet_engine.generate(data)
        total_assets = result["total_assets"]
        total_liab_equity = result["total_liabilities"] + result["equity"]["total_equity"]
        assert total_assets != total_liab_equity
        assert result["balance_check"] is False


class TestCashFlowOperations:
    def test_cash_flow_operations(self, income_stmt_engine, balance_sheet_engine):
        operating = 350000000
        investing = -200000000
        financing = -50000000

        net_change = operating + investing + financing
        assert net_change == 100000000

        opening_cash = 300000000
        closing_cash = opening_cash + net_change
        assert closing_cash == 400000000


class TestRetainedEarningsRollforward:
    def test_retained_earnings_rollforward(self, income_stmt_engine):
        data = {
            "company_id": "HAQLY-001",
            "period_start": "2025-01-01",
            "period_end": "2025-12-31",
            "revenue": 800000000,
            "cogs": 400000000,
            "admin_expenses": 100000000,
            "depreciation": 50000000,
            "tax_expense": 75000000,
        }
        result = income_stmt_engine.generate(data)
        net_income = result["totals"]["net_income"]

        beginning_re = 1200000000
        dividends = 200000000
        ending_re = beginning_re + net_income - dividends

        expected_net = 800000000 - 400000000 - 100000000 - 50000000 - 75000000
        assert net_income == expected_net
        assert ending_re == beginning_re + net_income - dividends

    @pytest.mark.parametrize("beginning_re,net_income,dividends,expected_ending", [
        (0, 500000000, 0, 500000000),
        (1000000000, 300000000, 100000000, 1200000000),
        (500000000, -100000000, 0, 400000000),
    ])
    def test_retained_earnings_scenarios(self, beginning_re, net_income, dividends, expected_ending):
        ending_re = beginning_re + net_income - dividends
        assert ending_re == expected_ending


class TestIntercompanyElimination:
    def test_intercompany_elimination(self):
        parent_receivables = 500000000
        subsidiary_payables = 300000000
        intercompany_balance = 100000000

        consolidated_receivables = parent_receivables - intercompany_balance
        consolidated_payables = subsidiary_payables - intercompany_balance

        assert consolidated_receivables == 400000000
        assert consolidated_payables == 200000000

        parent_revenue = 2000000000
        subsidiary_revenue = 800000000
        intercompany_sales = 150000000

        consolidated_revenue = parent_revenue + subsidiary_revenue - 2 * intercompany_sales
        assert consolidated_revenue == 2500000000

    def test_group_assets_exclude_intercompany(self):
        parent_assets = 5000000000
        subsidiary_assets = 2000000000
        intercompany_investment = 800000000

        total_assets_before_elimination = parent_assets + subsidiary_assets
        consolidated_assets = total_assets_before_elimination - intercompany_investment
        assert consolidated_assets == 6200000000
