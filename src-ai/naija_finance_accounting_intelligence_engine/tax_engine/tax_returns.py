# Author: Quadri Atharu
"""Tax return form generation for Nigerian tax types."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class TaxReturnGenerator:
    """Generate Nigerian tax return forms and filing data."""

    def generate_vat_return(
        self,
        company_id: str,
        period_start: str,
        period_end: str,
        output_vat: float,
        input_vat: float,
        adjustments: float = 0,
        company_name: str = "",
        tin: str = "",
    ) -> Dict[str, Any]:
        """Generate VAT return form data (VAT Form 001)."""
        net_vat = round(output_vat - input_vat + adjustments, 2)

        return {
            "form_type": "VAT001",
            "form_name": "Value Added Tax Return",
            "company_id": company_id,
            "company_name": company_name,
            "tin": tin,
            "period_start": period_start,
            "period_end": period_end,
            "filing_deadline": self._monthly_deadline(period_end),
            "fields": {
                "box_a": {"label": "Total output VAT collected", "value": round(output_vat, 2)},
                "box_b": {"label": "Total input VAT claimed", "value": round(input_vat, 2)},
                "box_c": {"label": "Adjustments", "value": round(adjustments, 2)},
                "box_d": {"label": "Net VAT payable/(refund)", "value": net_vat},
            },
            "tax_payable": abs(net_vat) if net_vat > 0 else 0,
            "tax_refund": abs(net_vat) if net_vat < 0 else 0,
            "payment_instructions": "Pay to FIRS designated account before the 21st of the following month",
            "penalty_for_late_filing": "N50,000 first month, N25,000 each subsequent month",
            "interest_on_late_payment": "At CBN monetary policy rate + 5%",
            "status": "draft",
            "generated_at": datetime.now().isoformat(),
        }

    def generate_cit_return(
        self,
        company_id: str,
        fiscal_year: int,
        assessable_profit: float,
        cit_payable: float,
        education_tax: float,
        company_name: str = "",
        tin: str = "",
        wht_credit: float = 0,
    ) -> Dict[str, Any]:
        """Generate CIT return form data (CIT Form)."""
        net_cit = round(cit_payable - wht_credit, 2)
        total_payable = round(net_cit + education_tax, 0)

        return {
            "form_type": "CIT001",
            "form_name": "Companies Income Tax Return",
            "company_id": company_id,
            "company_name": company_name,
            "tin": tin,
            "fiscal_year": fiscal_year,
            "filing_deadline": f"{fiscal_year + 1}-06-30",
            "fields": {
                "assessable_profit": {"label": "Assessable Profit", "value": round(assessable_profit, 2)},
                "cit_payable": {"label": "CIT Payable", "value": round(cit_payable, 2)},
                "education_tax": {"label": "Education Tax (2%)", "value": round(education_tax, 2)},
                "wht_credit": {"label": "WHT Tax Credit", "value": round(wht_credit, 2)},
                "net_cit": {"label": "Net CIT After WHT Credit", "value": net_cit},
                "total_payable": {"label": "Total Tax Payable (CIT + Edu Tax)", "value": total_payable},
            },
            "tax_payable": total_payable,
            "payment_instructions": "Pay to FIRS designated account; filing due 6 months after financial year-end",
            "penalty_for_late_filing": "N25,000 first month, N12,500 each subsequent month",
            "status": "draft",
            "generated_at": datetime.now().isoformat(),
        }

    def generate_wht_return(
        self,
        company_id: str,
        period_start: str,
        period_end: str,
        wht_total: float,
        wht_line_items: List[Dict[str, Any]],
        company_name: str = "",
        tin: str = "",
    ) -> Dict[str, Any]:
        """Generate WHT return form data."""
        return {
            "form_type": "WHT001",
            "form_name": "Withholding Tax Return",
            "company_id": company_id,
            "company_name": company_name,
            "tin": tin,
            "period_start": period_start,
            "period_end": period_end,
            "fields": {
                "total_wht_deducted": {"label": "Total WHT Deducted and Remitted", "value": round(wht_total, 2)},
                "number_of_beneficiaries": {"label": "Number of Beneficiaries", "value": len(wht_line_items)},
            },
            "line_items": wht_line_items,
            "tax_payable": 0,
            "filing_note": "WHT must be remitted to FIRS by the 21st of the following month",
            "certificate_issuance": "WHT certificates must be issued to beneficiaries within the same period",
            "status": "draft",
            "generated_at": datetime.now().isoformat(),
        }

    def generate_cgt_return(
        self,
        company_id: str,
        fiscal_year: int,
        chargeable_gains: float,
        cgt_amount: float,
        company_name: str = "",
        tin: str = "",
    ) -> Dict[str, Any]:
        """Generate CGT return form data."""
        return {
            "form_type": "CGT001",
            "form_name": "Capital Gains Tax Return",
            "company_id": company_id,
            "company_name": company_name,
            "tin": tin,
            "fiscal_year": fiscal_year,
            "fields": {
                "chargeable_gains": {"label": "Total Chargeable Gains", "value": round(chargeable_gains, 2)},
                "cgt_rate": {"label": "CGT Rate", "value": "10%"},
                "cgt_payable": {"label": "CGT Payable", "value": round(cgt_amount, 2)},
            },
            "tax_payable": round(cgt_amount, 2),
            "status": "draft",
            "generated_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _monthly_deadline(period_end: str) -> str:
        """Compute the filing deadline (21st of the following month)."""
        try:
            pe = datetime.fromisoformat(period_end)
            if pe.month == 12:
                return datetime(pe.year + 1, 1, 21).isoformat()
            return datetime(pe.year, pe.month + 1, 21).isoformat()
        except (ValueError, TypeError):
            return "Check with FIRS for deadline"

    def health_check(self) -> bool:
        return True
