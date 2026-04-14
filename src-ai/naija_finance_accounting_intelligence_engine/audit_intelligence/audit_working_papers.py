# Author: Quadri Atharu
"""Audit working paper generation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.logging import get_logger

logger = get_logger(__name__)


class AuditWorkingPapersEngine:
    """Audit working paper generation engine."""

    def generate_working_paper(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a complete audit working paper."""
        company_id = data.get("company_id", "")
        audit_area = data.get("audit_area", "general")
        period_start = data.get("period_start", "")
        period_end = data.get("period_end", "")
        auditor = data.get("auditor", "")
        procedures = data.get("procedures", [])

        results: List[Dict[str, Any]] = []
        for proc in procedures:
            results.append({
                "procedure": proc.get("description", ""),
                "type": proc.get("type", "substantive"),
                "conclusion": proc.get("conclusion", "satisfactory"),
                "exceptions": proc.get("exceptions", []),
                "evidence_references": proc.get("evidence_references", []),
            })

        exceptions_found = [r for r in results if r.get("exceptions")]

        return {
            "working_paper_id": f"WP-{audit_area}-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            "company_id": company_id,
            "audit_area": audit_area,
            "period_start": period_start,
            "period_end": period_end,
            "auditor": auditor,
            "prepared_at": datetime.now().isoformat(),
            "procedures_performed": len(procedures),
            "procedure_results": results,
            "exceptions_count": len(exceptions_found),
            "exceptions": exceptions_found,
            "overall_conclusion": "exceptions_noted" if exceptions_found else "no_exceptions",
        }

    def generate_lead_schedule(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a lead schedule summarizing a financial statement line."""
        line_item = data.get("line_item", "")
        amounts: List[Dict[str, Any]] = data.get("amounts", [])
        total = round(sum(float(a.get("amount", 0)) for a in amounts), 2)

        return {
            "schedule_type": "lead_schedule",
            "line_item": line_item,
            "sub_items": amounts,
            "total": total,
            "cross_reference": data.get("cross_reference", ""),
            "generated_at": datetime.now().isoformat(),
        }

    def generate_substantive_test_summary(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate a summary of substantive testing performed."""
        tests: List[Dict[str, Any]] = data.get("tests", [])
        passed = [t for t in tests if t.get("result") == "pass"]
        failed = [t for t in tests if t.get("result") == "fail"]

        return {
            "total_tests": len(tests),
            "tests_passed": len(passed),
            "tests_failed": len(failed),
            "pass_rate": round(len(passed) / len(tests), 4) if tests else 0,
            "failed_tests": failed,
            "overall_assessment": "satisfactory" if not failed else "exceptions_noted",
            "generated_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
