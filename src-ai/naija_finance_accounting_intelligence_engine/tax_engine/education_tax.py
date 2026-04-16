# Author: Quadri Atharu
"""Nigerian Education Tax computation engine — 1% of assessable profit.

Updated per Nigeria Tax Reform Acts 2025 (effective 2026):
- Rate reduced from 2% to 1%
- NDDC levy merged into Education Tax
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict

from ..core.exceptions import TaxError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

EDU_TAX_RATE = 0.01
EDU_TACT_FUND = "Tertiary Education Trust Fund (TETFund) + NDDC Levy (merged per Tax Reform 2025)"


class EducationTaxEngine:
    """Nigerian Education Tax computation engine (1% of assessable profit)."""

    def compute_education_tax(self, assessable_profit: float, rate: float = EDU_TAX_RATE) -> Dict[str, Any]:
        """Compute education tax at 1% of assessable profit."""
        if assessable_profit < 0:
            raise TaxError("Assessable profit cannot be negative for education tax computation")

        tax_amount = _money_round(assessable_profit * rate)

        result: Dict[str, Any] = {
            "tax_type": "EDU_TAX",
            "assessable_profit": _money_round(assessable_profit),
            "rate": rate,
            "rate_pct": f"{rate * 100:.0f}%",
            "education_tax": tax_amount,
            "fund": EDU_TACT_FUND,
            "legal_basis": "Tertiary Education Trust Fund Act 2011; Nigeria Tax Reform Acts 2025 (NDDC merged)",
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("education_tax_computed", assessable_profit=assessable_profit, tax=tax_amount)
        return result

    def compute_education_tax_with_cit(self, assessable_profit: float, cit_rate: float = 0.25) -> Dict[str, Any]:
        """Compute education tax alongside CIT for total tax burden analysis."""
        edu_tax = self.compute_education_tax(assessable_profit)
        cit_amount = _money_round(assessable_profit * cit_rate)
        total_tax = _money_round(cit_amount + edu_tax["education_tax"])
        effective_rate = round(total_tax / assessable_profit, 4) if assessable_profit > 0 else 0

        return {
            "assessable_profit": _money_round(assessable_profit),
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
