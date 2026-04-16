# Author: Quadri Atharu
"""Credit risk engine — ECL computation and counterparty rating."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

RATING_FACTORS = {
    "payment_history": {"weight": 0.30},
    "financial_strength": {"weight": 0.25},
    "industry_risk": {"weight": 0.20},
    "country_risk": {"weight": 0.15},
    "relationship_length": {"weight": 0.10},
}

ECL_PROVISION_RATES = {
    "stage_1": 0.01,
    "stage_2": 0.05,
    "stage_3": 0.50,
}


class CreditRiskEngine:
    """Credit risk engine with ECL and counterparty rating."""

    def compute_ecl(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute Expected Credit Loss per IFRS 9 three-stage model."""
        receivables: List[Dict[str, Any]] = data.get("receivables", [])

        stage1_total = 0.0
        stage2_total = 0.0
        stage3_total = 0.0
        details: List[Dict[str, Any]] = []

        for rec in receivables:
            amount = float(rec.get("amount", 0))
            days_overdue = int(rec.get("days_overdue", 0))
            is_defaulted = rec.get("is_defaulted", False)

            if is_defaulted or days_overdue > 180:
                stage = 3
            elif days_overdue > 30:
                stage = 2
            else:
                stage = 1

            provision_rate = float(rec.get("provision_rate", ECL_PROVISION_RATES[f"stage_{stage}"]))
            ecl = _money_round(amount * provision_rate)

            if stage == 1:
                stage1_total += amount
            elif stage == 2:
                stage2_total += amount
            else:
                stage3_total += amount

            details.append({
                "counterparty": rec.get("counterparty", ""),
                "amount": _money_round(amount),
                "days_overdue": days_overdue,
                "stage": stage,
                "provision_rate": provision_rate,
                "ecl": ecl,
            })

        stage1_ecl = _money_round(stage1_total * ECL_PROVISION_RATES["stage_1"])
        stage2_ecl = _money_round(stage2_total * ECL_PROVISION_RATES["stage_2"])
        stage3_ecl = _money_round(stage3_total * ECL_PROVISION_RATES["stage_3"])
        total_ecl = _money_round(stage1_ecl + stage2_ecl + stage3_ecl)

        return {
            "standard": "IFRS 9",
            "stage_1": {"exposure": _money_round(stage1_total), "provision_rate": ECL_PROVISION_RATES["stage_1"], "ecl": stage1_ecl, "description": "Performing — no significant increase in credit risk"},
            "stage_2": {"exposure": _money_round(stage2_total), "provision_rate": ECL_PROVISION_RATES["stage_2"], "ecl": stage2_ecl, "description": "Significant increase in credit risk (30-180 days overdue)"},
            "stage_3": {"exposure": _money_round(stage3_total), "provision_rate": ECL_PROVISION_RATES["stage_3"], "ecl": stage3_ecl, "description": "Credit-impaired (>180 days or defaulted)"},
            "total_ecl": total_ecl,
            "total_exposure": _money_round(stage1_total + stage2_total + stage3_total),
            "details": details,
            "computed_at": datetime.now().isoformat(),
        }

    def rate_counterparty(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Rate a counterparty's creditworthiness."""
        scores: Dict[str, float] = {}
        for factor, config in RATING_FACTORS.items():
            scores[factor] = Decimal(str(data.get(f"score_{factor}", 5)))

        weighted_score = _money_round(sum(scores[f] * RATING_FACTORS[f]["weight"] for f in scores))

        if weighted_score >= 8:
            rating = "AAA"
            risk_level = "minimal"
        elif weighted_score >= 7:
            rating = "AA"
            risk_level = "low"
        elif weighted_score >= 6:
            rating = "A"
            risk_level = "low_moderate"
        elif weighted_score >= 5:
            rating = "BBB"
            risk_level = "moderate"
        elif weighted_score >= 4:
            rating = "BB"
            risk_level = "high_moderate"
        elif weighted_score >= 3:
            rating = "B"
            risk_level = "high"
        else:
            rating = "CCC"
            risk_level = "very_high"

        return {
            "counterparty": data.get("name", ""),
            "scores": scores,
            "weighted_score": weighted_score,
            "rating": rating,
            "risk_level": risk_level,
            "credit_limit_recommendation": f"Limit exposure to {rating} counterparties",
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
