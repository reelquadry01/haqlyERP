# Author: Quadri Atharu
"""Tax risk flag detection for Nigerian tax compliance."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)

RISK_THRESHOLDS = {
    "vat_ratio_high": {"threshold": 0.15, "description": "VAT as % of revenue exceeds 15% — possible over-claim"},
    "vat_ratio_low": {"threshold": 0.01, "description": "VAT as % of revenue below 1% — possible under-reporting"},
    "wht_vs_total_payments": {"threshold": 0.03, "description": "WHT as % of total payments below 3% — possible non-deduction"},
    "cit_effective_rate_low": {"threshold": 0.10, "description": "Effective CIT rate below 10% — possible aggressive tax planning"},
    "cit_effective_rate_zero": {"threshold": 0.0, "description": "Zero CIT on positive profit — minimum tax may apply"},
    "related_party_transactions_pct": {"threshold": 0.25, "description": "Related party transactions exceed 25% of total — transfer pricing risk"},
    "capital_allowance_vs_ppe": {"threshold": 0.50, "description": "Capital allowances exceed 50% of PPE additions — possible over-claim"},
    "input_vat_vs_output_vat": {"threshold": 0.95, "description": "Input VAT exceeds 95% of output VAT — possible over-claim or fraud"},
    "late_filing_frequency": {"threshold": 3, "description": "Late filing 3+ times — compliance risk"},
    "unreconciled_tax_accounts": {"threshold": 0.05, "description": "Unreconciled tax balances exceed 5% of total tax payable"},
}

SEVERITY_LEVELS = {"low": 1, "medium": 2, "high": 3, "critical": 4}


class TaxRiskDetector:
    """Detect tax risk flags and compliance issues for Nigerian entities."""

    def detect_risks(self, tax_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run all risk detection checks and return flagged items."""
        flags: List[Dict[str, Any]] = []

        self._check_vat_ratio(tax_data, flags)
        self._check_input_output_vat_ratio(tax_data, flags)
        self._check_wht_coverage(tax_data, flags)
        self._check_cit_effective_rate(tax_data, flags)
        self._check_related_party(tax_data, flags)
        self._check_capital_allowance_ratio(tax_data, flags)
        self._check_late_filing(tax_data, flags)
        self._check_unreconciled_tax(tax_data, flags)

        max_severity = max((SEVERITY_LEVELS.get(f.get("severity", "medium"), 2) for f in flags), default=0)
        overall_risk = "critical" if max_severity >= 4 else ("high" if max_severity >= 3 else ("medium" if max_severity >= 2 else "low"))

        result: Dict[str, Any] = {
            "total_flags": len(flags),
            "flags": flags,
            "overall_risk_level": overall_risk,
            "risk_score": self._compute_risk_score(flags),
            "checked_at": datetime.now().isoformat(),
        }

        logger.info("tax_risks_detected", total_flags=len(flags), overall_risk=overall_risk)
        return result

    def _check_vat_ratio(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check VAT-to-revenue ratio."""
        revenue = float(data.get("revenue", 0))
        output_vat = float(data.get("output_vat", 0))
        if revenue <= 0:
            return
        ratio = output_vat / revenue
        high_threshold = RISK_THRESHOLDS["vat_ratio_high"]["threshold"]
        low_threshold = RISK_THRESHOLDS["vat_ratio_low"]["threshold"]

        if ratio > high_threshold:
            flags.append(self._make_flag("vat_ratio_high", "VAT", RISK_THRESHOLDS["vat_ratio_high"]["description"], ratio, high_threshold, "medium"))
        if ratio < low_threshold:
            flags.append(self._make_flag("vat_ratio_low", "VAT", RISK_THRESHOLDS["vat_ratio_low"]["description"], ratio, low_threshold, "high", "Possible under-reporting of output VAT — verify all taxable supplies are captured"))

    def _check_input_output_vat_ratio(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check input VAT vs output VAT ratio."""
        output_vat = float(data.get("output_vat", 0))
        input_vat = float(data.get("input_vat", 0))
        if output_vat <= 0:
            return
        ratio = input_vat / output_vat
        threshold = RISK_THRESHOLDS["input_vat_vs_output_vat"]["threshold"]
        if ratio > threshold:
            flags.append(self._make_flag("input_vat_vs_output_vat", "VAT", RISK_THRESHOLDS["input_vat_vs_output_vat"]["description"], ratio, threshold, "critical", "Verify input VAT claims — risk of fraudulent claims or over-claiming"))

    def _check_wht_coverage(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check WHT deduction coverage."""
        total_payments = float(data.get("total_payments", 0))
        wht_deducted = float(data.get("wht_deducted", 0))
        if total_payments <= 0:
            return
        ratio = wht_deducted / total_payments
        threshold = RISK_THRESHOLDS["wht_vs_total_payments"]["threshold"]
        if ratio < threshold:
            flags.append(self._make_flag("wht_vs_total_payments", "WHT", RISK_THRESHOLDS["wht_vs_total_payments"]["description"], ratio, threshold, "medium", "Some payments may not have WHT deducted — check all qualifying payments"))

    def _check_cit_effective_rate(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check effective CIT rate."""
        profit_before_tax = float(data.get("profit_before_tax", 0))
        cit_payable = float(data.get("cit_payable", 0))
        if profit_before_tax <= 0:
            return
        effective_rate = cit_payable / profit_before_tax
        low_threshold = RISK_THRESHOLDS["cit_effective_rate_low"]["threshold"]
        zero_threshold = RISK_THRESHOLDS["cit_effective_rate_zero"]["threshold"]

        if effective_rate <= zero_threshold and profit_before_tax > 0:
            flags.append(self._make_flag("cit_effective_rate_zero", "CIT", RISK_THRESHOLDS["cit_effective_rate_zero"]["description"], effective_rate, zero_threshold, "high", "Zero CIT on positive profit — minimum tax rules may apply"))
        elif effective_rate < low_threshold:
            flags.append(self._make_flag("cit_effective_rate_low", "CIT", RISK_THRESHOLDS["cit_effective_rate_low"]["description"], effective_rate, low_threshold, "medium"))

    def _check_related_party(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check related party transaction ratio."""
        total_transactions = float(data.get("total_transactions", 0))
        related_party_transactions = float(data.get("related_party_transactions", 0))
        if total_transactions <= 0:
            return
        ratio = related_party_transactions / total_transactions
        threshold = RISK_THRESHOLDS["related_party_transactions_pct"]["threshold"]
        if ratio > threshold:
            flags.append(self._make_flag("related_party_transactions_pct", "CIT", RISK_THRESHOLDS["related_party_transactions_pct"]["description"], ratio, threshold, "high", "Transfer pricing documentation required per FIRS guidelines"))

    def _check_capital_allowance_ratio(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check capital allowance vs PPE additions."""
        ppe_additions = float(data.get("ppe_additions", 0))
        capital_allowances = float(data.get("capital_allowances", 0))
        if ppe_additions <= 0:
            return
        ratio = capital_allowances / ppe_additions
        threshold = RISK_THRESHOLDS["capital_allowance_vs_ppe"]["threshold"]
        if ratio > threshold:
            flags.append(self._make_flag("capital_allowance_vs_ppe", "CIT", RISK_THRESHOLDS["capital_allowance_vs_ppe"]["description"], ratio, threshold, "medium"))

    def _check_late_filing(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check late filing frequency."""
        late_count = int(data.get("late_filing_count", 0))
        threshold = RISK_THRESHOLDS["late_filing_frequency"]["threshold"]
        if late_count >= threshold:
            flags.append(self._make_flag("late_filing_frequency", "ALL", RISK_THRESHOLDS["late_filing_frequency"]["description"], late_count, threshold, "high", "Implement compliance calendar to avoid further penalties"))

    def _check_unreconciled_tax(self, data: Dict[str, Any], flags: List[Dict[str, Any]]) -> None:
        """Check unreconciled tax accounts."""
        total_tax_payable = float(data.get("total_tax_payable", 0))
        unreconciled = float(data.get("unreconciled_tax_balance", 0))
        if total_tax_payable <= 0:
            return
        ratio = unreconciled / total_tax_payable
        threshold = RISK_THRESHOLDS["unreconciled_tax_accounts"]["threshold"]
        if ratio > threshold:
            flags.append(self._make_flag("unreconciled_tax_accounts", "ALL", RISK_THRESHOLDS["unreconciled_tax_accounts"]["description"], ratio, threshold, "medium", "Reconcile tax accounts before filing period-end returns"))

    @staticmethod
    def _make_flag(
        risk_type: str,
        tax_type: str,
        description: str,
        metric_value: float,
        threshold_value: float,
        severity: str = "medium",
        recommendation: str = "",
    ) -> Dict[str, Any]:
        """Create a standardized risk flag."""
        deviation = round(abs(metric_value - threshold_value) / max(abs(threshold_value), 0.01) * 100, 2) if threshold_value else 0
        return {
            "risk_type": risk_type,
            "severity": severity,
            "tax_type": tax_type,
            "description": description,
            "metric_value": round(metric_value, 4),
            "threshold_value": round(threshold_value, 4),
            "deviation_pct": deviation,
            "recommendation": recommendation or f"Review and verify {tax_type} computations and filings",
            "detected_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _compute_risk_score(flags: List[Dict[str, Any]]) -> float:
        """Compute an overall risk score from 0-100."""
        if not flags:
            return 0.0
        score = sum(SEVERITY_LEVELS.get(f.get("severity", "medium"), 2) * 10 for f in flags)
        return min(round(score, 2), 100.0)

    def health_check(self) -> bool:
        return True
