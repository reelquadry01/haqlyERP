# Author: Quadri Atharu
"""Nigerian VAT computation engine — 7.5% standard rate with exempt and zero-rated items.

Updated per Nigeria Tax Reform Acts 2025 (effective 2026):
- VAT rate remains 7.5%
- Registration threshold raised to NGN 50M (was NGN 25M)
- Small business exemption threshold raised to NGN 50M
"""

from __future__ import annotations

from datetime import datetime
from decimal import Decimal, ROUND_HALF_UP
from typing import Any, Dict, List, Optional

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

VAT_STANDARD_RATE = Decimal("0.075")

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

REGISTRATION_THRESHOLD = Decimal("50000000")

FILING_DAY_OF_MONTH = 21

TWO_PLACES = Decimal("0.01")


def _to_decimal(val: Any) -> Decimal:
    if isinstance(val, Decimal):
        return val
    return Decimal(str(val))


def _money_round(val: Decimal) -> Decimal:
    return val.quantize(TWO_PLACES, rounding=ROUND_HALF_UP)


class VatEngine:
    """Nigerian VAT computation engine implementing the VAT Act as amended."""

    def __init__(self, rate: Optional[Decimal] = None) -> None:
        self.rate = rate if rate is not None else VAT_STANDARD_RATE

    def compute_output_vat(self, taxable_amount: Any, rate: Optional[Any] = None, item_category: str = "") -> Dict[str, Any]:
        """Compute output VAT on sales/revenue."""
        ta = _to_decimal(taxable_amount)
        effective_rate = _to_decimal(rate) if rate is not None else self.rate
        item = item_category.lower().strip()

        if item in VAT_ZERO_RATED_ITEMS:
            return self._result(ta, Decimal("0"), Decimal("0"), "zero_rated", item, warnings=["Item is zero-rated for VAT"])

        if item in VAT_EXEMPT_ITEMS and item not in VAT_ZERO_RATED_ITEMS:
            return self._result(ta, Decimal("0"), Decimal("0"), "exempt", item, warnings=["Item is exempt from VAT"])

        vat_amount = _money_round(ta * effective_rate)
        gross_amount = _money_round(ta + vat_amount)

        warnings: List[str] = []
        if ta < 0:
            raise TaxError("Taxable amount cannot be negative for VAT computation")

        return self._result(ta, vat_amount, effective_rate, "standard", item, gross_amount=gross_amount, warnings=warnings)

    def compute_input_vat(self, purchase_amount: Any, rate: Optional[Any] = None, item_category: str = "") -> Dict[str, Any]:
        """Compute input VAT on purchases/expenses."""
        pa = _to_decimal(purchase_amount)
        effective_rate = _to_decimal(rate) if rate is not None else self.rate
        item = item_category.lower().strip()

        if item in VAT_ZERO_RATED_ITEMS:
            return self._result(pa, Decimal("0"), Decimal("0"), "zero_rated", item, input_vat=True, warnings=["No input VAT on zero-rated items"])

        if item in VAT_EXEMPT_ITEMS and item not in VAT_ZERO_RATED_ITEMS:
            return self._result(pa, Decimal("0"), Decimal("0"), "exempt", item, input_vat=True, warnings=["No input VAT on exempt items"])

        vat_amount = _money_round(pa * effective_rate)
        return self._result(pa, vat_amount, effective_rate, "standard", item, input_vat=True)

    def compute_vat_payable(self, output_vat: Any, input_vat: Any) -> Dict[str, Any]:
        """Compute net VAT payable (output VAT minus input VAT)."""
        ov = _to_decimal(output_vat)
        iv = _to_decimal(input_vat)
        if ov < 0 or iv < 0:
            raise TaxError("VAT amounts cannot be negative")

        net_vat = _money_round(ov - iv)
        is_refund = net_vat < 0

        result: Dict[str, Any] = {
            "output_vat": _money_round(ov),
            "input_vat": _money_round(iv),
            "net_vat": abs(net_vat),
            "direction": "refund" if is_refund else "payable",
            "vat_rate": self.rate,
            "computed_at": datetime.now().isoformat(),
        }

        if is_refund:
            result["refund_eligibility"] = self._check_refund_eligibility(abs(net_vat), ov)
            result["warnings"] = ["Net VAT is negative — you may be eligible for a VAT refund from FIRS"]

        logger.info("vat_payable_computed", output_vat=str(ov), input_vat=str(iv), net_vat=str(net_vat))
        return result

    def compute_vat_from_inclusive_amount(self, gross_amount: Any, rate: Optional[Any] = None) -> Dict[str, Any]:
        """Extract VAT and net amounts from a VAT-inclusive gross amount."""
        ga = _to_decimal(gross_amount)
        effective_rate = _to_decimal(rate) if rate is not None else self.rate
        if ga < 0:
            raise TaxError("Gross amount cannot be negative")

        net_amount = _money_round(ga / (Decimal("1") + effective_rate))
        vat_amount = _money_round(ga - net_amount)

        return {
            "gross_amount": _money_round(ga),
            "net_amount": net_amount,
            "vat_amount": vat_amount,
            "vat_rate": effective_rate,
            "computed_at": datetime.now().isoformat(),
        }

    def check_registration_requirement(self, annual_turnover: Any) -> Dict[str, Any]:
        """Check if a business is required to register for VAT."""
        at = _to_decimal(annual_turnover)
        required = at >= REGISTRATION_THRESHOLD
        return {
            "annual_turnover": str(at),
            "registration_threshold": str(REGISTRATION_THRESHOLD),
            "registration_required": required,
            "recommendation": "Register for VAT with FIRS immediately" if required else "VAT registration optional but may be beneficial",
            "threshold_exceeded_by": str(_money_round(at - REGISTRATION_THRESHOLD)) if required else "0",
        }

    def generate_vat_return_data(
        self,
        company_id: str,
        period_start: str,
        period_end: str,
        output_vat_total: Any,
        input_vat_total: Any,
        adjustments: Any = 0,
    ) -> Dict[str, Any]:
        """Generate data for a VAT return filing."""
        ovt = _to_decimal(output_vat_total)
        ivt = _to_decimal(input_vat_total)
        adj = _to_decimal(adjustments)
        net_before_adj = _money_round(ovt - ivt)
        net_after_adj = _money_round(net_before_adj + adj)

        filing_due = self._compute_filing_deadline(period_end)

        return {
            "company_id": company_id,
            "tax_type": "VAT",
            "period_start": period_start,
            "period_end": period_end,
            "box1_output_vat": str(_money_round(ovt)),
            "box2_input_vat": str(_money_round(ivt)),
            "box3_net_vat_before_adjustments": str(net_before_adj),
            "box4_adjustments": str(_money_round(adj)),
            "box5_vat_payable": str(abs(net_after_adj)) if net_after_adj > 0 else "0",
            "box6_vat_refund": str(abs(net_after_adj)) if net_after_adj < 0 else "0",
            "filing_deadline": filing_due,
            "filing_day_rule": f"21st day of the month following the tax period",
            "late_filing_penalty": "N50,000 in the first instance, N25,000 for each subsequent month",
            "computed_at": datetime.now().isoformat(),
        }

    def _result(
        self,
        taxable: Decimal,
        vat: Decimal,
        rate: Decimal,
        category: str,
        item: str,
        input_vat: bool = False,
        gross_amount: Optional[Decimal] = None,
        warnings: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Build a standard VAT computation result."""
        result: Dict[str, Any] = {
            "tax_type": "VAT",
            "direction": "input" if input_vat else "output",
            "taxable_amount": str(_money_round(taxable)),
            "vat_amount": str(_money_round(vat)),
            "vat_rate": str(rate),
            "category": category,
            "item_category": item,
            "computed_at": datetime.now().isoformat(),
        }
        if gross_amount is not None:
            result["gross_amount"] = str(_money_round(gross_amount))
        if warnings:
            result["warnings"] = warnings
        return result

    @staticmethod
    def _check_refund_eligibility(refund_amount: Decimal, output_vat: Decimal) -> Dict[str, Any]:
        """Check eligibility for VAT refund (typically for exporters)."""
        return {
            "refund_amount": str(refund_amount),
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
