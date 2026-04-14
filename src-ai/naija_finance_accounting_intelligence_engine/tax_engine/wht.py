# Author: Quadri Atharu
"""Nigerian Withholding Tax (WHT) computation engine."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import TaxError
from ..core.logging import get_logger

logger = get_logger(__name__)

WHT_RATES: Dict[str, Dict[str, Any]] = {
    "contractors": {"rate": 0.05, "threshold": 0, "description": "Contractors and sub-contractors"},
    "consultancy": {"rate": 0.05, "threshold": 0, "description": "Professional consultancy services"},
    "management_services": {"rate": 0.05, "threshold": 0, "description": "Management services"},
    "technical_services": {"rate": 0.05, "threshold": 0, "description": "Technical services fees"},
    "professional_services": {"rate": 0.05, "threshold": 0, "description": "Professional services (legal, accounting, etc.)"},
    "commission": {"rate": 0.05, "threshold": 0, "description": "Commission payments"},
    "royalties": {"rate": 0.05, "threshold": 0, "description": "Royalty payments"},
    "entertainment": {"rate": 0.05, "threshold": 0, "description": "Contractual entertainment"},
    "director_fees": {"rate": 0.10, "threshold": 0, "description": "Director's fees"},
    "interest": {"rate": 0.10, "threshold": 0, "description": "Interest income on deposits"},
    "dividends": {"rate": 0.10, "threshold": 0, "description": "Dividend payments"},
    "rent": {"rate": 0.10, "threshold": 0, "description": "Rent on land/building"},
    "hire_purchase": {"rate": 0.05, "threshold": 0, "description": "Hire purchase payments"},
}

WHT_TREATMENT = {
    "5_percent_categories": {
        "tax_credit_available": True,
        "note": "WHT at 5% can be used as tax credit against CIT liability",
        "categories": ["contractors", "consultancy", "management_services", "technical_services", "professional_services", "commission", "royalties", "entertainment", "hire_purchase"],
    },
    "10_percent_categories": {
        "tax_credit_available": False,
        "note": "WHT at 10% is a final tax — no further credit against CIT",
        "categories": ["interest", "dividends", "rent", "director_fees"],
    },
}


class WhtEngine:
    """Nigerian Withholding Tax computation engine."""

    def compute_wht(self, payment_amount: float, category: str = "consultancy") -> Dict[str, Any]:
        """Compute WHT on a payment based on category."""
        if payment_amount < 0:
            raise TaxError("Payment amount cannot be negative for WHT computation")

        cat = category.lower().strip()
        rate_info = WHT_RATES.get(cat)

        if rate_info is None:
            raise TaxError(f"Unknown WHT category: {category}", details={"available_categories": list(WHT_RATES.keys())})

        rate = rate_info["rate"]
        wht_amount = round(payment_amount * rate, 2)
        net_payment = round(payment_amount - wht_amount, 2)

        is_final_tax = rate == 0.10
        treatment = WHT_TREATMENT["10_percent_categories"] if is_final_tax else WHT_TREATMENT["5_percent_categories"]

        result: Dict[str, Any] = {
            "tax_type": "WHT",
            "category": cat,
            "category_description": rate_info["description"],
            "payment_amount": round(payment_amount, 2),
            "wht_rate": rate,
            "wht_amount": wht_amount,
            "net_payment": net_payment,
            "is_final_tax": is_final_tax,
            "tax_credit_available": not is_final_tax,
            "treatment_note": treatment["note"],
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("wht_computed", category=cat, payment=payment_amount, wht=wht_amount, rate=rate)
        return result

    def compute_batch_wht(self, payments: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Compute WHT on a batch of payments."""
        results: List[Dict[str, Any]] = []
        total_wht = 0.0
        total_gross = 0.0

        for payment in payments:
            amount = float(payment.get("amount", 0))
            category = payment.get("category", "consultancy")
            result = self.compute_wht(amount, category)
            results.append({**result, "payment_reference": payment.get("reference")})
            total_wht += result["wht_amount"]
            total_gross += amount

        return {
            "tax_type": "WHT",
            "batch_size": len(payments),
            "total_gross_payments": round(total_gross, 2),
            "total_wht_deducted": round(total_wht, 2),
            "total_net_payments": round(total_gross - total_wht, 2),
            "line_items": results,
            "computed_at": datetime.now().isoformat(),
        }

    def compute_wht_credit(self, wht_deducted: float, cit_liability: float) -> Dict[str, Any]:
        """Compute the WHT tax credit available against CIT liability."""
        credit_amount = min(wht_deducted, cit_liability)
        excess = max(wht_deducted - cit_liability, 0)

        return {
            "wht_deducted": round(wht_deducted, 2),
            "cit_liability": round(cit_liability, 2),
            "wht_credit_used": round(credit_amount, 2),
            "excess_wht": round(excess, 2),
            "net_cit_after_credit": round(cit_liability - credit_amount, 2),
            "note": "Only 5% WHT categories qualify as tax credit; 10% categories are final tax",
        }

    def generate_wht_certificate_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate data for a WHT withholding certificate."""
        payment_amount = float(data.get("payment_amount", 0))
        category = data.get("category", "consultancy")
        wht_result = self.compute_wht(payment_amount, category)

        return {
            "certificate_type": "WHT",
            "payer_name": data.get("payer_name", ""),
            "payer_tin": data.get("payer_tin", ""),
            "recipient_name": data.get("recipient_name", ""),
            "recipient_tin": data.get("recipient_tin", ""),
            "payment_description": wht_result["category_description"],
            "payment_amount": wht_result["payment_amount"],
            "wht_rate": wht_result["wht_rate"],
            "wht_amount": wht_result["wht_amount"],
            "is_final_tax": wht_result["is_final_tax"],
            "payment_date": data.get("payment_date", ""),
            "period": data.get("period", ""),
        }

    def list_categories(self) -> Dict[str, Any]:
        """List all available WHT categories and rates."""
        categories: List[Dict[str, Any]] = []
        for cat, info in WHT_RATES.items():
            categories.append({
                "category": cat,
                "rate": info["rate"],
                "rate_pct": f"{info['rate'] * 100:.0f}%",
                "description": info["description"],
                "is_final_tax": info["rate"] == 0.10,
            })
        return {"categories": categories, "total_categories": len(categories)}

    def health_check(self) -> bool:
        return True
