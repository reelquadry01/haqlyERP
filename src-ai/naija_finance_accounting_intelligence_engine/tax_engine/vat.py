# Author: Quadri Atharu
"""Nigerian VAT computation engine — 7.5% standard rate with exempt and zero-rated items."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

VAT_STANDARD_RATE = 0.075

VAT_EXEMPT_ITEMS = {
    "medical_services", "medical_supplies", "pharmaceuticals", "educational_services",
    "educational_materials", "financial_services", "insurance_premiums", "mortgage_services",
    "rent_residential", "books", "newspapers", "magazines", "baby_products",
    "exported_goods", "exported_services", "international_transport",
    "plant_machinery_import", "raw_materials_import", "agricultural_inputs",
    "religious_services", "charitable_services",
}

VAT_ZERO_RATED_ITEMS = {
    "exported_goods", "exported_services", "international_transport",
    "international_air_tickets", "goods_in_transit",
}

REGISTRATION_THRESHOLD = 25_000_000

FILING_DAY_OF_MONTH = 21


class VatEngine:
    """Nigerian VAT computation engine implementing the VAT Act as amended."""

    def __init__(self, rate: float = VAT_STANDARD_RATE) -> None:
        self.rate = rate

    def compute_output_vat(self, taxable_amount: float, rate: Optional[float] = None, item_category: str = "") -> Dict[str, Any]:
        """Compute output VAT on sales/revenue."""
        effective_rate = rate or self.rate
        item = item_category.lower().strip()

        if item in VAT_ZERO_RATED_ITEMS:
            return self._result(taxable_amount, 0.0, 0.0, "zero_rated", item, warnings=["Item is zero-rated for VAT"])

        if item in VAT_EXEMPT_ITEMS and item not in VAT_ZERO_RATED_ITEMS:
            return self._result(taxable_amount, 0.0, 0.0, "exempt", item, warnings=["Item is exempt from VAT"])

        vat_amount = round(taxable_amount * effective_rate, 2)
        gross_amount = round(taxable_amount + vat_amount, 2)

        warnings: List[str] = []
        if taxable_amount < 0:
            raise TaxError("Taxable amount cannot be negative for VAT computation")

        return self._result(taxable_amount, vat_amount, effective_rate, "standard", item, gross_amount=gross_amount, warnings=warnings)

    def compute_input_vat(self, purchase_amount: float, rate: Optional[float] = None, item_category: str = "") -> Dict[str, Any]:
        """Compute input VAT on purchases/expenses."""
        effective_rate = rate or self.rate
        item = item_category.lower().strip()

        if item in VAT_ZERO_RATED_ITEMS:
            return self._result(purchase_amount, 0.0, 0.0, "zero_rated", item, input_vat=True, warnings=["No input VAT on zero-rated items"])

        if item in VAT_EXEMPT_ITEMS and item not in VAT_ZERO_RATED_ITEMS:
            return self._result(purchase_amount, 0.0, 0.0, "exempt", item, input_vat=True, warnings=["No input VAT on exempt items"])

        vat_amount = round(purchase_amount * effective_rate, 2)
        return self._result(purchase_amount, vat_amount, effective_rate, "standard", item, input_vat=True)

    def compute_vat_payable(self, output_vat: float, input_vat: float) -> Dict[str, Any]:
        """Compute net VAT payable (output VAT minus input VAT)."""
        if output_vat < 0 or input_vat < 0:
            raise TaxError("VAT amounts cannot be negative")

        net_vat = round(output_vat - input_vat, 2)
        is_refund = net_vat < 0

        result: Dict[str, Any] = {
            "output_vat": round(output_vat, 2),
            "input_vat": round(input_vat, 2),
            "net_vat": abs(net_vat),
            "direction": "refund" if is_refund else "payable",
            "vat_rate": self.rate,
            "computed_at": datetime.now().isoformat(),
        }

        if is_refund:
            result["refund_eligibility"] = self._check_refund_eligibility(abs(net_vat), output_vat)
            result["warnings"] = ["Net VAT is negative — you may be eligible for a VAT refund from FIRS"]

        logger.info("vat_payable_computed", output_vat=output_vat, input_vat=input_vat, net_vat=net_vat)
        return result

    def compute_vat_from_inclusive_amount(self, gross_amount: float, rate: Optional[float] = None) -> Dict[str, Any]:
        """Extract VAT and net amounts from a VAT-inclusive gross amount."""
        effective_rate = rate or self.rate
        if gross_amount < 0:
            raise TaxError("Gross amount cannot be negative")

        net_amount = round(gross_amount / (1 + effective_rate), 2)
        vat_amount = round(gross_amount - net_amount, 2)

        return {
            "gross_amount": round(gross_amount, 2),
            "net_amount": net_amount,
            "vat_amount": vat_amount,
            "vat_rate": effective_rate,
            "computed_at": datetime.now().isoformat(),
        }

    def check_registration_requirement(self, annual_turnover: float) -> Dict[str, Any]:
        """Check if a business is required to register for VAT."""
        required = annual_turnover >= REGISTRATION_THRESHOLD
        return {
            "annual_turnover": annual_turnover,
            "registration_threshold": REGISTRATION_THRESHOLD,
            "registration_required": required,
            "recommendation": "Register for VAT with FIRS immediately" if required else "VAT registration optional but may be beneficial",
            "threshold_exceeded_by": round(annual_turnover - REGISTRATION_THRESHOLD, 2) if required else 0,
        }

    def generate_vat_return_data(
        self,
        company_id: str,
        period_start: str,
        period_end: str,
        output_vat_total: float,
        input_vat_total: float,
        adjustments: float = 0,
    ) -> Dict[str, Any]:
        """Generate data for a VAT return filing."""
        net_before_adj = round(output_vat_total - input_vat_total, 2)
        net_after_adj = round(net_before_adj + adjustments, 2)

        filing_due = self._compute_filing_deadline(period_end)

        return {
            "company_id": company_id,
            "tax_type": "VAT",
            "period_start": period_start,
            "period_end": period_end,
            "box1_output_vat": round(output_vat_total, 2),
            "box2_input_vat": round(input_vat_total, 2),
            "box3_net_vat_before_adjustments": net_before_adj,
            "box4_adjustments": round(adjustments, 2),
            "box5_vat_payable": abs(net_after_adj) if net_after_adj > 0 else 0,
            "box6_vat_refund": abs(net_after_adj) if net_after_adj < 0 else 0,
            "filing_deadline": filing_due,
            "filing_day_rule": f"21st day of the month following the tax period",
            "late_filing_penalty": "N50,000 in the first instance, N25,000 for each subsequent month",
            "computed_at": datetime.now().isoformat(),
        }

    def _result(
        self,
        taxable: float,
        vat: float,
        rate: float,
        category: str,
        item: str,
        input_vat: bool = False,
        gross_amount: Optional[float] = None,
        warnings: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Build a standard VAT computation result."""
        result: Dict[str, Any] = {
            "tax_type": "VAT",
            "direction": "input" if input_vat else "output",
            "taxable_amount": round(taxable, 2),
            "vat_amount": round(vat, 2),
            "vat_rate": rate,
            "category": category,
            "item_category": item,
            "computed_at": datetime.now().isoformat(),
        }
        if gross_amount is not None:
            result["gross_amount"] = round(gross_amount, 2)
        if warnings:
            result["warnings"] = warnings
        return result

    @staticmethod
    def _check_refund_eligibility(refund_amount: float, output_vat: float) -> Dict[str, Any]:
        """Check eligibility for VAT refund (typically for exporters)."""
        return {
            "refund_amount": refund_amount,
            "eligible": True,
            "conditions": [
                "Must file monthly VAT returns",
                "Must have supporting documents for all input VAT claims",
                "Refund claim must be made within 5 years",
                "FIRS may verify before approval",
            ],
        }

    @staticmethod
    def _compute_filing_deadline(period_end: str) -> str:
        """Compute the filing deadline (21st of the following month)."""
        try:
            pe = datetime.fromisoformat(period_end)
            if pe.month == 12:
                deadline = datetime(pe.year + 1, 1, FILING_DAY_OF_MONTH)
            else:
                deadline = datetime(pe.year, pe.month + 1, FILING_DAY_OF_MONTH)
            return deadline.isoformat()
        except (ValueError, TypeError):
            return "Unable to compute deadline — invalid period_end format"

    def health_check(self) -> bool:
        return True
