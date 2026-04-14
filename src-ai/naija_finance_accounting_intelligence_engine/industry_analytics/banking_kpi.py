# Author: Quadri Atharu
"""Banking KPI engine — LDR, NPL, NIM, CIR."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.logging import get_logger

logger = get_logger(__name__)


class BankingKpiEngine:
    """Banking KPI computation engine."""

    def compute_all(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute all banking KPIs."""
        total_loans = float(data.get("total_loans", 0))
        total_deposits = float(data.get("total_deposits", 0))
        non_performing_loans = float(data.get("non_performing_loans", 0))
        interest_income = float(data.get("interest_income", 0))
        interest_expense = float(data.get("interest_expense", 0))
        earning_assets = float(data.get("earning_assets", 0))
        operating_income = float(data.get("operating_income", 0))
        operating_expenses = float(data.get("operating_expenses", 0))
        total_assets = float(data.get("total_assets", 0))
        capital = float(data.get("regulatory_capital", 0))
        risk_weighted_assets = float(data.get("risk_weighted_assets", 0))

        ldr = round(total_loans / total_deposits, 4) if total_deposits > 0 else None
        npl_ratio = round(non_performing_loans / total_loans, 4) if total_loans > 0 else None
        nim = round((interest_income - interest_expense) / earning_assets, 4) if earning_assets > 0 else None
        cir = round(operating_expenses / operating_income, 4) if operating_income > 0 else None
        roa = round(operating_income / total_assets, 4) if total_assets > 0 else None
        car = round(capital / risk_weighted_assets, 4) if risk_weighted_assets > 0 else None
        provision_coverage = round(float(data.get("loan_loss_provisions", 0)) / non_performing_loans, 4) if non_performing_loans > 0 else None

        return {
            "loan_to_deposit_ratio": ldr,
            "npl_ratio": npl_ratio,
            "net_interest_margin": nim,
            "cost_to_income_ratio": cir,
            "return_on_assets": roa,
            "capital_adequacy_ratio": car,
            "provision_coverage_ratio": provision_coverage,
            "regulatory_thresholds": {
                "car_minimum": "10% (CBN)",
                "npl_maximum": "5% (CBN)",
                "ldr_maximum": "80% (CBN)",
            },
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
