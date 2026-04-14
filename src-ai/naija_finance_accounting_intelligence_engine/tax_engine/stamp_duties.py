# Author: Quadri Atharu
"""Nigerian Stamp Duties computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

STAMP_DUTY_SCHEDULE: Dict[str, Dict[str, Any]] = {
    "general": {"rate_type": "flat", "amount": 100, "description": "General contract/agreement"},
    "lease": {"rate_type": "progressive", "amount": 0, "description": "Lease agreement", "rates": [
        {"threshold": 7, "rate": 0.005, "note": "Lease ≤ 7 years: 0.5% of consideration"},
        {"threshold": 35, "rate": 0.01, "note": "Lease 7-35 years: 1% of consideration"},
        {"threshold": 999, "rate": 0.0125, "note": "Lease > 35 years: 1.25% of consideration"},
    ]},
    "sale_of_property": {"rate_type": "ad_valorem", "rate": 0.015, "amount": 0, "description": "Conveyance/sale of property (1.5%)"},
    "mortgage": {"rate_type": "ad_valorem", "rate": 0.01, "amount": 0, "description": "Mortgage/debenture (1%)"},
    "transfer_of_shares": {"rate_type": "ad_valorem", "rate": 0.005, "amount": 0, "description": "Transfer of shares (0.5%)"},
    "insurance_policy": {"rate_type": "flat", "amount": 100, "description": "Insurance policy"},
    "cheque": {"rate_type": "flat", "amount": 50, "description": "Cheque (N50 per cheque)"},
    "electronic_transfer": {"rate_type": "flat", "amount": 50, "description": "Electronic transfer (N50 per transaction above N10,000)"},
    "receipt": {"rate_type": "flat", "amount": 50, "description": "Receipt for money or property (above N10,000)"},
    "power_of_attorney": {"rate_type": "flat", "amount": 500, "description": "Power of attorney"},
    "affidavit": {"rate_type": "flat", "amount": 200, "description": "Affidavit/deposition"},
    "deed_of_gift": {"rate_type": "ad_valorem", "rate": 0.015, "amount": 0, "description": "Deed of gift (1.5%)"},
    "hire_purchase": {"rate_type": "ad_valorem", "rate": 0.005, "amount": 0, "description": "Hire purchase agreement (0.5%)"},
    "partnership_deed": {"rate_type": "flat", "amount": 1000, "description": "Partnership deed"},
    "proxy": {"rate_type": "flat", "amount": 100, "description": "Proxy form for company meetings"},
    "bill_of_exchange": {"rate_type": "ad_valorem", "rate": 0.0025, "amount": 0, "description": "Bill of exchange (0.25%)"},
}

ELECTRONIC_TRANSFER_THRESHOLD = 10_000
RECEIPT_THRESHOLD = 10_000


class StampDutyEngine:
    """Nigerian Stamp Duties computation engine."""

    def compute_stamp_duty(self, document_type: str = "general", value: float = 0, lease_term_years: int = 0) -> Dict[str, Any]:
        """Compute stamp duty for a document type."""
        doc_type = document_type.lower().strip()
        schedule = STAMP_DUTY_SCHEDULE.get(doc_type)

        if schedule is None:
            raise TaxError(f"Unknown document type: {document_type}", details={"available_types": list(STAMP_DUTY_SCHEDULE.keys())})

        if schedule["rate_type"] == "flat":
            duty = schedule["amount"]
            if doc_type == "electronic_transfer" and value <= ELECTRONIC_TRANSFER_THRESHOLD:
                duty = 0
            elif doc_type == "receipt" and value <= RECEIPT_THRESHOLD:
                duty = 0
            return self._flat_result(doc_type, value, duty, schedule)

        elif schedule["rate_type"] == "ad_valorem":
            duty = round(value * schedule["rate"], 2)
            return self._ad_valorem_result(doc_type, value, duty, schedule)

        elif schedule["rate_type"] == "progressive":
            duty = self._compute_lease_duty(value, lease_term_years, schedule)
            return self._progressive_result(doc_type, value, duty, lease_term_years, schedule)

        raise TaxError(f"Unknown rate type: {schedule['rate_type']}")

    def compute_batch_stamp_duty(self, documents: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Compute stamp duty for a batch of documents."""
        results: List[Dict[str, Any]] = []
        total_duty = 0.0

        for doc in documents:
            result = self.compute_stamp_duty(
                document_type=doc.get("document_type", "general"),
                value=float(doc.get("value", 0)),
                lease_term_years=int(doc.get("lease_term_years", 0)),
            )
            results.append(result)
            total_duty += result["stamp_duty"]

        return {
            "tax_type": "STAMP_DUTY",
            "batch_size": len(documents),
            "total_stamp_duty": round(total_duty, 2),
            "individual_duties": results,
            "computed_at": datetime.now().isoformat(),
        }

    def list_document_types(self) -> Dict[str, Any]:
        """List all stamp duty document types and rates."""
        types: List[Dict[str, Any]] = []
        for doc_type, info in STAMP_DUTY_SCHEDULE.items():
            entry: Dict[str, Any] = {
                "document_type": doc_type,
                "description": info["description"],
                "rate_type": info["rate_type"],
            }
            if info["rate_type"] == "flat":
                entry["flat_amount"] = info["amount"]
            elif info["rate_type"] == "ad_valorem":
                entry["rate"] = f"{info['rate'] * 100:.2f}%"
            elif info["rate_type"] == "progressive":
                entry["rates"] = info.get("rates", [])
            types.append(entry)
        return {"document_types": types, "total_types": len(types)}

    def _compute_lease_duty(self, value: float, term_years: int, schedule: Dict[str, Any]) -> float:
        """Compute progressive stamp duty for lease agreements."""
        rates = schedule.get("rates", [])
        applicable_rate = 0.005

        for rate_info in rates:
            if term_years <= rate_info["threshold"]:
                applicable_rate = rate_info["rate"]
                break
        else:
            if rates:
                applicable_rate = rates[-1]["rate"]

        return round(value * applicable_rate, 2)

    def _flat_result(self, doc_type: str, value: float, duty: float, schedule: Dict[str, Any]) -> Dict[str, Any]:
        """Build result for flat-rate stamp duty."""
        return {
            "tax_type": "STAMP_DUTY",
            "document_type": doc_type,
            "description": schedule["description"],
            "rate_type": "flat",
            "document_value": round(value, 2),
            "stamp_duty": round(duty, 2),
            "computed_at": datetime.now().isoformat(),
        }

    def _ad_valorem_result(self, doc_type: str, value: float, duty: float, schedule: Dict[str, Any]) -> Dict[str, Any]:
        """Build result for ad valorem stamp duty."""
        return {
            "tax_type": "STAMP_DUTY",
            "document_type": doc_type,
            "description": schedule["description"],
            "rate_type": "ad_valorem",
            "rate": schedule["rate"],
            "rate_pct": f"{schedule['rate'] * 100:.2f}%",
            "document_value": round(value, 2),
            "stamp_duty": round(duty, 2),
            "computed_at": datetime.now().isoformat(),
        }

    def _progressive_result(self, doc_type: str, value: float, duty: float, term: int, schedule: Dict[str, Any]) -> Dict[str, Any]:
        """Build result for progressive stamp duty."""
        return {
            "tax_type": "STAMP_DUTY",
            "document_type": doc_type,
            "description": schedule["description"],
            "rate_type": "progressive",
            "lease_term_years": term,
            "rates_schedule": schedule.get("rates", []),
            "document_value": round(value, 2),
            "stamp_duty": round(duty, 2),
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
