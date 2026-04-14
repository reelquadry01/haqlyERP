# Author: Quadri Atharu
"""Composite tax schedule generation for Nigerian entities."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger
from .vat import VatEngine
from .wht import WhtEngine
from .cit import CitEngine
from .education_tax import EducationTaxEngine
from .capital_gains_tax import CapitalGainsTaxEngine
from .stamp_duties import StampDutyEngine

logger = get_logger(__name__)


class TaxScheduleGenerator:
    """Generate composite tax schedules combining all Nigerian tax types."""

    def __init__(self) -> None:
        self.vat = VatEngine()
        self.wht = WhtEngine()
        self.cit = CitEngine()
        self.edu = EducationTaxEngine()
        self.cgt = CapitalGainsTaxEngine()
        self.stamp = StampDutyEngine()

    def generate_full_schedule(
        self,
        company_id: str,
        period_start: str,
        period_end: str,
        vat_data: Dict[str, Any] | None = None,
        wht_data: Dict[str, Any] | None = None,
        cit_data: Dict[str, Any] | None = None,
        cgt_data: Dict[str, Any] | None = None,
        stamp_data: Dict[str, Any] | None = None,
    ) -> Dict[str, Any]:
        """Generate a comprehensive tax schedule for a period."""
        schedule: Dict[str, Any] = {
            "company_id": company_id,
            "period_start": period_start,
            "period_end": period_end,
            "schedule_type": "composite_tax_schedule",
            "taxes": {},
            "total_tax_liability": 0.0,
            "generated_at": datetime.now().isoformat(),
        }

        total = 0.0

        if vat_data:
            vat_result = self.vat.compute_vat_payable(
                output_vat=float(vat_data.get("output_vat", 0)),
                input_vat=float(vat_data.get("input_vat", 0)),
            )
            schedule["taxes"]["VAT"] = vat_result
            total += vat_result.get("net_vat", 0) if vat_result.get("direction") == "payable" else 0

        if wht_data:
            wht_result = self.wht.compute_batch_wht(wht_data.get("payments", []))
            schedule["taxes"]["WHT"] = wht_result
            total += wht_result.get("total_wht_deducted", 0)

        if cit_data:
            cit_result = self.cit.compute_cit(
                profit_before_tax=float(cit_data.get("profit_before_tax", 0)),
                turnover=float(cit_data.get("turnover", 0)),
                industry=cit_data.get("industry", "general"),
                capital_allowances=cit_data.get("capital_allowances"),
                non_deductible_expenses=float(cit_data.get("non_deductible_expenses", 0)),
            )
            schedule["taxes"]["CIT"] = cit_result
            total += cit_result.get("cit_payable", 0)

            edu_result = self.edu.compute_education_tax(cit_result.get("assessable_profit", 0))
            schedule["taxes"]["EDU_TAX"] = edu_result
            total += edu_result.get("education_tax", 0)

        if cgt_data:
            cgt_result = self.cgt.compute_batch_cgt(cgt_data.get("disposals", []))
            schedule["taxes"]["CGT"] = cgt_result
            total += cgt_result.get("total_cgt", 0)

        if stamp_data:
            stamp_result = self.stamp.compute_batch_stamp_duty(stamp_data.get("documents", []))
            schedule["taxes"]["STAMP_DUTY"] = stamp_result
            total += stamp_result.get("total_stamp_duty", 0)

        schedule["total_tax_liability"] = round(total, 2)
        schedule["tax_type_count"] = len(schedule["taxes"])

        logger.info("full_tax_schedule_generated", company_id=company_id, total=total, types=len(schedule["taxes"]))
        return schedule

    def generate_vat_schedule(self, company_id: str, period_start: str, period_end: str, transactions: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate a VAT-only schedule from transactions."""
        output_vat_total = 0.0
        input_vat_total = 0.0
        line_items: List[Dict[str, Any]] = []

        for txn in transactions:
            amount = float(txn.get("amount", 0))
            category = txn.get("item_category", "")
            is_sale = txn.get("is_sale", True)

            if is_sale:
                vat_result = self.vat.compute_output_vat(amount, item_category=category)
                output_vat_total += vat_result.get("vat_amount", 0)
            else:
                vat_result = self.vat.compute_input_vat(amount, item_category=category)
                input_vat_total += vat_result.get("vat_amount", 0)

            line_items.append(vat_result)

        net_result = self.vat.compute_vat_payable(output_vat_total, input_vat_total)

        return {
            "company_id": company_id,
            "period_start": period_start,
            "period_end": period_end,
            "schedule_type": "vat_schedule",
            "line_items": line_items,
            "output_vat_total": round(output_vat_total, 2),
            "input_vat_total": round(input_vat_total, 2),
            "net_vat_result": net_result,
            "generated_at": datetime.now().isoformat(),
        }

    def generate_cit_schedule(self, company_id: str, fiscal_year: int, cit_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a CIT schedule with education tax."""
        cit_result = self.cit.compute_cit(
            profit_before_tax=float(cit_data.get("profit_before_tax", 0)),
            turnover=float(cit_data.get("turnover", 0)),
            industry=cit_data.get("industry", "general"),
            is_manufacturing=cit_data.get("is_manufacturing", False),
            capital_allowances=cit_data.get("capital_allowances"),
        )

        edu_result = self.edu.compute_education_tax(cit_result.get("assessable_profit", 0))

        return {
            "company_id": company_id,
            "fiscal_year": fiscal_year,
            "schedule_type": "cit_schedule",
            "cit": cit_result,
            "education_tax": edu_result,
            "total_income_tax_burden": round(cit_result.get("cit_payable", 0) + edu_result.get("education_tax", 0), 2),
            "generated_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
