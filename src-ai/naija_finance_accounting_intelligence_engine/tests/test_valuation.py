# Author: Quadri Atharu
"""Pytest tests for valuation models — NPV, IRR, WACC, DCF, and investment feasibility."""

import pytest

from ..valuation.npv import NpvEngine
from ..valuation.irr import IrrEngine
from ..valuation.wacc import WaccEngine
from ..valuation.dcf import DcfEngine
from ..valuation.investment_feasibility import InvestmentFeasibilityEngine


@pytest.fixture
def npv_engine():
    return NpvEngine()


@pytest.fixture
def irr_engine():
    return IrrEngine()


@pytest.fixture
def wacc_engine():
    return WaccEngine()


@pytest.fixture
def dcf_engine():
    return DcfEngine()


@pytest.fixture
def feasibility_engine():
    return InvestmentFeasibilityEngine()


@pytest.fixture
def nigerian_infrastructure_project():
    return {
        "cash_flows": [500000000, 750000000, 900000000, 1100000000, 1300000000],
        "discount_rate": 0.15,
        "initial_investment": 2000000000,
    }


class TestNPVCalculation:
    def test_npv_calculation(self, npv_engine, nigerian_infrastructure_project):
        result = npv_engine.compute_npv(nigerian_infrastructure_project)
        assert result["npv"] is not None
        assert result["discount_rate"] == 0.15
        assert result["total_periods"] == 5
        assert result["pv_of_cash_flows"] > 0
        assert len(result["pv_details"]) == 5
        assert result["profitability_index"] is not None

    def test_npv_positive_accept(self, npv_engine):
        data = {
            "cash_flows": [300000000, 400000000, 500000000],
            "discount_rate": 0.10,
            "initial_investment": 800000000,
        }
        result = npv_engine.compute_npv(data)
        assert result["npv"] > 0
        assert "ACCEPT" in result["recommendation"]

    def test_npv_negative_reject(self, npv_engine):
        data = {
            "cash_flows": [50000000, 60000000, 70000000],
            "discount_rate": 0.20,
            "initial_investment": 500000000,
        }
        result = npv_engine.compute_npv(data)
        assert result["npv"] < 0
        assert "REJECT" in result["recommendation"]

    def test_npv_sensitivity(self, npv_engine):
        data = {
            "cash_flows": [500000000, 750000000, 900000000],
            "initial_investment": 1500000000,
            "discount_rate": 0.12,
            "rate_range": 0.05,
        }
        result = npv_engine.compute_npv_sensitivity(data)
        assert len(result["sensitivities"]) == 11
        assert result["base_rate"] == 0.12


class TestIRRConvergence:
    def test_irr_convergence(self, irr_engine):
        data = {
            "cash_flows": [-2000000000, 500000000, 750000000, 900000000, 1100000000, 1300000000],
        }
        result = irr_engine.compute_irr(data)
        assert result["irr"] is not None
        assert result["irr"] > 0
        assert abs(result["npv_at_irr"]) < 1.0

    def test_irr_insufficient_flows(self, irr_engine):
        data = {"cash_flows": [-100000000]}
        result = irr_engine.compute_irr(data)
        assert result["irr"] is None

    def test_mirr_computation(self, irr_engine):
        data = {
            "cash_flows": [-1500000000, 400000000, 600000000, 800000000, 500000000],
            "finance_rate": 0.15,
            "reinvest_rate": 0.10,
        }
        result = irr_engine.compute_mirr(data)
        assert result["mirr"] is not None
        assert result["mirr"] > 0


class TestWACCComputation:
    def test_wacc_computation(self, wacc_engine):
        data = {
            "market_cap": 500000000000,
            "total_debt": 200000000000,
            "cost_of_equity": 0.18,
            "cost_of_debt_pretax": 0.14,
            "tax_rate": 0.30,
            "cash": 50000000000,
        }
        result = wacc_engine.compute_wacc(data)
        assert result["enterprise_value"] == 650000000000
        assert 0 < result["wacc"] < 1
        assert abs(result["equity_weight"] + result["debt_weight"] - 1.0) < 0.02
        assert "wacc_pct" in result

    def test_capm_cost_of_equity(self, wacc_engine):
        data = {
            "risk_free_rate": 0.15,
            "beta": 1.2,
            "market_return": 0.22,
            "size_premium": 0.02,
            "country_risk_premium": 0.04,
        }
        result = wacc_engine.compute_cost_of_equity_capm(data)
        expected_equity_risk_premium = 0.22 - 0.15
        expected_cost = 0.15 + 1.2 * expected_equity_risk_premium + 0.02 + 0.04
        assert abs(result["cost_of_equity"] - expected_cost) < 0.001
        assert result["model"] == "CAPM + Adjustments"


class TestDCFTerminalValue:
    def test_dcf_terminal_value(self, dcf_engine):
        data = {
            "cash_flows": [800000000, 1000000000, 1200000000, 1500000000],
            "discount_rate": 0.12,
            "terminal_growth_rate": 0.03,
            "net_debt": 3000000000,
            "shares_outstanding": 5000000000,
        }
        result = dcf_engine.compute_dcf(data)
        assert result["enterprise_value"] > 0
        assert result["equity_value"] == result["enterprise_value"] - 3000000000
        assert result["terminal_value"] > 0
        assert result["pv_of_terminal_value"] > 0
        assert result["terminal_value_pct_of_ev"] > 0
        assert result["value_per_share"] > 0

    def test_dcf_exit_multiple(self, dcf_engine):
        data = {
            "cash_flows": [600000000, 800000000, 1000000000],
            "discount_rate": 0.10,
            "terminal_growth_rate": 0.02,
            "terminal_value_method": "exit_multiple",
            "exit_multiple": 10,
            "net_debt": 0,
            "shares_outstanding": 1000000000,
        }
        result = dcf_engine.compute_dcf(data)
        assert result["terminal_value_method"] == "exit_multiple"
        assert result["terminal_value"] == 10000000000
        assert result["value_per_share"] > 0


class TestInvestmentFeasibility:
    def test_investment_feasibility(self, feasibility_engine):
        data = {
            "project_name": "Lagos-Ibadan Expressway Toll Concession",
            "initial_investment": 5000000000,
            "cash_flows": [800000000, 1200000000, 1500000000, 1800000000, 2000000000],
            "discount_rate": 0.12,
        }
        result = feasibility_engine.compute_feasibility(data)
        assert result["project_name"] == "Lagos-Ibadan Expressway Toll Concession"
        assert result["payback_period_years"] is not None
        assert result["payback_period_years"] > 0
        assert result["npv"] is not None
        assert result["profitability_index"] is not None
        assert isinstance(result["feasible"], bool)

    def test_investment_not_feasible(self, feasibility_engine):
        data = {
            "project_name": "Unprofitable Venture",
            "initial_investment": 10000000000,
            "cash_flows": [200000000, 300000000, 250000000],
            "discount_rate": 0.15,
        }
        result = feasibility_engine.compute_feasibility(data)
        assert result["npv"] < 0
        assert result["feasible"] is False

    def test_discounted_payback(self, feasibility_engine):
        data = {
            "project_name": "Quick Payback Project",
            "initial_investment": 1000000000,
            "cash_flows": [400000000, 500000000, 300000000, 200000000],
            "discount_rate": 0.10,
        }
        result = feasibility_engine.compute_feasibility(data)
        assert result["payback_period_years"] is not None
        assert result["discounted_payback_years"] is not None
        assert result["discounted_payback_years"] >= result["payback_period_years"]
