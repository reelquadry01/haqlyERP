# Author: Quadri Atharu
"""Tax Agent — computes all taxes, generates schedules, detects risks, and prepares returns."""

from __future__ import annotations

from typing import Any, Dict

from ..tax_engine import (
    CapitalGainsTaxEngine,
    CitEngine,
    EducationTaxEngine,
    StampDutyEngine,
    TaxReturnGenerator,
    TaxRiskDetector,
    TaxScheduleGenerator,
    VatEngine,
    WhtEngine,
)
from .base_agent import BaseAgent


class TaxAgent(BaseAgent):
    """Agent responsible for Nigerian tax computation and compliance."""

    agent_name = "tax_agent"

    def __init__(self) -> None:
        super().__init__()
        self._vat = VatEngine()
        self._wht = WhtEngine()
        self._cit = CitEngine()
        self._edu = EducationTaxEngine()
        self._cgt = CapitalGainsTaxEngine()
        self._stamp = StampDutyEngine()
        self._schedules = TaxScheduleGenerator()
        self._risk_detector = TaxRiskDetector()
        self._returns = TaxReturnGenerator()
        self.register_skill("compute_all_taxes", self._compute_all_taxes)
        self.register_skill("generate_schedules", self._generate_schedules)
        self.register_skill("detect_risks", self._detect_risks)
        self.register_skill("prepare_returns", self._prepare_returns)

    def _compute_all_taxes(self, data: Dict[str, Any]) -> Dict[str, Any]:
        results: Dict[str, Any] = {}
        total_liability = 0.0

        vat_data = data.get("vat")
        if vat_data:
            output_vat = float(vat_data.get("output_vat", 0))
            input_vat = float(vat_data.get("input_vat", 0))
            vat_result = self._vat.compute_vat_payable(output_vat, input_vat)
            results["vat"] = vat_result
            total_liability += vat_result.get("net_vat", 0) if vat_result.get("direction") == "payable" else 0

        wht_data = data.get("wht")
        if wht_data:
            payments = wht_data.get("payments", [])
            wht_result = self._wht.compute_batch_wht(payments)
            results["wht"] = wht_result
            total_liability += wht_result.get("total_wht_deducted", 0)

        cit_data = data.get("cit")
        if cit_data:
            cit_result = self._cit.compute_cit(
                profit_before_tax=float(cit_data.get("profit_before_tax", 0)),
                turnover=float(cit_data.get("turnover", 0)),
                industry=cit_data.get("industry", "general"),
                capital_allowances=cit_data.get("capital_allowances"),
                non_deductible_expenses=float(cit_data.get("non_deductible_expenses", 0)),
            )
            results["cit"] = cit_result
            total_liability += cit_result.get("cit_payable", 0)

            edu_result = self._edu.compute_education_tax(cit_result.get("assessable_profit", 0))
            results["education_tax"] = edu_result
            total_liability += edu_result.get("education_tax", 0)

        cgt_data = data.get("cgt")
        if cgt_data:
            cgt_result = self._cgt.compute_batch_cgt(cgt_data.get("disposals", []))
            results["cgt"] = cgt_result
            total_liability += cgt_result.get("total_cgt", 0)

        stamp_data = data.get("stamp_duty")
        if stamp_data:
            stamp_result = self._stamp.compute_batch_stamp_duty(stamp_data.get("documents", []))
            results["stamp_duty"] = stamp_result
            total_liability += stamp_result.get("total_stamp_duty", 0)

        return {
            "success": True,
            "taxes": results,
            "total_tax_liability": round(total_liability, 2),
            "tax_types_computed": list(results.keys()),
        }

    def _generate_schedules(self, data: Dict[str, Any]) -> Dict[str, Any]:
        schedule_type = data.get("schedule_type", "full")
        company_id = data.get("company_id", "")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")

        if schedule_type == "vat":
            transactions = data.get("transactions", [])
            result = self._schedules.generate_vat_schedule(company_id, period_start, period_end, transactions)
        elif schedule_type == "cit":
            fiscal_year = int(data.get("fiscal_year", 2024))
            cit_data = data.get("cit_data", {})
            result = self._schedules.generate_cit_schedule(company_id, fiscal_year, cit_data)
        else:
            result = self._schedules.generate_full_schedule(
                company_id=company_id,
                period_start=period_start,
                period_end=period_end,
                vat_data=data.get("vat_data"),
                wht_data=data.get("wht_data"),
                cit_data=data.get("cit_data"),
                cgt_data=data.get("cgt_data"),
                stamp_data=data.get("stamp_data"),
            )

        return {
            "success": True,
            "schedule_type": schedule_type,
            "schedule": result,
        }

    def _detect_risks(self, data: Dict[str, Any]) -> Dict[str, Any]:
        tax_data = data.get("tax_data", {})
        result = self._risk_detector.detect_risks(tax_data)
        return {
            "success": True,
            "risk_assessment": result,
            "overall_risk_level": result.get("overall_risk_level", "unknown"),
            "total_flags": result.get("total_flags", 0),
        }

    def _prepare_returns(self, data: Dict[str, Any]) -> Dict[str, Any]:
        return_type = data.get("return_type", "vat").lower()
        company_id = data.get("company_id", "")
        company_name = data.get("company_name", "")
        tin = data.get("tin", "")

        if return_type == "vat":
            period_start = data.get("period_start", "")
            period_end = data.get("period_end", "")
            output_vat = float(data.get("output_vat", 0))
            input_vat = float(data.get("input_vat", 0))
            adjustments = float(data.get("adjustments", 0))
            result = self._returns.generate_vat_return(
                company_id=company_id,
                period_start=period_start,
                period_end=period_end,
                output_vat=output_vat,
                input_vat=input_vat,
                adjustments=adjustments,
                company_name=company_name,
                tin=tin,
            )
        elif return_type == "cit":
            fiscal_year = int(data.get("fiscal_year", 2024))
            assessable_profit = float(data.get("assessable_profit", 0))
            cit_payable = float(data.get("cit_payable", 0))
            result = self._returns.generate_cit_return(
                company_id=company_id,
                fiscal_year=fiscal_year,
                assessable_profit=assessable_profit,
                cit_payable=cit_payable,
                company_name=company_name,
                tin=tin,
            )
        elif return_type == "wht":
            period_start = data.get("period_start", "")
            period_end = data.get("period_end", "")
            wht_deductions = data.get("wht_deductions", [])
            result = self._returns.generate_wht_return(
                company_id=company_id,
                period_start=period_start,
                period_end=period_end,
                wht_deductions=wht_deductions,
                company_name=company_name,
                tin=tin,
            )
        else:
            return {"success": False, "error": f"Unsupported return type: {return_type}"}

        return {
            "success": True,
            "return_type": return_type,
            "tax_return": result,
        }
