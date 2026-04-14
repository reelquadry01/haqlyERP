# Author: Quadri Atharu
"""Pytest tests for IFRS compliance — IFRS 9, IFRS 15, IFRS 16, IAS 2, IAS 12."""

import pytest
from decimal import Decimal

from ..ifrs.ifrs9_financial_instruments import (
    classify_financial_asset,
    compute_expected_credit_loss,
    compute_impairment,
    validate_classification,
)
from ..ifrs.ifrs15_revenue import (
    identify_contract,
    identify_performance_obligations,
    determine_transaction_price,
    allocate_transaction_price,
    apply_five_step_model,
)
from ..ifrs.ifrs16_leases import (
    classify_lease,
    compute_lease_liability,
    compute_rou_asset,
    compute_lease_depreciation,
    compute_lease_interest,
)
from ..ifrs.ias2_inventory import (
    compute_inventory_cost,
    compute_nrv,
    apply_lower_of_cost_or_nrv,
    recognize_inventory_write_down,
)
from ..ifrs.ias12_income_taxes import (
    compute_deferred_tax_liability,
    compute_deferred_tax_asset,
    recognize_current_tax,
    compute_tax_expense,
    validate_recognition,
)


@pytest.fixture
def nigerian_loan_portfolio():
    return [
        {
            "name": "Dangote Cement Loan",
            "exposure_at_default": 5000000000,
            "probability_of_default_12m": 0.8,
            "probability_of_default_lifetime": 3.5,
            "loss_given_default": 45,
        },
        {
            "name": "NNPC Trade Facility",
            "exposure_at_default": 8000000000,
            "probability_of_default_12m": 0.5,
            "probability_of_default_lifetime": 2.0,
            "loss_given_default": 35,
        },
    ]


@pytest.fixture
def valid_contract_data():
    return {
        "contract_id": "CTR-2025-001",
        "customer": "Dangote Industries",
        "start_date": "2025-01-01",
        "end_date": "2025-12-31",
        "is_approved": True,
        "has_identifiable_rights": True,
        "payment_terms": "Net 30",
        "has_commercial_substance": True,
        "collectibility_probable": True,
        "obligations": [
            {
                "obligation_id": "OBL-001",
                "description": "Supply of cement — 50,000 tonnes",
                "stand_alone_price": 3750000000,
                "is_distinct": True,
                "satisfaction_method": "over_time",
            },
            {
                "obligation_id": "OBL-002",
                "description": "Delivery and logistics",
                "stand_alone_price": 500000000,
                "is_distinct": True,
                "satisfaction_method": "point_in_time",
            },
        ],
        "variable_consideration": 250000000,
    }


@pytest.fixture
def nigerian_lease_data():
    return {
        "lease_id": "LEASE-LAG-001",
        "term_months": 36,
        "has_purchase_option": False,
        "asset_value_new": Decimal("50000"),
        "transfer_ownership": False,
    }


class TestIFRS9ExpectedCreditLoss:
    def test_ifrs9_expected_credit_loss(self, nigerian_loan_portfolio):
        result = compute_expected_credit_loss(nigerian_loan_portfolio, stage=1)
        assert result.stage == 1
        assert result.lifetime_or_12_month == "12_month"
        assert result.ecl_amount > 0
        assert result.exposure_at_default == Decimal("13000000000")

    def test_ifrs9_stage_allocation(self, nigerian_loan_portfolio):
        stage1 = compute_expected_credit_loss(nigerian_loan_portfolio, stage=1)
        assert stage1.lifetime_or_12_month == "12_month"

        stage2 = compute_expected_credit_loss(nigerian_loan_portfolio, stage=2)
        assert stage2.lifetime_or_12_month == "lifetime"
        assert stage2.ecl_amount > stage1.ecl_amount

        stage3 = compute_expected_credit_loss(nigerian_loan_portfolio, stage=3)
        assert stage3.lifetime_or_12_month == "lifetime"
        assert stage3.ecl_amount > stage2.ecl_amount

        asset = {
            "name": "GTBank Term Loan",
            "carrying_amount": 2000000000,
            "exposure_at_default": 2000000000,
            "probability_of_default_12m": 1.2,
            "probability_of_default_lifetime": 5.0,
            "loss_given_default": 40,
        }
        imp1 = compute_impairment(asset, stage=1)
        imp3 = compute_impairment(asset, stage=3)
        assert imp3.impairment_amount > imp1.impairment_amount
        assert imp1.stage == 1
        assert imp3.stage == 3


class TestIFRS9Classification:
    def test_amortized_cost_classification(self):
        asset = {
            "name": "FGN Bond Portfolio",
            "business_model": "hold_to_collect",
            "sppi_test": True,
            "is_equity": False,
        }
        result = classify_financial_asset(asset)
        assert result.classification == "Amortized Cost"

    def test_fvtoci_classification(self):
        asset = {
            "name": "Treasury Bill Portfolio",
            "business_model": "hold_to_collect_and_sell",
            "sppi_test": True,
            "is_equity": False,
        }
        result = classify_financial_asset(asset)
        assert result.classification == "FVTOCI"

    def test_fvtpl_classification_sppi_fail(self):
        asset = {
            "name": "Structured Note",
            "business_model": "hold_to_collect",
            "sppi_test": False,
            "is_equity": False,
        }
        result = classify_financial_asset(asset)
        assert result.classification == "FVTPL"

    def test_equity_fvtpl_default(self):
        asset = {
            "name": "Nigerian Breweries Shares",
            "business_model": "trading",
            "sppi_test": False,
            "is_equity": True,
            "irrevocable_fvoci_election": False,
        }
        result = classify_financial_asset(asset)
        assert result.classification == "FVTPL"

    def test_classification_validation(self):
        validation = validate_classification(
            {"name": "Loan", "is_equity": False},
            business_model="hold_to_collect",
            sppi_test=True,
        )
        assert validation.is_valid is True

        validation_bad = validate_classification(
            {"name": "Equity", "is_equity": True},
            business_model="hold_to_collect",
            sppi_test=True,
        )
        assert validation_bad.is_valid is False


class TestIFRS15PerformanceObligation:
    def test_ifrs15_performance_obligation(self, valid_contract_data):
        contract = identify_contract(valid_contract_data)
        obligations = identify_performance_obligations(
            contract,
            valid_contract_data["obligations"],
        )
        assert len(obligations) == 2
        assert obligations[0].is_distinct is True
        assert obligations[0].stand_alone_price == Decimal("3750000000")
        assert obligations[1].satisfaction_method == "point_in_time"

    def test_ifrs15_variable_consideration(self, valid_contract_data):
        contract = identify_contract(valid_contract_data)
        obligations = identify_performance_obligations(
            contract,
            valid_contract_data["obligations"],
        )
        base_price = determine_transaction_price(contract, obligations)
        assert base_price == Decimal("4250000000")

        variable = Decimal(valid_contract_data.get("variable_consideration", 0))
        total_price = base_price + variable
        assert total_price == Decimal("4500000000")

        allocations = allocate_transaction_price(obligations, total_price)
        total_allocated = sum(a.allocated_amount for a in allocations)
        assert abs(total_allocated - Decimal("4500000000")) < Decimal("0.01")

    def test_ifrs15_five_step_model(self, valid_contract_data):
        result = apply_five_step_model(valid_contract_data)
        assert result.contract_id == "CTR-2025-001"
        assert result.total_transaction_price == Decimal("4500000000")
        assert len(result.allocations) == 2
        assert len(result.revenue_entries) == 2

    def test_ifrs15_contract_rejected_when_criteria_unmet(self):
        with pytest.raises(ValueError, match="Contract criteria not met"):
            identify_contract({
                "contract_id": "BAD-001",
                "customer": "Unknown",
                "start_date": "2025-01-01",
                "end_date": "2025-12-31",
                "is_approved": False,
                "has_identifiable_rights": True,
                "payment_terms": "Net 30",
                "has_commercial_substance": True,
                "collectibility_probable": True,
            })


class TestIFRS16LeaseLiability:
    def test_ifrs16_lease_liability(self):
        payments = [Decimal("15000000")] * 3
        schedule = compute_lease_liability(payments, Decimal("10"), 3)
        assert len(schedule) == 3
        assert schedule[0].opening_balance > 0
        assert schedule[0].interest > 0
        assert schedule[-1].closing_balance == Decimal("0")

    def test_ifrs16_lease_classification(self, nigerian_lease_data):
        result = classify_lease(nigerian_lease_data)
        assert result.classification == "on_balance_sheet"
        assert result.is_short_term is False
        assert result.is_low_value is False

    def test_ifrs16_short_term_exemption(self):
        lease = {
            "lease_id": "ST-001",
            "term_months": 6,
            "has_purchase_option": False,
            "asset_value_new": Decimal("200000"),
            "transfer_ownership": False,
        }
        result = classify_lease(lease)
        assert result.classification == "short_term_exemption"
        assert result.is_short_term is True

    def test_ifrs16_low_value_exemption(self):
        lease = {
            "lease_id": "LV-001",
            "term_months": 24,
            "has_purchase_option": False,
            "asset_value_new": Decimal("3000"),
            "transfer_ownership": False,
        }
        result = classify_lease(lease)
        assert result.classification == "low_value_exemption"
        assert result.is_low_value is True

    def test_ifrs16_lease_modification(self):
        payments = [Decimal("10000000")] * 5
        schedule = compute_lease_liability(payments, Decimal("8"), 5)
        rou = compute_rou_asset(schedule[0].opening_balance, Decimal("500000"), Decimal("0"))
        dep_schedule = compute_lease_depreciation(rou, 5)
        total_dep = sum(d.depreciation for d in dep_schedule)
        assert abs(total_dep - rou) < Decimal("1")

    def test_ifrs16_lease_interest(self):
        interest = compute_lease_interest(Decimal("40000000"), Decimal("10"))
        assert interest == Decimal("4000000")


class TestIAS2InventoryValuation:
    def test_ias2_inventory_valuation_fifo(self):
        items = [
            {"name": "Cement Grade A", "quantity": 5000, "unit_cost": 4500, "purchase_order": 1},
            {"name": "Cement Grade B", "quantity": 3000, "unit_cost": 4200, "purchase_order": 2},
        ]
        result = compute_inventory_cost(items, "FIFO")
        expected_total = Decimal("5000") * Decimal("4500") + Decimal("3000") * Decimal("4200")
        assert result.total_cost == expected_total
        assert result.method == "FIFO"

    def test_ias2_weighted_average(self):
        items = [
            {"name": "Cement", "quantity": 5000, "unit_cost": 4500},
            {"name": "Cement", "quantity": 3000, "unit_cost": 4200},
        ]
        result = compute_inventory_cost(items, "WeightedAverage")
        avg_unit = (Decimal("5000") * Decimal("4500") + Decimal("3000") * Decimal("4200")) / Decimal("8000")
        assert result.items[0]["unit_cost"] == avg_unit.quantize(Decimal("0.01"))

    def test_ias2_lifo_rejected(self):
        with pytest.raises(ValueError, match="not supported"):
            compute_inventory_cost([], "LIFO")

    def test_ias2_nrv_and_write_down(self):
        nrv = compute_nrv({}, selling_price=Decimal("5000000"), costs_to_sell=Decimal("800000"))
        assert nrv.nrv == Decimal("4200000")

        val = apply_lower_of_cost_or_nrv(Decimal("3800000"), Decimal("4200000"))
        assert val == Decimal("3800000")

        val2 = apply_lower_of_cost_or_nrv(Decimal("5000000"), Decimal("4200000"))
        assert val2 == Decimal("4200000")

        wd = recognize_inventory_write_down(Decimal("5000000"), Decimal("4200000"))
        assert wd.write_down_amount == Decimal("800000")
        assert wd.debit_account == "Inventory Write-down Expense (COS)"


class TestIAS12DeferredTax:
    def test_ias12_deferred_tax(self):
        dtl = compute_deferred_tax_liability(Decimal("1500000000"), Decimal("30"))
        assert dtl.deferred_tax_amount == Decimal("450000000")

        dta = compute_deferred_tax_asset(Decimal("300000000"), Decimal("30"))
        assert dta.deferred_tax_amount == Decimal("90000000")

    def test_ias12_current_tax(self):
        current = recognize_current_tax(Decimal("2500000000"), Decimal("30"))
        assert current.current_tax == Decimal("750000000")

    def test_ias12_total_tax_expense(self):
        expense = compute_tax_expense(Decimal("750000000"), Decimal("50000000"))
        assert expense.total_tax_expense == Decimal("800000000")

    def test_ias12_dta_recognition_criteria(self):
        valid = validate_recognition(Decimal("200000000"), True)
        assert valid.can_recognize is True

        invalid = validate_recognition(Decimal("200000000"), False)
        assert invalid.can_recognize is False
        assert "not probable" in invalid.rationale.lower()

        zero = validate_recognition(Decimal("0"), False)
        assert zero.can_recognize is True
