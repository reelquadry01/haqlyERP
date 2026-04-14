# Author: Quadri Atharu
"""Pytest tests for Nigerian tax computation accuracy."""

import pytest

from ..tax_engine.vat import VatEngine, VAT_EXEMPT_ITEMS
from ..tax_engine.wht import WhtEngine
from ..tax_engine.cit import CitEngine
from ..tax_engine.education_tax import EducationTaxEngine
from ..tax_engine.capital_gains_tax import CapitalGainsTaxEngine


class TestVATComputation:
    @pytest.fixture
    def vat_engine(self):
        return VatEngine()

    def test_vat_7_5_percent(self, vat_engine):
        result = vat_engine.compute_output_vat(100000)
        assert result["vat_amount"] == 7500.0
        assert result["vat_rate"] == 0.075
        assert result["category"] == "standard"

    def test_vat_exempt_items(self, vat_engine):
        result = vat_engine.compute_output_vat(50000, item_category="medical_services")
        assert result["vat_amount"] == 0.0
        assert result["category"] == "exempt"

    def test_vat_zero_rated_items(self, vat_engine):
        result = vat_engine.compute_output_vat(200000, item_category="exported_goods")
        assert result["vat_amount"] == 0.0
        assert result["category"] == "zero_rated"

    def test_vat_custom_rate(self, vat_engine):
        result = vat_engine.compute_output_vat(100000, rate=0.05)
        assert result["vat_amount"] == 5000.0

    def test_vat_payable(self, vat_engine):
        result = vat_engine.compute_vat_payable(output_vat=75000, input_vat=30000)
        assert result["net_vat"] == 45000
        assert result["direction"] == "payable"

    def test_vat_refund(self, vat_engine):
        result = vat_engine.compute_vat_payable(output_vat=20000, input_vat=50000)
        assert result["direction"] == "refund"

    def test_vat_inclusive_extraction(self, vat_engine):
        result = vat_engine.compute_vat_from_inclusive_amount(107500)
        assert result["vat_amount"] == 7500.0
        assert result["net_amount"] == 100000.0


class TestWHTComputation:
    @pytest.fixture
    def wht_engine(self):
        return WhtEngine()

    def test_wht_contractors(self, wht_engine):
        result = wht_engine.compute_wht(100000, category="contractors")
        assert result["wht_amount"] == 5000.0
        assert result["wht_rate"] == 0.05
        assert result["is_final_tax"] is False

    def test_wht_interest(self, wht_engine):
        result = wht_engine.compute_wht(100000, category="interest")
        assert result["wht_amount"] == 10000.0
        assert result["wht_rate"] == 0.10
        assert result["is_final_tax"] is True

    def test_wht_dividends(self, wht_engine):
        result = wht_engine.compute_wht(500000, category="dividends")
        assert result["wht_amount"] == 50000.0
        assert result["wht_rate"] == 0.10

    def test_wht_rent(self, wht_engine):
        result = wht_engine.compute_wht(200000, category="rent")
        assert result["wht_amount"] == 20000.0

    def test_wht_net_payment(self, wht_engine):
        result = wht_engine.compute_wht(100000, category="consultancy")
        assert result["net_payment"] == 95000.0

    def test_wht_tax_credit_available_for_5pct(self, wht_engine):
        result = wht_engine.compute_wht(100000, category="contractors")
        assert result["tax_credit_available"] is True

    def test_wht_tax_credit_not_available_for_10pct(self, wht_engine):
        result = wht_engine.compute_wht(100000, category="interest")
        assert result["tax_credit_available"] is False


class TestCITComputation:
    @pytest.fixture
    def cit_engine(self):
        return CitEngine()

    def test_cit_small_company(self, cit_engine):
        result = cit_engine.compute_cit(
            profit_before_tax=500000,
            turnover=200000,
            industry="general",
        )
        assert result["cit_payable"] == 0.0
        assert "small" in result["applicable_bracket"].lower()

    def test_cit_large_company(self, cit_engine):
        result = cit_engine.compute_cit(
            profit_before_tax=50000000,
            turnover=5000000000,
            industry="general",
        )
        assert result["applicable_rate"] == 0.30

    def test_cit_with_capital_allowances(self, cit_engine):
        result = cit_engine.compute_cit(
            profit_before_tax=10000000,
            turnover=2000000000,
            industry="manufacturing",
            is_manufacturing=True,
            capital_allowances=[
                {"asset_type": "plant_and_machinery", "cost": 5000000, "residual_value": 500000, "is_first_year": True, "years_claimed": 0},
            ],
        )
        assert result["capital_allowance_total"] > 0
        assert result["adjusted_profit"] < result["assessable_profit"]

    def test_minimum_tax_when_cit_below_minimum(self, cit_engine):
        result = cit_engine.compute_cit(
            profit_before_tax=100,
            turnover=100000000,
        )
        minimum_tax = 100000000 * 0.005
        assert result["cit_payable"] >= minimum_tax


class TestEducationTax:
    @pytest.fixture
    def edu_engine(self):
        return EducationTaxEngine()

    def test_education_tax_2_percent(self, edu_engine):
        result = edu_engine.compute_education_tax(1000000)
        assert result["education_tax"] == 20000.0
        assert result["rate"] == 0.02

    def test_education_tax_zero_profit(self, edu_engine):
        result = edu_engine.compute_education_tax(0)
        assert result["education_tax"] == 0.0


class TestCGT:
    @pytest.fixture
    def cgt_engine(self):
        return CapitalGainsTaxEngine()

    def test_cgt_10_percent(self, cgt_engine):
        chargeable_gain = 5000000
        result = cgt_engine.compute_cgt(
            disposal_proceeds=10000000,
            cost_basis=5000000,
        )
        assert result["chargeable_gain"] == chargeable_gain
        assert result["cgt_amount"] == chargeable_gain * 0.10

    def test_cgt_exempt_disposal(self, cgt_engine):
        result = cgt_engine.compute_cgt(
            disposal_proceeds=20000000,
            cost_basis=5000000,
            is_exempt=True,
            exemption_type="personal_residence",
        )
        assert result["cgt_amount"] == 0.0
        assert result["is_exempt"] is True

    def test_cgt_no_gain(self, cgt_engine):
        result = cgt_engine.compute_cgt(
            disposal_proceeds=3000000,
            cost_basis=5000000,
        )
        assert result["cgt_amount"] == 0.0
        assert result["is_loss"] is True
