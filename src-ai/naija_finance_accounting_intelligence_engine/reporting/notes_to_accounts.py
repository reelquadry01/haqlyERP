# Author: Quadri Atharu
"""Notes to Accounts generation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)

NOTE_TEMPLATES = [
    {"note_number": 1, "title": "Revenue", "ifrs_ref": "IFRS 15"},
    {"note_number": 2, "title": "Cost of Sales", "ifrs_ref": "IAS 2"},
    {"note_number": 3, "title": "Operating Expenses", "ifrs_ref": "IAS 1"},
    {"note_number": 4, "title": "Depreciation and Amortisation", "ifrs_ref": "IAS 16/IAS 38"},
    {"note_number": 5, "title": "Finance Costs", "ifrs_ref": "IFRS 9"},
    {"note_number": 6, "title": "Income Tax Expense", "ifrs_ref": "IAS 12"},
    {"note_number": 7, "title": "Property, Plant and Equipment", "ifrs_ref": "IAS 16"},
    {"note_number": 8, "title": "Intangible Assets", "ifrs_ref": "IAS 38"},
    {"note_number": 9, "title": "Inventories", "ifrs_ref": "IAS 2"},
    {"note_number": 10, "title": "Trade and Other Receivables", "ifrs_ref": "IFRS 9"},
    {"note_number": 11, "title": "Cash and Cash Equivalents", "ifrs_ref": "IAS 7"},
    {"note_number": 12, "title": "Trade and Other Payables", "ifrs_ref": "IAS 1"},
    {"note_number": 13, "title": "Borrowings", "ifrs_ref": "IAS 1/IFRS 16"},
    {"note_number": 14, "title": "Provisions", "ifrs_ref": "IAS 37"},
    {"note_number": 15, "title": "Share Capital and Reserves", "ifrs_ref": "IAS 1"},
    {"note_number": 16, "title": "Financial Instruments", "ifrs_ref": "IFRS 9"},
    {"note_number": 17, "title": "Related Party Transactions", "ifrs_ref": "IAS 24"},
    {"note_number": 18, "title": "Leases", "ifrs_ref": "IFRS 16"},
    {"note_number": 19, "title": "Earnings Per Share", "ifrs_ref": "IAS 33"},
    {"note_number": 20, "title": "Contingencies and Commitments", "ifrs_ref": "IAS 37"},
]


class NotesToAccountsEngine:
    """Notes to Accounts generation engine."""

    def generate_notes(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate notes to the financial statements."""
        company_id = data.get("company_id", "")
        period_end = data.get("period_end", "")
        note_data = data.get("note_data", {})

        notes: List[Dict[str, Any]] = []
        for template in NOTE_TEMPLATES:
            note_num = template["note_number"]
            content = note_data.get(str(note_num), note_data.get(template["title"].lower().replace(" ", "_"), {}))

            notes.append({
                "note_number": note_num,
                "title": template["title"],
                "ifrs_reference": template["ifrs_ref"],
                "content": content,
                "has_data": bool(content),
            })

        populated = [n for n in notes if n["has_data"]]

        return {
            "company_id": company_id,
            "period_end": period_end,
            "notes": notes,
            "total_notes": len(notes),
            "populated_notes": len(populated),
            "generated_at": datetime.now().isoformat(),
        }

    def generate_accounting_policies_note(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate the accounting policies note."""
        policies = data.get("policies", {})

        default_policies = {
            "basis_of_preparation": "The financial statements are prepared under the historical cost convention as modified by the revaluation of certain assets and in accordance with International Financial Reporting Standards (IFRS).",
            "revenue_recognition": "Revenue is recognised in accordance with IFRS 15 Revenue from Contracts with Customers.",
            "inventory_valuation": "Inventories are stated at the lower of cost and net realisable value. Cost is determined using the weighted average method.",
            "ppe_policy": "Property, plant and equipment are stated at historical cost less accumulated depreciation and impairment losses. Depreciation is provided on a straight-line basis.",
            "leasing_policy": "Leases are accounted for in accordance with IFRS 16 Leases. Right-of-use assets and lease liabilities are recognised on the statement of financial position.",
            "tax_policy": "Income tax comprises current tax and deferred tax. Deferred tax is recognised in accordance with IAS 12.",
            "fx_policy": "Foreign currency transactions are translated at the exchange rates prevailing at the dates of the transactions.",
            "impairment_policy": "Assets are tested for impairment in accordance with IAS 36 Impairment of Assets.",
        }

        for key, value in default_policies.items():
            if key not in policies:
                policies[key] = value

        return {
            "note_number": 0,
            "title": "Significant Accounting Policies",
            "policies": policies,
        }

    def health_check(self) -> bool:
        return True
