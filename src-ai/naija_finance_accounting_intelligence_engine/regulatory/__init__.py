"""Regulatory compliance module for Nigerian financial regulations.

Author: Quadri Atharu

Provides FIRS compliance rules, CAMA company law requirements,
Nigerian tax law reference, CBN policy integration, NBS statistical
alignment, and dynamic regulatory update management.
"""

from .firs_rules import validate_tax_compliance, get_applicable_taxes, compute_tax_obligations, check_filing_status
from .cama import check_company_registration_requirements, validate_annual_return_filing, check_director_requirements
from .nigerian_tax_laws import TaxLawReference
from .cbn_policies import get_current_mpr, get_cash_reserve_ratio, get_treasury_bill_rate, check_fx_regulations, get_form_a_requirements
from .nbs_structures import map_to_nbs_classification, get_economic_indicators, format_statistical_return
from .dynamic_updates import RegulatoryUpdateManager

__all__ = [
    "validate_tax_compliance", "get_applicable_taxes", "compute_tax_obligations", "check_filing_status",
    "check_company_registration_requirements", "validate_annual_return_filing", "check_director_requirements",
    "TaxLawReference",
    "get_current_mpr", "get_cash_reserve_ratio", "get_treasury_bill_rate", "check_fx_regulations", "get_form_a_requirements",
    "map_to_nbs_classification", "get_economic_indicators", "format_statistical_return",
    "RegulatoryUpdateManager",
]
