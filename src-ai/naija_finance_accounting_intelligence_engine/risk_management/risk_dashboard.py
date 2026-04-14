# Author: Quadri Atharu
"""Risk dashboard generation and composite risk scoring."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class RiskDashboardEngine:
    """Risk dashboard generation and composite risk scoring engine."""

    def generate_dashboard(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a comprehensive risk dashboard."""
        credit_risk_score = float(data.get("credit_risk_score", 0))
        liquidity_risk_score = float(data.get("liquidity_risk_score", 0))
        market_risk_score = float(data.get("market_risk_score", 0))
        operational_risk_score = float(data.get("operational_risk_score", 0))
        compliance_risk_score = float(data.get("compliance_risk_score", 0))

        weights = {
            "credit": 0.25,
            "liquidity": 0.25,
            "market": 0.20,
            "operational": 0.15,
            "compliance": 0.15,
        }

        composite_score = round(
            credit_risk_score * weights["credit"] +
            liquidity_risk_score * weights["liquidity"] +
            market_risk_score * weights["market"] +
            operational_risk_score * weights["operational"] +
            compliance_risk_score * weights["compliance"],
            2,
        )

        if composite_score <= 20:
            overall_risk = "low"
            color = "green"
        elif composite_score <= 40:
            overall_risk = "moderate"
            color = "yellow"
        elif composite_score <= 60:
            overall_risk = "high"
            color = "orange"
        else:
            overall_risk = "critical"
            color = "red"

        return {
            "dashboard_type": "risk_overview",
            "composite_risk_score": composite_score,
            "overall_risk_level": overall_risk,
            "color_code": color,
            "risk_categories": {
                "credit": {"score": credit_risk_score, "weight": weights["credit"], "contribution": round(credit_risk_score * weights["credit"], 2)},
                "liquidity": {"score": liquidity_risk_score, "weight": weights["liquidity"], "contribution": round(liquidity_risk_score * weights["liquidity"], 2)},
                "market": {"score": market_risk_score, "weight": weights["market"], "contribution": round(market_risk_score * weights["market"], 2)},
                "operational": {"score": operational_risk_score, "weight": weights["operational"], "contribution": round(operational_risk_score * weights["operational"], 2)},
                "compliance": {"score": compliance_risk_score, "weight": weights["compliance"], "contribution": round(compliance_risk_score * weights["compliance"], 2)},
            },
            "top_risks": self._identify_top_risks(data),
            "recommendations": self._generate_recommendations(overall_risk, composite_score),
            "generated_at": datetime.now().isoformat(),
        }

    def _identify_top_risks(self, data: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Identify top risk areas."""
        risk_scores = {
            "credit": float(data.get("credit_risk_score", 0)),
            "liquidity": float(data.get("liquidity_risk_score", 0)),
            "market": float(data.get("market_risk_score", 0)),
            "operational": float(data.get("operational_risk_score", 0)),
            "compliance": float(data.get("compliance_risk_score", 0)),
        }
        sorted_risks = sorted(risk_scores.items(), key=lambda x: x[1], reverse=True)
        return [{"category": k, "score": v, "severity": "critical" if v > 60 else ("high" if v > 40 else "moderate")} for k, v in sorted_risks[:3]]

    @staticmethod
    def _generate_recommendations(overall: str, score: float) -> List[str]:
        """Generate risk mitigation recommendations."""
        if overall == "critical":
            return ["URGENT: Implement immediate risk mitigation measures", "Escalate to board risk committee", "Review all risk limits and tighten controls"]
        elif overall == "high":
            return ["Increase monitoring frequency", "Review and update risk appetite statement", "Develop contingency plans for top risk areas"]
        elif overall == "moderate":
            return ["Continue regular monitoring", "Review risk trends for deterioration", "Ensure risk policies are current"]
        return ["Maintain current risk management practices", "Periodic review of risk appetite"]

    def health_check(self) -> bool:
        return True
