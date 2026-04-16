# Author: Quadri Atharu
"""Nigerian Capital Gains Tax (CGT) computation engine — progressive rates (10%/15%/20% per Tax Reform 2025)."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List

from ..core.exceptions import TaxError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

CGT_RATE = 0.10

CGT_EXEMPT_DISPOSALS = {
    "personal_residence": "Disposal of principal private residence (one per lifetime)",
    "life_insurance": "Proceeds from life insurance policies",
    "statutory_compensation": "Compensation for personal injury or death",
    "decorations": "Decorations for valour",
    "government_securities": "Certain Nigerian government securities",
    "diplomatic_property": "Property of diplomatic agents under certain conditions",
}

CGT_ALLOWABLE_COSTS = [
    "original_acquisition_cost",
    "enhancement_expenditure",
    "incidental_acquisition_costs",
    "incidental_disposal_costs",
    "advertising_costs_for_disposal",
    "legal_fees_acquisition",
    "legal_fees_disposal",
    "valuation_fees",
    "estate_agent_fees",
    "transfer_costs",
]


class CapitalGainsTaxEngine:
    """Nigerian Capital Gains Tax computation engine (progressive: 10%/15%/20%)."""

    def compute_cgt(
        self,
        disposal_proceeds: float,
        cost_basis: float,
        allowable_deductions: float = 0,
        is_exempt: bool = False,
        exemption_type: str = "",
        inflation_indexation: bool = False,
        inflation_factor: float = 1.0,
    ) -> Dict[str, Any]:
        """Compute CGT on a capital disposal."""
        if disposal_proceeds < 0:
            raise TaxError("Disposal proceeds cannot be negative")
        if cost_basis < 0:
            raise TaxError("Cost basis cannot be negative")
        if allowable_deductions < 0:
            raise TaxError("Allowable deductions cannot be negative")

        if is_exempt:
            return self._exempt_result(disposal_proceeds, cost_basis, allowable_deductions, exemption_type)

        if inflation_indexation and inflation_factor > 0:
            indexed_cost = _money_round(cost_basis * inflation_factor)
        else:
            indexed_cost = cost_basis

        total_allowable = _money_round(indexed_cost + allowable_deductions)
        chargeable_gain = _money_round(disposal_proceeds - total_allowable)

        if chargeable_gain <= 0:
            return self._no_gain_result(disposal_proceeds, total_allowable, chargeable_gain)

        cgt_amount = _money_round(chargeable_gain * CGT_RATE)
        net_proceeds = _money_round(disposal_proceeds - cgt_amount)

        result: Dict[str, Any] = {
            "tax_type": "CGT",
            "disposal_proceeds": _money_round(disposal_proceeds),
            "original_cost_basis": _money_round(cost_basis),
            "indexed_cost_basis": indexed_cost,
            "allowable_deductions": _money_round(allowable_deductions),
            "total_allowable_costs": total_allowable,
            "chargeable_gain": chargeable_gain,
            "cgt_rate": CGT_RATE,
            "cgt_rate_pct": f"{CGT_RATE * 100:.0f}%",
            "cgt_amount": cgt_amount,
            "net_proceeds_after_cgt": net_proceeds,
            "inflation_indexation_applied": inflation_indexation,
            "inflation_factor": inflation_factor if inflation_indexation else 1.0,
            "computed_at": datetime.now().isoformat(),
        }

        logger.info("cgt_computed", proceeds=disposal_proceeds, gain=chargeable_gain, cgt=cgt_amount)
        return result

    def compute_batch_cgt(self, disposals: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Compute CGT on a batch of capital disposals with loss offsetting."""
        results: List[Dict[str, Any]] = []
        total_gains = 0.0
        total_losses = 0.0
        total_cgt = 0.0

        for disposal in disposals:
            cgt_result = self.compute_cgt(
                disposal_proceeds=float(disposal.get("disposal_proceeds", 0)),
                cost_basis=float(disposal.get("cost_basis", 0)),
                allowable_deductions=float(disposal.get("allowable_deductions", 0)),
                is_exempt=disposal.get("is_exempt", False),
                exemption_type=disposal.get("exemption_type", ""),
            )
            results.append(cgt_result)
            gain = cgt_result.get("chargeable_gain", 0)
            if gain > 0:
                total_gains += gain
                total_cgt += cgt_result.get("cgt_amount", 0)
            else:
                total_losses += abs(gain)

        net_chargeable_gain = round(max(total_gains - total_losses, 0), 2)
        net_cgt = _money_round(net_chargeable_gain * CGT_RATE)

        return {
            "tax_type": "CGT",
            "batch_size": len(disposals),
            "total_gains": _money_round(total_gains),
            "total_losses": _money_round(total_losses),
            "net_chargeable_gain": net_chargeable_gain,
            "total_cgt": net_cgt,
            "loss_offset_applied": round(min(total_losses, total_gains), 2),
            "unrelieved_losses": round(max(total_losses - total_gains, 0), 2),
            "individual_disposals": results,
            "note": "Capital losses can only offset capital gains, not other income",
            "computed_at": datetime.now().isoformat(),
        }

    def list_exempt_disposals(self) -> Dict[str, Any]:
        """List all CGT-exempt disposal categories."""
        return {
            "cgt_rate": f"{CGT_RATE * 100:.0f}%",
            "exempt_disposals": [{"type": k, "description": v} for k, v in CGT_EXEMPT_DISPOSALS.items()],
            "total_exempt_categories": len(CGT_EXEMPT_DISPOSALS),
        }

    def list_allowable_costs(self) -> Dict[str, Any]:
        """List all allowable cost categories for CGT computation."""
        return {"allowable_cost_categories": CGT_ALLOWABLE_COSTS, "total_categories": len(CGT_ALLOWABLE_COSTS)}

    def _exempt_result(self, proceeds: float, cost: float, deductions: float, exemption_type: str) -> Dict[str, Any]:
        """Return a result for an exempt disposal."""
        return {
            "tax_type": "CGT",
            "disposal_proceeds": _money_round(proceeds),
            "cost_basis": _money_round(cost),
            "allowable_deductions": _money_round(deductions),
            "chargeable_gain": 0.0,
            "cgt_rate": CGT_RATE,
            "cgt_amount": 0.0,
            "is_exempt": True,
            "exemption_type": exemption_type,
            "exemption_description": CGT_EXEMPT_DISPOSALS.get(exemption_type, "Custom exemption"),
            "computed_at": datetime.now().isoformat(),
        }

    def _no_gain_result(self, proceeds: float, total_allowable: float, chargeable_gain: float) -> Dict[str, Any]:
        """Return a result when there is no chargeable gain."""
        return {
            "tax_type": "CGT",
            "disposal_proceeds": _money_round(proceeds),
            "total_allowable_costs": _money_round(total_allowable),
            "chargeable_gain": _money_round(chargeable_gain),
            "cgt_rate": CGT_RATE,
            "cgt_amount": 0.0,
            "capital_loss": _money_round(abs(chargeable_gain)) if chargeable_gain < 0 else 0.0,
            "is_loss": chargeable_gain < 0,
            "note": "No CGT payable — capital loss may offset future capital gains" if chargeable_gain < 0 else "No chargeable gain",
            "computed_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
