# Author: Quadri Atharu
"""IFRS disclosure generation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)

DISCLOSURE_TEMPLATES = {
    "IFRS_15": {"title": "Revenue from Contracts with Customers", "sections": ["disaggregation_of_revenue", "contract_balances", "performance_obligations", "transaction_price_allocation"]},
    "IFRS_16": {"title": "Leases", "sections": ["right_of_use_assets", "lease_liabilities", "lease_expenses", "sale_and_leaseback"]},
    "IFRS_9": {"title": "Financial Instruments", "sections": ["financial_assets_classification", "financial_liabilities", "impairment_ecl", "hedge_accounting", "fair_value_hierarchy"]},
    "IAS_12": {"title": "Income Taxes", "sections": ["tax_expense_composition", "deferred_tax", "unrecognised_dtas", "tax_rate_reconciliation"]},
    "IAS_16": {"title": "Property, Plant and Equipment", "sections": ["carrying_amounts", "depreciation_methods", "revaluation", "capital_commitments"]},
    "IAS_2": {"title": "Inventories", "sections": ["carrying_amounts", "valuation_methods", "inventory_provisions", "cost_of_sales"]},
    "IAS_37": {"title": "Provisions, Contingent Liabilities and Assets", "sections": ["provisions", "contingent_liabilities", "contingent_assets"]},
    "IAS_36": {"title": "Impairment of Assets", "sections": ["impairment_losses", "cash_generating_units", "reversal_of_impairment"]},
    "IAS_24": {"title": "Related Party Disclosures", "sections": ["related_party_transactions", "key_management_compensation", "outstanding_balances"]},
    "IAS_33": {"title": "Earnings Per Share", "sections": ["basic_eps", "diluted_eps", "adjustments"]},
}


class DisclosureEngine:
    """IFRS disclosure generation engine."""

    def generate_disclosure(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate an IFRS disclosure for a specific standard."""
        standard = data.get("standard", "").upper()
        company_data = data.get("company_data", {})

        template = DISCLOSURE_TEMPLATES.get(standard)
        if not template:
            return {"error": f"No disclosure template for {standard}", "available_standards": list(DISCLOSURE_TEMPLATES.keys())}

        sections: Dict[str, Any] = {}
        for section in template["sections"]:
            sections[section] = self._generate_section(standard, section, company_data)

        return {
            "standard": standard,
            "title": template["title"],
            "sections": sections,
            "generated_at": datetime.now().isoformat(),
        }

    def generate_all_disclosures(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate all applicable IFRS disclosures."""
        applicable_standards = data.get("applicable_standards", list(DISCLOSURE_TEMPLATES.keys()))
        company_data = data.get("company_data", {})

        disclosures: Dict[str, Any] = {}
        for standard in applicable_standards:
            disclosures[standard] = self.generate_disclosure({"standard": standard, "company_data": company_data})

        return {
            "disclosures": disclosures,
            "standards_covered": len(disclosures),
            "generated_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _generate_section(standard: str, section: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a specific disclosure section."""
        section_data = data.get(section, {})
        return {
            "title": section.replace("_", " ").title(),
            "standard_reference": f"{standard} — {section.replace('_', ' ').title()}",
            "data": section_data,
            "compliance_status": "reported" if section_data else "not_applicable_or_no_data",
        }

    def health_check(self) -> bool:
        return True
