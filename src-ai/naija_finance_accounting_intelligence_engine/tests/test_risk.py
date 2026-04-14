# Author: Quadri Atharu
"""Pytest tests for risk management — credit risk, probability of default, VaR, liquidity, and market risk."""

import pytest

from ..risk_management.credit_risk import CreditRiskEngine, ECL_PROVISION_RATES
from ..risk_management.market_risk import MarketRiskEngine
from ..risk_management.liquidity_risk import LiquidityRiskEngine


@pytest.fixture
def credit_risk_engine():
    return CreditRiskEngine()


@pytest.fixture
def market_risk_engine():
    return MarketRiskEngine()


@pytest.fixture
def liquidity_risk_engine():
    return LiquidityRiskEngine()


@pytest.fixture
def nigerian_receivables():
    return [
        {"counterparty": "Dangote Cement", "amount": 5000000000, "days_overdue": 10, "is_defaulted": False},
        {"counterparty": "NNPC Retail", "amount": 3000000000, "days_overdue": 60, "is_defaulted": False},
        {"counterparty": "DSC Steel", "amount": 1500000000, "days_overdue": 200, "is_defaulted": True},
        {"counterparty": "MTN Nigeria", "amount": 2000000000, "days_overdue": 5, "is_defaulted": False},
    ]


class TestCreditRiskScoring:
    def test_credit_risk_scoring(self, credit_risk_engine):
        result = credit_risk_engine.rate_counterparty({
            "name": "Dangote Industries",
            "score_payment_history": 9,
            "score_financial_strength": 8,
            "score_industry_risk": 7,
            "score_country_risk": 5,
            "score_relationship_length": 8,
        })
        assert result["rating"] in ("AAA", "AA", "A", "BBB", "BB", "B", "CCC")
        assert result["weighted_score"] > 0
        assert result["risk_level"] in ("minimal", "low", "low_moderate", "moderate", "high_moderate", "high", "very_high")

    def test_counterparty_with_poor_scores(self, credit_risk_engine):
        result = credit_risk_engine.rate_counterparty({
            "name": "Distressed Corp",
            "score_payment_history": 2,
            "score_financial_strength": 2,
            "score_industry_risk": 3,
            "score_country_risk": 4,
            "score_relationship_length": 2,
        })
        assert result["rating"] in ("B", "CCC")
        assert result["risk_level"] in ("high", "very_high")


class TestProbabilityOfDefault:
    def test_probability_of_default(self, credit_risk_engine, nigerian_receivables):
        result = credit_risk_engine.compute_ecl({"receivables": nigerian_receivables})
        assert result["total_ecl"] > 0
        assert result["total_exposure"] == 11500000000
        assert result["standard"] == "IFRS 9"

        details = result["details"]
        dangote = next(d for d in details if d["counterparty"] == "Dangote Cement")
        assert dangote["stage"] == 1
        assert dangote["provision_rate"] == ECL_PROVISION_RATES["stage_1"]

    def test_loss_given_default(self, credit_risk_engine, nigerian_receivables):
        result = credit_risk_engine.compute_ecl({"receivables": nigerian_receivables})

        dsc = next(d for d in result["details"] if d["counterparty"] == "DSC Steel")
        assert dsc["stage"] == 3
        assert dsc["provision_rate"] == ECL_PROVISION_RATES["stage_3"]
        expected_ecl = 1500000000 * ECL_PROVISION_RATES["stage_3"]
        assert abs(dsc["ecl"] - expected_ecl) < 1

        nnpc = next(d for d in result["details"] if d["counterparty"] == "NNPC Retail")
        assert nnpc["stage"] == 2
        assert nnpc["provision_rate"] == ECL_PROVISION_RATES["stage_2"]

    def test_ecl_stage_totals(self, credit_risk_engine, nigerian_receivables):
        result = credit_risk_engine.compute_ecl({"receivables": nigerian_receivables})

        assert result["stage_1"]["exposure"] == 7000000000
        assert result["stage_2"]["exposure"] == 3000000000
        assert result["stage_3"]["exposure"] == 1500000000

        total_ecl = result["stage_1"]["ecl"] + result["stage_2"]["ecl"] + result["stage_3"]["ecl"]
        assert abs(total_ecl - result["total_ecl"]) < 1


class TestVaRCalculation:
    def test_var_calculation(self, market_risk_engine):
        data = {
            "portfolio_value": 100000000000,
            "daily_volatility": 0.015,
            "confidence_level": 0.95,
            "horizon_days": 1,
        }
        result = market_risk_engine.compute_var(data)
        assert result["var_daily"] > 0
        assert result["var_daily"] == 100000000000 * 0.015 * 1.645
        assert result["z_score"] == 1.645
        assert result["method"] == "parametric"

    def test_var_10_day_horizon(self, market_risk_engine):
        data = {
            "portfolio_value": 50000000000,
            "daily_volatility": 0.02,
            "confidence_level": 0.99,
            "horizon_days": 10,
        }
        result = market_risk_engine.compute_var(data)
        var_daily = 50000000000 * 0.02 * 2.33
        var_10day = var_daily * (10 ** 0.5)
        assert abs(result["var_daily"] - var_daily) < 1
        assert abs(result["var_horizon"] - var_10day) < 1

    def test_cvar_calculation(self, market_risk_engine):
        data = {
            "portfolio_value": 80000000000,
            "daily_volatility": 0.018,
            "confidence_level": 0.95,
        }
        result = market_risk_engine.compute_cvar(data)
        assert result["cvar"] > 0
        assert result["cvar"] >= result["var"]
        assert result["cvar_var_ratio"] >= 1.0


class TestLiquidityRiskRatio:
    def test_liquidity_risk_ratio(self, liquidity_risk_engine):
        data = {
            "current_cash": 50000000000,
            "monthly_burn_rate": 8000000000,
            "committed_outflows": 5000000000,
            "undrawn_facilities": 30000000000,
            "liquid_assets": 20000000000,
        }
        result = liquidity_risk_engine.stress_test_liquidity(data)
        assert result["current_position"]["total_liquid_resources"] == 100000000000
        assert result["current_position"]["months_of_survival"] > 0

        normal = result["stress_scenarios"]["normal"]
        assert normal["passes_lcr_threshold"] is True

    def test_liquidity_stress_scenarios(self, liquidity_risk_engine):
        data = {
            "current_cash": 5000000000,
            "monthly_burn_rate": 4000000000,
            "committed_outflows": 2000000000,
            "undrawn_facilities": 1000000000,
            "liquid_assets": 2000000000,
        }
        result = liquidity_risk_engine.stress_test_liquidity(data)
        assert "normal" in result["stress_scenarios"]
        assert "moderate" in result["stress_scenarios"]
        assert "severe" in result["stress_scenarios"]
        assert "extreme" in result["stress_scenarios"]
        assert result["overall_risk"] in ("low", "moderate", "high", "critical")


class TestMarketRiskSensitivity:
    def test_market_risk_sensitivity(self, market_risk_engine):
        data = {
            "portfolio_value": 200000000000,
            "daily_volatility": 0.025,
            "confidence_level": 0.95,
            "horizon_days": 1,
        }
        result = market_risk_engine.compute_var(data)
        var_95 = result["var_daily"]

        data_99 = {**data, "confidence_level": 0.99}
        result_99 = market_risk_engine.compute_var(data_99)
        var_99 = result_99["var_daily"]

        assert var_99 > var_95

        data_low_vol = {**data, "daily_volatility": 0.01}
        result_low = market_risk_engine.compute_var(data_low_vol)
        assert result_low["var_daily"] < var_95
