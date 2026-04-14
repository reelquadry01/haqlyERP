# Author: Quadri Atharu
"""Pytest tests for industry profiles — oil & gas, manufacturing, banking, and profile completeness."""

import pytest
from decimal import Decimal

from ..industry_profiles import PROFILES
from ..industry_profiles.oil_gas import OilGasProfile
from ..industry_profiles.manufacturing import ManufacturingProfile
from ..industry_profiles.banking import BankingProfile


@pytest.fixture
def oil_gas_profile():
    return OilGasProfile()


@pytest.fixture
def manufacturing_profile():
    return ManufacturingProfile()


@pytest.fixture
def banking_profile():
    return BankingProfile()


class TestAllProfilesLoaded:
    def test_all_profiles_loaded(self):
        assert len(PROFILES) == 15
        expected_keys = [
            "oil_gas", "manufacturing", "banking", "insurance", "retail",
            "telecommunications", "agriculture", "construction", "logistics",
            "healthcare", "education", "government", "ngo", "technology", "automotive",
        ]
        for key in expected_keys:
            assert key in PROFILES, f"Missing profile: {key}"

    def test_all_profiles_instantiable(self):
        for key, cls in PROFILES.items():
            instance = cls()
            assert hasattr(instance, "name")
            assert instance.name != ""


class TestOilGasProfile:
    def test_oil_gas_profile(self, oil_gas_profile):
        assert oil_gas_profile.name == "Oil & Gas"
        assert "crude_oil_sales" in oil_gas_profile.typical_revenue_models
        assert "natural_gas_sales" in oil_gas_profile.typical_revenue_models
        assert oil_gas_profile.tax_implications["ppt_jv"] == Decimal("50")
        assert oil_gas_profile.tax_implications["ppt_psc"] == Decimal("85")

    def test_oil_gas_posting_suggestions(self, oil_gas_profile):
        result = oil_gas_profile.get_posting_suggestions({"type": "crude_sale"})
        assert len(result) == 1
        assert result[0]["credit_account"] == "Crude Oil Revenue"

        result2 = oil_gas_profile.get_posting_suggestions({"type": "royalty"})
        assert result2[0]["debit_account"] == "Royalty Expense"

    def test_oil_gas_kpis(self, oil_gas_profile):
        kpis = oil_gas_profile.get_industry_kpis({
            "additions_to_reserves": 50000000,
            "production": 40000000,
            "finding_cost_total": 500000000,
            "opex_total": 3000000000,
            "production_volume": 30000000,
            "realized_price": 75,
            "unit_cost": 30,
            "capex": 800000000,
            "proven_reserves": 500000000,
            "annual_production": 40000000,
        })
        assert "reserve_replacement_ratio_pct" in kpis
        assert "netback_usd_per_boe" in kpis
        assert kpis["netback_usd_per_boe"] == Decimal("45")
        assert float(kpis["reserve_life_index_years"]) > 0


class TestManufacturingProfile:
    def test_manufacturing_profile(self, manufacturing_profile):
        assert manufacturing_profile.name == "Manufacturing"
        assert "product_sales" in manufacturing_profile.typical_revenue_models
        assert "raw_materials" in manufacturing_profile.cost_structures
        assert manufacturing_profile.tax_implications["cit"] == Decimal("30")

    def test_manufacturing_posting_suggestions(self, manufacturing_profile):
        result = manufacturing_profile.get_posting_suggestions({"type": "product_sale"})
        assert len(result) == 2
        assert result[0]["credit_account"] == "Sales Revenue"
        assert result[1]["debit_account"] == "Cost of Goods Sold"

    def test_manufacturing_kpis(self, manufacturing_profile):
        kpis = manufacturing_profile.get_industry_kpis({
            "actual_output": 80000,
            "max_capacity": 100000,
            "total_cost": 500000000,
            "units_produced": 80000,
            "scrap_units": 2000,
            "total_units": 82000,
            "cogs": 450000000,
            "avg_inventory": 75000000,
            "availability_pct": 90,
            "performance_pct": 95,
            "quality_pct": 98,
        })
        assert float(kpis["capacity_utilization_pct"]) == 80.0
        assert float(kpis["inventory_turnover"]) > 0
        assert float(kpis["oee_pct"]) > 0


class TestBankingProfile:
    def test_banking_profile(self, banking_profile):
        assert banking_profile.name == "Banking"
        assert "interest_income" in banking_profile.typical_revenue_models
        assert "loan_loss_provision" in banking_profile.cost_structures
        assert banking_profile.inventory_logic == "N/A - Banks do not carry physical inventory"

    def test_banking_posting_suggestions(self, banking_profile):
        result = banking_profile.get_posting_suggestions({"type": "loan_disbursement"})
        assert result[0]["debit_account"] == "Loans & Advances"

        result2 = banking_profile.get_posting_suggestions({"type": "loan_loss_provision"})
        assert result2[0]["credit_account"] == "ECL Provision (IFRS 9)"

    def test_banking_kpis(self, banking_profile):
        kpis = banking_profile.get_industry_kpis({
            "total_loans": 200000000000,
            "total_deposits": 300000000000,
            "npl_balance": 15000000000,
            "gross_loans": 200000000000,
            "operating_income": 80000000000,
            "operating_expense": 40000000000,
            "interest_income": 60000000000,
            "interest_expense": 20000000000,
            "earning_assets": 250000000000,
        })
        assert float(kpis["loan_to_deposit_ratio_pct"]) == pytest.approx(66.67, rel=0.01)
        assert float(kpis["npl_ratio_pct"]) == 7.5
        assert float(kpis["cost_to_income_ratio_pct"]) == 50.0
        assert float(kpis["net_interest_margin_pct"]) == pytest.approx(16.0, rel=0.01)


class TestProfileCOACompleteness:
    def test_profile_coa_completeness(self):
        for key, cls in PROFILES.items():
            instance = cls()
            coa = instance.typical_coa_ranges
            assert len(coa) > 0, f"Profile '{key}' has empty COA ranges"
            for range_name, (lo, hi) in coa.items():
                assert lo < hi, f"Profile '{key}' COA range '{range_name}' has lo >= hi"


class TestProfileRevenuePatterns:
    def test_profile_revenue_patterns(self):
        for key, cls in PROFILES.items():
            instance = cls()
            rev_models = instance.typical_revenue_models
            assert len(rev_models) > 0, f"Profile '{key}' has no revenue models"
            for model_name, description in rev_models.items():
                assert isinstance(description, str)
                assert len(description) > 0
