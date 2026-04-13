"""IFRS accounting standards implementation module.

Author: Quadri Atharu

Provides IFRS 9 (Financial Instruments), IFRS 15 (Revenue Recognition),
IFRS 16 (Leases), IAS 2 (Inventory), IAS 16 (PPE), IAS 12 (Income Taxes),
and IAS 37 (Provisions) implementations for Nigerian IFRS-compliant
financial reporting.
"""

from .ifrs9_financial_instruments import classify_financial_asset, compute_expected_credit_loss, compute_impairment, validate_classification
from .ifrs15_revenue import apply_five_step_model, identify_contract, identify_performance_obligations, determine_transaction_price, allocate_transaction_price, recognize_revenue
from .ifrs16_leases import classify_lease, compute_lease_liability, compute_rou_asset, compute_lease_depreciation, compute_lease_interest
from .ias2_inventory import compute_inventory_cost, compute_nrv, apply_lower_of_cost_or_nrv, recognize_inventory_write_down
from .ias16_ppe import classify_expenditure, compute_depreciation, compute_revaluation, compute_impairment_ppe
from .ias12_income_taxes import compute_deferred_tax_liability, compute_deferred_tax_asset, recognize_current_tax, compute_tax_expense, validate_recognition
from .ias37_provisions import recognize_provision, compute_provision_amount, classify_contingent_liability, classify_contingent_asset

__all__ = [
    "classify_financial_asset", "compute_expected_credit_loss", "compute_impairment", "validate_classification",
    "apply_five_step_model", "identify_contract", "identify_performance_obligations",
    "determine_transaction_price", "allocate_transaction_price", "recognize_revenue",
    "classify_lease", "compute_lease_liability", "compute_rou_asset", "compute_lease_depreciation", "compute_lease_interest",
    "compute_inventory_cost", "compute_nrv", "apply_lower_of_cost_or_nrv", "recognize_inventory_write_down",
    "classify_expenditure", "compute_depreciation", "compute_revaluation", "compute_impairment_ppe",
    "compute_deferred_tax_liability", "compute_deferred_tax_asset", "recognize_current_tax", "compute_tax_expense", "validate_recognition",
    "recognize_provision", "compute_provision_amount", "classify_contingent_liability", "classify_contingent_asset",
]
