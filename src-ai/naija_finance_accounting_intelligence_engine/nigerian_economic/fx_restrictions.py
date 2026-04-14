# Author: Quadri Atharu
"""FX restrictions and import premium computation for Nigerian businesses.

Implements CBN foreign exchange restriction lists, import premium
calculations, and FX window analysis per Nigerian monetary policy.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.logging import get_logger

logger = get_logger(__name__)

CBN_RESTRICTED_IMPORTS = {
    "rice", "poultry", "vegetable_oil", "palm_oil", "beef", "pork",
    "toothpicks", "spaghetti", "noodles", "cement", "medicaments",
    "woven_sacks", "soap", "cosmetics", "textile_fabrics", "second_hand_clothing",
    "footwear", "bags", "furniture", "second_hand_vehicles",
}

FX_WINDOWS = {
    "cbn_official": {"description": "CBN official window (I&E)", "typical_spread": 0.0},
    "bdc": {"description": "Bureau de Change window", "typical_spread": 0.15},
    "parallel": {"description": "Parallel/informal market", "typical_spread": 0.25},
    "smis": {"description": "Small and Medium Enterprises Intervention", "typical_spread": 0.0},
    "ptxp": {"description": "Promoters of Technology Export Program", "typical_spread": 0.0},
}

IMPORT_PREMIUM_RATES = {
    "raw_materials": 0.0,
    "capital_equipment": 0.0,
    "semi_finished_goods": 0.05,
    "finished_goods": 0.15,
    "luxury_goods": 0.30,
    "restricted_goods": 0.50,
}


class FxRestrictionsEngine:
    """Nigerian FX restriction and import premium computation engine."""

    def check_fx_restriction(self, item_category: str, transaction_type: str = "import") -> Dict[str, Any]:
        """Check if an item is subject to FX restrictions for imports."""
        item = item_category.lower().strip().replace(" ", "_")
        is_restricted = item in CBN_RESTRICTED_IMPORTS

        restriction_details: Dict[str, Any] = {
            "item_category": item_category,
            "transaction_type": transaction_type,
            "is_restricted": is_restricted,
            "fx_available": not is_restricted,
        }

        if is_restricted:
            restriction_details.update({
                "restriction_source": "CBN Foreign Exchange Restriction List",
                "alternative": "Source from local manufacturers or apply for special CBN approval",
                "penalty_risk": "Import duty may apply at 70% + VAT 7.5% on restricted items",
                "restriction_type": "full_fx_restriction",
            })
        else:
            window = self._determine_fx_window(item)
            restriction_details.update({
                "restriction_type": "none",
                "recommended_fx_window": window,
                "fx_window_details": FX_WINDOWS.get(window, {}),
            })

        restriction_details["checked_at"] = datetime.now().isoformat()

        if is_restricted:
            logger.warning("fx_restriction_detected", item=item_category)

        return restriction_details

    def compute_import_premium(
        self,
        item_category: str,
        invoice_value_usd: float,
        cbn_rate: float,
        parallel_rate: Optional[float] = None,
        quantity: float = 1,
    ) -> Dict[str, Any]:
        """Compute the import premium and effective NGN cost for imports.

        Calculates: FX premium, import duty impact, and total landed cost
        in NGN terms for imported items.
        """
        item = item_category.lower().strip().replace(" ", "_")
        premium_rate = self._get_import_premium_rate(item)
        is_restricted = item in CBN_RESTRICTED_IMPORTS

        effective_rate = cbn_rate
        fx_premium_per_usd = 0.0

        if parallel_rate and parallel_rate > cbn_rate:
            fx_premium_per_usd = round(parallel_rate - cbn_rate, 2)
            effective_rate = parallel_rate if is_restricted else cbn_rate

        invoice_value_ngn_cbn = round(invoice_value_usd * cbn_rate, 2)
        invoice_value_ngn_effective = round(invoice_value_usd * effective_rate, 2)

        import_premium_amount = round(invoice_value_usd * cbn_rate * premium_rate, 2)
        total_landed_cost = round(invoice_value_ngn_effective + import_premium_amount, 2)

        duty_rate = 0.70 if is_restricted else 0.05
        duty_amount = round(invoice_value_ngn_effective * duty_rate, 2)
        vat_rate = 0.075
        vat_amount = round((invoice_value_ngn_effective + duty_amount) * vat_rate, 2) if not is_restricted else 0

        total_cost_with_duties = round(total_landed_cost + duty_amount + vat_amount, 2)

        return {
            "item_category": item_category,
            "is_restricted_import": is_restricted,
            "invoice_value_usd": invoice_value_usd,
            "quantity": quantity,
            "fx_rates": {
                "cbn_official": cbn_rate,
                "parallel": parallel_rate,
                "effective_used": effective_rate,
                "fx_premium_per_usd": fx_premium_per_usd,
            },
            "cost_breakdown": {
                "invoice_value_ngn_cbn": invoice_value_ngn_cbn,
                "invoice_value_ngn_effective": invoice_value_ngn_effective,
                "import_premium_rate": premium_rate,
                "import_premium_amount": import_premium_amount,
                "customs_duty_rate": duty_rate,
                "customs_duty_amount": duty_amount,
                "vat_rate": vat_rate,
                "vat_amount": vat_amount,
                "total_landed_cost": total_cost_with_duties,
            },
            "premium_impact_pct": round(import_premium_amount / invoice_value_ngn_cbn, 4) if invoice_value_ngn_cbn else 0,
            "computed_at": datetime.now().isoformat(),
        }

    def compare_fx_windows(
        self,
        amount_usd: float,
        cbn_rate: float,
        parallel_rate: float,
        bdc_rate: Optional[float] = None,
    ) -> Dict[str, Any]:
        """Compare costs across different FX windows for a given USD amount."""
        windows: Dict[str, Dict[str, Any]] = {}

        windows["cbn_official"] = {
            "rate": cbn_rate,
            "amount_ngn": round(amount_usd * cbn_rate, 2),
            "availability": "subject_to_cbn_allocation",
            "processing_days": "1-5",
        }

        if bdc_rate:
            windows["bdc"] = {
                "rate": bdc_rate,
                "amount_ngn": round(amount_usd * bdc_rate, 2),
                "premium_vs_cbn": round((bdc_rate - cbn_rate) * amount_usd, 2),
                "availability": "market_rate",
                "processing_days": "1-2",
            }

        windows["parallel"] = {
            "rate": parallel_rate,
            "amount_ngn": round(amount_usd * parallel_rate, 2),
            "premium_vs_cbn": round((parallel_rate - cbn_rate) * amount_usd, 2),
            "availability": "informal_market",
            "processing_days": "same_day",
            "compliance_risk": "high",
        }

        best_rate = min(
            (w["rate"] for w in windows.values() if "rate" in w),
            default=cbn_rate,
        )
        worst_rate = max(
            (w["rate"] for w in windows.values() if "rate" in w),
            default=parallel_rate,
        )

        return {
            "amount_usd": amount_usd,
            "windows": windows,
            "best_rate": best_rate,
            "worst_rate": worst_rate,
            "max_spread_ngn": round((worst_rate - best_rate) * amount_usd, 2),
            "max_spread_pct": round((worst_rate - best_rate) / best_rate, 4) if best_rate else 0,
            "recommendation": "Use CBN official window for compliance; parallel market carries regulatory risk",
            "compared_at": datetime.now().isoformat(),
        }

    def _determine_fx_window(self, item_category: str) -> str:
        """Determine the recommended FX window based on item category."""
        item = item_category.lower().strip().replace(" ", "_")
        if item in ("raw_materials", "capital_equipment", "machinery"):
            return "smis"
        return "cbn_official"

    def _get_import_premium_rate(self, item_category: str) -> float:
        """Get the import premium rate for a category."""
        for key, rate in IMPORT_PREMIUM_RATES.items():
            if key in item_category:
                return rate
        if item_category in CBN_RESTRICTED_IMPORTS:
            return IMPORT_PREMIUM_RATES["restricted_goods"]
        return IMPORT_PREMIUM_RATES.get("finished_goods", 0.15)

    def health_check(self) -> bool:
        return True
