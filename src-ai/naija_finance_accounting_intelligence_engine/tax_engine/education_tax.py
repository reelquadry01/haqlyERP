# Author: Quadri Atharu
"""Nigerian Education Tax computation engine — 2% of assessable profit."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

EDU_TAX_RATE = 0.02
EDU_TACT_FUND = "Tertiary Education Trust Fund (TETFund)"


class EducationTaxEngine:
    """Nigerian Education Tax computation engine (2% of assessable profit)."""

    def compute_education_tax(self, assessable_profit: float, rate: float = EDU_TAX_RATE) -> Dict[str, Any]:
        """Compute education tax at 2% of assessable profit."""
        if assessable_profit < 0:
            raise TaxError("Assessable profit cannot be negative for education tax computation")

        tax_amount = round(assessable_profit * rate, 2)

        result: Dict[str, Any] = {
            "tax_type": "EDU_TAX",
            "assessable_profit": round(assessable_profit, 2),
            "rate": rate,
            "rate_pct": f"{rate * 100:.0f}%",
            "education_tax": tax_amount,
            "fund": EDU_TACT_FUND,
            "legal_basis": "Tertiary Education Trust Fund (Establishment, Etc.) Act 2011",
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("education_tax_computed", assessable_profit=assessable_profit, tax=tax_amount)
        return result

    def compute_education_tax_with_cit(self, assessable_profit: float, cit_rate: float = 0.30) -> Dict[str, Any]:
        """Compute education tax alongside CIT for total tax burden analysis."""
        edu_tax = self.compute_education_tax(assessable_profit)
        cit_amount = round(assessable_profit * cit_rate, 2)
        total_tax = round(cit_amount + edu_tax["education_tax"], 2)
        effective_rate = round(total_tax / assessable_profit, 4) if assessable_profit > 0 else 0

        return {
            "assessable_profit": round(assessable_profit, 2),
            "cit_at_rate": cit_rate,
            "cit_amount": cit_amount,
            "education_tax": edu_tax["education_tax"],
            "total_tax_burden": total_tax,
            "effective_combined_rate": effective_rate,
            "breakdown": {
                "cit": cit_amount,
                "education_tax": edu_tax["education_tax"],
            },
        }

    def health_check(self) -> bool:
        return True
