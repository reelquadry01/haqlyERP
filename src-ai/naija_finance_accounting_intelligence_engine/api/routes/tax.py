# Author: Quadri Atharu
"""Tax routes — VAT, WHT, CIT, composite schedules, and risk flags."""

from __future__ import annotations

from typing import Any, Dict, Optional

from fastapi import APIRouter, HTTPException, status

from ...tax_engine.vat import VatEngine
from ...tax_engine.wht import WhtEngine
from ...tax_engine.cit import CitEngine
from ...tax_engine.education_tax import EducationTaxEngine
from ...tax_engine.capital_gains_tax import CapitalGainsTaxEngine
from ...tax_engine.tax_schedules import TaxScheduleGenerator
from ...tax_engine.tax_risk_flags import TaxRiskDetector

router = APIRouter(prefix="/tax", tags=["Tax"])

_vat = VatEngine()
_wht = WhtEngine()
_cit = CitEngine()
_edu = EducationTaxEngine()
_cgt = CapitalGainsTaxEngine()
_schedules = TaxScheduleGenerator()
_risk = TaxRiskDetector()


@router.post("/compute-vat")
async def compute_vat(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute output VAT on a taxable amount."""
    taxable_amount = body.get("taxable_amount")
    if taxable_amount is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="taxable_amount is required")
    try:
        result = _vat.compute_output_vat(
            taxable_amount=float(taxable_amount),
            rate=body.get("rate"),
            item_category=body.get("item_category", ""),
        )
        return {"status": "success", "vat": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/compute-wht")
async def compute_wht(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute Withholding Tax on a payment."""
    amount = body.get("amount")
    category = body.get("category")
    if amount is None or not category:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="amount and category are required")
    try:
        result = _wht.compute_wht(payment_amount=float(amount), category=category)
        return {"status": "success", "wht": result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/compute-cit")
async def compute_cit(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute Companies Income Tax."""
    profit = body.get("profit")
    turnover = body.get("turnover")
    industry = body.get("industry", "general")
    if profit is None or turnover is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="profit and turnover are required")
    try:
        cit_result = _cit.compute_cit(
            profit_before_tax=float(profit),
            turnover=float(turnover),
            industry=industry,
            is_manufacturing=body.get("is_manufacturing", False),
            is_small_company=body.get("is_small_company"),
            capital_allowances=body.get("capital_allowances"),
            non_deductible_expenses=float(body.get("non_deductible_expenses", 0)),
            exempt_income=float(body.get("exempt_income", 0)),
        )
        edu_result = _edu.compute_education_tax(cit_result["assessable_profit"])
        return {"status": "success", "cit": cit_result, "education_tax": edu_result}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/compute-all")
async def compute_all_taxes(body: Dict[str, Any]) -> Dict[str, Any]:
    """Compute all tax types for a company in a given period."""
    company_id = body.get("company_id", "")
    period = body.get("period", "")
    if not company_id or not period:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="company_id and period are required")
    try:
        schedule = _schedules.generate_full_schedule(
            company_id=company_id,
            period_start=period,
            period_end=period,
            vat_data=body.get("vat_data"),
            wht_data=body.get("wht_data"),
            cit_data=body.get("cit_data"),
            cgt_data=body.get("cgt_data"),
            stamp_data=body.get("stamp_data"),
        )
        return {"status": "success", "schedule": schedule}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.get("/schedule")
async def get_tax_schedule(company_id: str, period: str) -> Dict[str, Any]:
    """Generate a composite tax schedule for a company and period."""
    try:
        schedule = _schedules.generate_full_schedule(
            company_id=company_id,
            period_start=period,
            period_end=period,
        )
        return {"status": "success", "schedule": schedule}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.get("/risk-flags")
async def get_tax_risk_flags(company_id: str) -> Dict[str, Any]:
    """Detect tax risk flags for a company."""
    try:
        flags = _risk.detect_risks({"company_id": company_id})
        return {"status": "success", "risk_flags": flags}
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))
