# Author: Quadri Atharu
"""Public sector exposure and government payment risk assessment for Nigerian businesses."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)

GOV_PAYMENT_DELAY_TIERS: Dict[str, Dict[str, Any]] = {
    "federal_ministry": {"avg_delay_days": 90, "reliability_score": 0.70},
    "state_government": {"avg_delay_days": 120, "reliability_score": 0.55},
    "local_government": {"avg_delay_days": 180, "reliability_score": 0.40},
    "government_agency": {"avg_delay_days": 60, "reliability_score": 0.75},
    "mda": {"avg_delay_days": 75, "reliability_score": 0.65},
    "parastatal": {"avg_delay_days": 90, "reliability_score": 0.60},
    "educational_institution": {"avg_delay_days": 45, "reliability_score": 0.80},
    "military_security": {"avg_delay_days": 30, "reliability_score": 0.85},
}

EXPOSURE_THRESHOLDS: Dict[str, float] = {
    "low": 0.15,
    "medium": 0.30,
    "high": 0.50,
    "critical": 0.70,
}


class PublicSectorEngine:
    """Assess public sector revenue exposure and government payment risk."""

    def assess_public_sector_exposure(
        self,
        revenue_from_gov: float,
        total_revenue: float,
    ) -> float:
        if total_revenue <= 0:
            logger.warning("public_sector_exposure_zero_revenue", revenue_from_gov=revenue_from_gov)
            return 0.0
        ratio = round(revenue_from_gov / total_revenue, 4)
        logger.info("public_sector_exposure_computed", ratio=ratio)
        return ratio

    def compute_gov_payment_risk(
        self,
        outstanding_amount: float,
        avg_delay_days: float,
        gov_entity_type: str = "federal_ministry",
        total_annual_revenue: float = 0,
        number_of_invoices: int = 1,
    ) -> Dict[str, Any]:
        delay_tier = GOV_PAYMENT_DELAY_TIERS.get(gov_entity_type.lower().strip(), {"avg_delay_days": 90, "reliability_score": 0.60})
        tier_avg_delay = delay_tier["avg_delay_days"]
        tier_reliability = delay_tier["reliability_score"]

        effective_delay = max(avg_delay_days, tier_avg_delay)

        risk_score = _compute_risk_score(effective_delay, outstanding_amount, tier_reliability, total_annual_revenue)
        risk_score = max(1, min(10, round(risk_score, 1)))

        risk_level = "low"
        if risk_score >= 8:
            risk_level = "critical"
        elif risk_score >= 6:
            risk_level = "high"
        elif risk_score >= 4:
            risk_level = "medium"

        recommendation = _generate_risk_recommendation(risk_level, effective_delay, outstanding_amount, total_annual_revenue)

        financing_cost = round(outstanding_amount * 0.25 * (effective_delay / 365), 2) if outstanding_amount > 0 else 0

        result: Dict[str, Any] = {
            "outstanding_amount": round(outstanding_amount, 2),
            "avg_delay_days": effective_delay,
            "reported_delay_days": avg_delay_days,
            "tier_average_delay": tier_avg_delay,
            "gov_entity_type": gov_entity_type,
            "tier_reliability_score": tier_reliability,
            "risk_score": risk_score,
            "risk_level": risk_level,
            "estimated_financing_cost": financing_cost,
            "number_of_invoices": number_of_invoices,
            "recommendation": recommendation,
            "computed_at": datetime.now().isoformat(),
        }

        if total_annual_revenue > 0:
            result["exposure_pct"] = round(outstanding_amount / total_annual_revenue, 4)

        logger.info("gov_payment_risk_computed", risk_score=risk_score, risk_level=risk_level, entity_type=gov_entity_type)
        return result

    def assess_portfolio_risk(
        self,
        gov_receivables: List[Dict[str, Any]],
        total_revenue: float,
    ) -> Dict[str, Any]:
        if not gov_receivables:
            return {
                "total_outstanding": 0.0,
                "total_revenue": total_revenue,
                "portfolio_risk_score": 0.0,
                "portfolio_risk_level": "none",
                "entity_breakdown": [],
                "recommendations": [],
                "computed_at": datetime.now().isoformat(),
            }

        total_outstanding = 0.0
        entity_breakdown: List[Dict[str, Any]] = []
        weighted_risk_sum = 0.0

        for receivable in gov_receivables:
            amount = float(receivable.get("outstanding_amount", 0))
            delay = float(receivable.get("avg_delay_days", 60))
            entity_type = receivable.get("gov_entity_type", "federal_ministry")
            risk = self.compute_gov_payment_risk(amount, delay, entity_type, total_revenue)

            total_outstanding += amount
            weighted_risk_sum += amount * risk["risk_score"]
            entity_breakdown.append(risk)

        portfolio_risk_score = round(weighted_risk_sum / total_outstanding, 1) if total_outstanding > 0 else 0
        portfolio_risk_level = "low"
        if portfolio_risk_score >= 8:
            portfolio_risk_level = "critical"
        elif portfolio_risk_score >= 6:
            portfolio_risk_level = "high"
        elif portfolio_risk_score >= 4:
            portfolio_risk_level = "medium"

        exposure_ratio = round(total_outstanding / total_revenue, 4) if total_revenue > 0 else 0

        recommendations: List[str] = []
        if exposure_ratio > 0.30:
            recommendations.append("High public sector concentration — diversify revenue sources urgently")
        if portfolio_risk_level in ("high", "critical"):
            recommendations.append("Consider factoring or discounting government receivables")
            recommendations.append("Require advance payments or milestone-based billing for government contracts")
        if any(r["avg_delay_days"] > 120 for r in entity_breakdown):
            recommendations.append("Extended payment delays detected — escalate through formal channels (BPP, Ministry of Finance)")
        if not recommendations:
            recommendations.append("Public sector receivables risk within acceptable range")

        return {
            "total_outstanding": round(total_outstanding, 2),
            "total_revenue": total_revenue,
            "exposure_ratio": exposure_ratio,
            "portfolio_risk_score": portfolio_risk_score,
            "portfolio_risk_level": portfolio_risk_level,
            "entity_count": len(gov_receivables),
            "entity_breakdown": entity_breakdown,
            "recommendations": recommendations,
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True


def _compute_risk_score(delay_days: float, outstanding: float, reliability: float, annual_revenue: float) -> float:
    delay_factor = min(delay_days / 30, 5)
    reliability_factor = (1 - reliability) * 5
    concentration_factor = 0
    if annual_revenue > 0:
        concentration = outstanding / annual_revenue
        concentration_factor = min(concentration * 3, 5)
    base_score = (delay_factor * 0.4) + (reliability_factor * 0.35) + (concentration_factor * 0.25)
    return base_score * 2


def _generate_risk_recommendation(risk_level: str, delay_days: float, outstanding: float, annual_revenue: float) -> str:
    if risk_level == "critical":
        return "Critical payment risk — consider ceasing further credit extension to this entity and pursue debt recovery through formal channels"
    elif risk_level == "high":
        return "High payment risk — require partial upfront payment and implement milestone-based billing; consider invoice factoring"
    elif risk_level == "medium":
        return "Moderate payment risk — monitor closely, implement stricter credit terms, and follow up on overdue invoices within 30 days"
    else:
        return "Acceptable payment risk — maintain standard credit terms but continue monitoring payment patterns"


engine = PublicSectorEngine()


def assess_public_sector_exposure(revenue_from_gov: float, total_revenue: float) -> float:
    return engine.assess_public_sector_exposure(revenue_from_gov, total_revenue)


def compute_gov_payment_risk(outstanding_amount: float, avg_delay_days: float, gov_entity_type: str = "federal_ministry", total_annual_revenue: float = 0, number_of_invoices: int = 1) -> Dict[str, Any]:
    return engine.compute_gov_payment_risk(outstanding_amount, avg_delay_days, gov_entity_type, total_annual_revenue, number_of_invoices)
