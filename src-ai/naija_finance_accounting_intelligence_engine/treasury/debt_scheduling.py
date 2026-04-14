# Author: Quadri Atharu
"""Debt maturity scheduling and DSCR computation."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class DebtSchedulingEngine:
    """Debt maturity scheduling and Debt Service Coverage Ratio computation."""

    def compute_dscr(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Debt Service Coverage Ratio (DSCR)."""
        ebitda = float(data.get("ebitda", 0))
        principal_payments = float(data.get("principal_payments", 0))
        interest_payments = float(data.get("interest_payments", 0))
        lease_payments = float(data.get("lease_payments", 0))

        total_debt_service = round(principal_payments + interest_payments + lease_payments, 2)

        if total_debt_service <= 0:
            return {"dscr": None, "message": "No debt service obligations — DSCR not applicable"}

        dscr = round(ebitda / total_debt_service, 2)

        if dscr >= 2.0:
            health = "excellent"
            covenant_status = "well_above_covenant"
        elif dscr >= 1.5:
            health = "good"
            covenant_status = "above_typical_covenant"
        elif dscr >= 1.25:
            health = "adequate"
            covenant_status = "at_covenant_threshold"
        elif dscr >= 1.0:
            health = "tight"
            covenant_status = "below_covenant_warning"
        else:
            health = "distressed"
            covenant_status = "covenant_breach"

        return {
            "ebitda": round(ebitda, 2),
            "principal_payments": round(principal_payments, 2),
            "interest_payments": round(interest_payments, 2),
            "lease_payments": round(lease_payments, 2),
            "total_debt_service": total_debt_service,
            "dscr": dscr,
            "health_assessment": health,
            "covenant_status": covenant_status,
            "warning": "DSCR below 1.0 — insufficient cash flow to service debt" if dscr < 1.0 else None,
            "recommendation": self._dscr_recommendation(dscr),
            "computed_at": datetime.now().isoformat(),
        }

    def generate_debt_maturity_profile(self, loans: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate a debt maturity profile showing repayment timeline."""
        maturity_buckets: Dict[str, Dict[str, Any]] = {
            "0_3_months": {"total": 0.0, "loans": []},
            "3_6_months": {"total": 0.0, "loans": []},
            "6_12_months": {"total": 0.0, "loans": []},
            "1_2_years": {"total": 0.0, "loans": []},
            "2_5_years": {"total": 0.0, "loans": []},
            "5_plus_years": {"total": 0.0, "loans": []},
        }

        total_outstanding = 0.0
        total_interest_next_12m = 0.0

        for loan in loans:
            outstanding = float(loan.get("outstanding_balance", 0))
            maturity_date = loan.get("maturity_date", "")
            loan_name = loan.get("loan_name", "Unnamed")
            annual_rate = float(loan.get("annual_rate", 0))

            total_outstanding += outstanding
            total_interest_next_12m += round(outstanding * annual_rate, 2)

            try:
                mat = datetime.fromisoformat(maturity_date)
                now = datetime.now()
                days_to_maturity = (mat - now).days
            except (ValueError, TypeError):
                days_to_maturity = 9999

            if days_to_maturity <= 90:
                bucket = "0_3_months"
            elif days_to_maturity <= 180:
                bucket = "3_6_months"
            elif days_to_maturity <= 365:
                bucket = "6_12_months"
            elif days_to_maturity <= 730:
                bucket = "1_2_years"
            elif days_to_maturity <= 1825:
                bucket = "2_5_years"
            else:
                bucket = "5_plus_years"

            maturity_buckets[bucket]["total"] = round(maturity_buckets[bucket]["total"] + outstanding, 2)
            maturity_buckets[bucket]["loans"].append({"loan_name": loan_name, "outstanding": round(outstanding, 2), "maturity_date": maturity_date, "annual_rate": annual_rate})

        return {
            "total_outstanding_debt": round(total_outstanding, 2),
            "total_interest_next_12_months": round(total_interest_next_12m, 2),
            "maturity_buckets": maturity_buckets,
            "near_term_risk": maturity_buckets["0_3_months"]["total"] + maturity_buckets["3_6_months"]["total"],
            "concentration_risk": self._assess_concentration(maturity_buckets),
            "generated_at": datetime.now().isoformat(),
        }

    def compute_weighted_average_cost_of_debt(self, loans: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Compute the weighted average cost of debt."""
        total_debt = 0.0
        weighted_cost = 0.0

        for loan in loans:
            outstanding = float(loan.get("outstanding_balance", 0))
            rate = float(loan.get("annual_rate", 0))
            total_debt += outstanding
            weighted_cost += outstanding * rate

        wacd = round(weighted_cost / total_debt, 6) if total_debt > 0 else 0

        return {
            "total_debt": round(total_debt, 2),
            "weighted_annual_cost": round(weighted_cost, 2),
            "weighted_average_cost_of_debt": wacd,
            "weighted_average_cost_pct": f"{wacd * 100:.2f}%",
            "loan_details": [{"loan_name": l.get("loan_name", ""), "outstanding": round(float(l.get("outstanding_balance", 0)), 2), "rate": float(l.get("annual_rate", 0))} for l in loans],
        }

    def stress_test_debt_service(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Stress test debt service under different scenarios."""
        base_ebitda = float(data.get("ebitda", 0))
        total_debt_service = float(data.get("total_debt_service", 0))

        scenarios: List[Dict[str, Any]] = []
        for label, shock in [("mild", -0.10), ("moderate", -0.25), ("severe", -0.50), ("extreme", -0.75)]:
            stressed_ebitda = round(base_ebitda * (1 + shock), 2)
            dscr = round(stressed_ebitda / total_debt_service, 2) if total_debt_service > 0 else None
            scenarios.append({
                "scenario": label,
                "ebitda_shock": f"{shock * 100:.0f}%",
                "stressed_ebitda": stressed_ebitda,
                "dscr": dscr,
                "viable": dscr is not None and dscr >= 1.0,
            })

        return {
            "base_ebitda": round(base_ebitda, 2),
            "total_debt_service": round(total_debt_service, 2),
            "base_dscr": round(base_ebitda / total_debt_service, 2) if total_debt_service > 0 else None,
            "stress_scenarios": scenarios,
        }

    @staticmethod
    def _dscr_recommendation(dscr: float) -> str:
        """Generate recommendation based on DSCR."""
        if dscr >= 2.0:
            return "Strong debt service capacity — consider debt optimization"
        elif dscr >= 1.5:
            return "Healthy coverage — maintain current position"
        elif dscr >= 1.25:
            return "Adequate but monitor closely — prepare contingency plans"
        elif dscr >= 1.0:
            return "Tight coverage — renegotiate terms or reduce debt"
        else:
            return "CRITICAL: Insufficient cash flow to service debt — immediate action required"

    @staticmethod
    def _assess_concentration(buckets: Dict[str, Dict[str, Any]]) -> str:
        """Assess concentration risk from maturity profile."""
        near_term = buckets["0_3_months"]["total"] + buckets["3_6_months"]["total"]
        total = sum(b["total"] for b in buckets.values())
        if total <= 0:
            return "no_debt"
        ratio = near_term / total
        if ratio > 0.5:
            return "high_concentration_near_term"
        elif ratio > 0.3:
            return "moderate_concentration"
        return "well_distributed"

    def health_check(self) -> bool:
        return True
