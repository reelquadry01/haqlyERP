# Author: Quadri Atharu
"""Adjusting entries: accruals, prepayments, provisions, and corrections."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)


class AdjustmentEngine:
    """Engine for creating period-end adjusting entries."""

    def create_accrual(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create an accrual adjusting entry for expenses incurred but not yet recorded."""
        expense_account = data.get("expense_account", "5100")
        liability_account = data.get("liability_account", "2100")
        amount = Decimal(str(data.get("amount", 0)))
        description = data.get("description", "Accrued expense adjustment")
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")

        if amount <= 0:
            raise AccountingError("Accrual amount must be positive")

        lines: List[Dict[str, Any]] = [
            {"account_code": expense_account, "description": description, "debit": _money_round(amount), "credit": 0.0},
            {"account_code": liability_account, "description": description, "debit": 0.0, "credit": _money_round(amount)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Accrual: {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="accrual",
            data=data,
        )
        logger.info("accrual_created", amount=amount, expense_account=expense_account, liability_account=liability_account)
        return entry

    def create_prepayment(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a prepayment adjusting entry for expenses paid in advance."""
        prepayment_account = data.get("prepayment_account", "1300")
        expense_account = data.get("expense_account", "5100")
        total_amount = Decimal(str(data.get("total_amount", 0)))
        periods_total = int(data.get("total_periods", 12))
        periods_elapsed = int(data.get("periods_elapsed", 1))
        description = data.get("description", "Prepayment adjustment")
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")

        if total_amount <= 0:
            raise AccountingError("Prepayment total amount must be positive")
        if periods_total <= 0:
            raise AccountingError("Total periods must be positive")

        per_period = _money_round(total_amount / periods_total)
        recognized_amount = _money_round(per_period * periods_elapsed)
        deferred_amount = _money_round(total_amount - recognized_amount)

        lines: List[Dict[str, Any]] = [
            {"account_code": prepayment_account, "description": f"Deferred: {description}", "debit": _money_round(deferred_amount), "credit": 0.0},
            {"account_code": expense_account, "description": f"Recognized: {description}", "debit": _money_round(recognized_amount), "credit": 0.0},
            {"account_code": expense_account, "description": f"Prepayment reversal: {description}", "debit": 0.0, "credit": _money_round(total_amount)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Prepayment: {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="prepayment",
            data=data,
        )
        entry["details"] = {
            "total_amount": total_amount,
            "per_period": per_period,
            "periods_total": periods_total,
            "periods_elapsed": periods_elapsed,
            "recognized_amount": recognized_amount,
            "deferred_amount": deferred_amount,
        }
        logger.info("prepayment_created", total_amount=total_amount, recognized=recognized_amount, deferred=deferred_amount)
        return entry

    def create_provision(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a provision adjusting entry (IAS 37)."""
        provision_account = data.get("provision_account", "2900")
        expense_account = data.get("expense_account", "5900")
        amount = Decimal(str(data.get("amount", 0)))
        description = data.get("description", "Provision adjustment")
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")
        provision_type = data.get("provision_type", "general")

        if amount <= 0:
            raise AccountingError("Provision amount must be positive")

        lines: List[Dict[str, Any]] = [
            {"account_code": expense_account, "description": description, "debit": _money_round(amount), "credit": 0.0},
            {"account_code": provision_account, "description": description, "debit": 0.0, "credit": _money_round(amount)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Provision ({provision_type}): {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="provision",
            data=data,
        )
        entry["provision_type"] = provision_type
        entry["standard"] = "IAS 37"
        logger.info("provision_created", amount=amount, provision_type=provision_type)
        return entry

    def create_depreciation(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a depreciation adjusting entry (IAS 16)."""
        asset_account = data.get("asset_account", "1610")
        depreciation_account = data.get("depreciation_account", "5200")
        method = data.get("method", "straight_line").lower()
        cost = Decimal(str(data.get("cost", 0)))
        residual_value = Decimal(str(data.get("residual_value", 0)))
        useful_life = int(data.get("useful_life_years", 5))
        accumulated_depreciation = Decimal(str(data.get("accumulated_depreciation", 0)))
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")
        description = data.get("description", "Depreciation adjustment")

        if cost <= 0:
            raise AccountingError("Asset cost must be positive for depreciation")
        if useful_life <= 0:
            raise AccountingError("Useful life must be positive")

        depreciable_amount = cost - residual_value
        remaining_value = depreciable_amount - accumulated_depreciation

        if remaining_value <= 0:
            return {"message": "Asset fully depreciated", "depreciiation_amount": 0, "remaining_value": 0}

        if method == "straight_line":
            dep_amount = _money_round(depreciable_amount / useful_life)
        elif method == "reducing_balance":
            rate = 1 - (residual_value / cost) ** (1 / useful_life) if cost > 0 else 0
            dep_amount = _money_round(remaining_value * rate)
        elif method == "sum_of_years":
            sum_years = useful_life * (useful_life + 1) / 2
            remaining_life = max(useful_life - int(accumulated_depreciation / (depreciable_amount / useful_life)), 1) if depreciable_amount > 0 else 1
            dep_amount = _money_round(depreciable_amount * remaining_life / sum_years)
        else:
            raise AccountingError(f"Unsupported depreciation method: {method}")

        dep_amount = min(dep_amount, remaining_value)

        lines: List[Dict[str, Any]] = [
            {"account_code": depreciation_account, "description": description, "debit": _money_round(dep_amount), "credit": 0.0},
            {"account_code": asset_account, "description": description, "debit": 0.0, "credit": _money_round(dep_amount)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Depreciation ({method}): {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="depreciation",
            data=data,
        )
        entry["details"] = {
            "method": method,
            "cost": cost,
            "residual_value": residual_value,
            "useful_life_years": useful_life,
            "depreciable_amount": depreciable_amount,
            "depreciation_amount": dep_amount,
            "accumulated_depreciation_after": _money_round(accumulated_depreciation + dep_amount),
            "net_book_value_after": _money_round(cost - accumulated_depreciation - dep_amount),
        }
        entry["standard"] = "IAS 16"
        logger.info("depreciation_created", method=method, amount=dep_amount)
        return entry

    def create_correction(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a correction entry for prior period errors (IAS 8)."""
        correct_account = data.get("correct_account", "")
        wrong_account = data.get("wrong_account", "")
        amount = Decimal(str(data.get("amount", 0)))
        description = data.get("description", "Error correction")
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")
        is_prior_period = data.get("is_prior_period", False)

        if amount <= 0:
            raise AccountingError("Correction amount must be positive")
        if not correct_account or not wrong_account:
            raise AccountingError("Both correct_account and wrong_account must be specified")

        correction_debit = correct_account
        correction_credit = wrong_account
        if data.get("should_reverse_side", False):
            correction_debit = wrong_account
            correction_credit = correct_account

        lines: List[Dict[str, Any]] = [
            {"account_code": correction_debit, "description": f"Correction: {description}", "debit": _money_round(amount), "credit": 0.0},
            {"account_code": correction_credit, "description": f"Correction: {description}", "debit": 0.0, "credit": _money_round(amount)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Correction: {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="correction",
            data=data,
        )
        entry["standard"] = "IAS 8"
        entry["is_prior_period"] = is_prior_period
        entry["retrospective_restatement_required"] = is_prior_period
        logger.info("correction_created", amount=amount, correct_account=correct_account, wrong_account=wrong_account)
        return entry

    def create_impairment(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create an impairment adjusting entry (IAS 36)."""
        impairment_account = data.get("impairment_account", "5900")
        asset_account = data.get("asset_account", "1610")
        impairment_loss = Decimal(str(data.get("impairment_loss", 0)))
        description = data.get("description", "Impairment loss")
        period_end = data.get("period_end", datetime.now().isoformat())
        company_id = data.get("company_id", "")

        if impairment_loss <= 0:
            raise AccountingError("Impairment loss must be positive")

        lines: List[Dict[str, Any]] = [
            {"account_code": impairment_account, "description": description, "debit": _money_round(impairment_loss), "credit": 0.0},
            {"account_code": asset_account, "description": description, "debit": 0.0, "credit": _money_round(impairment_loss)},
        ]

        entry = self._build_adjusting_entry(
            company_id=company_id,
            description=f"Impairment: {description}",
            lines=lines,
            period_end=period_end,
            adjustment_type="impairment",
            data=data,
        )
        entry["standard"] = "IAS 36"
        logger.info("impairment_created", amount=impairment_loss)
        return entry

    def generate_all_period_end_adjustments(self, company_id: str, period_end: str, adjustment_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Generate all period-end adjustments in a single batch."""
        results: List[Dict[str, Any]] = []
        total_adjustments = 0

        for adj in adjustment_data:
            adj_type = adj.get("adjustment_type", "").lower()
            adj["company_id"] = company_id
            adj["period_end"] = period_end

            if adj_type == "accrual":
                result = self.create_accrual(adj)
            elif adj_type == "prepayment":
                result = self.create_prepayment(adj)
            elif adj_type == "provision":
                result = self.create_provision(adj)
            elif adj_type == "depreciation":
                result = self.create_depreciation(adj)
            elif adj_type == "correction":
                result = self.create_correction(adj)
            elif adj_type == "impairment":
                result = self.create_impairment(adj)
            else:
                logger.warning("unknown_adjustment_type", adjustment_type=adj_type)
                continue

            results.append(result)
            total_adjustments += 1

        return {
            "company_id": company_id,
            "period_end": period_end,
            "total_adjustments": total_adjustments,
            "adjustments": results,
            "generated_at": datetime.now().isoformat(),
        }

    @staticmethod
    def _build_adjusting_entry(
        company_id: str,
        description: str,
        lines: List[Dict[str, Any]],
        period_end: str,
        adjustment_type: str,
        data: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Build a standardized adjusting entry dict."""
        total_debit = round(sum(l.get("debit", 0) for l in lines), 2)
        total_credit = round(sum(l.get("credit", 0) for l in lines), 2)

        return {
            "id": data.get("id", f"ADJ-{adjustment_type.upper()}-{datetime.now().strftime('%Y%m%d%H%M%S')}"),
            "company_id": company_id,
            "entry_date": period_end,
            "description": description,
            "is_adjusting": True,
            "adjustment_type": adjustment_type,
            "lines": lines,
            "total_debit": total_debit,
            "total_credit": total_credit,
            "status": "DRAFT",
            "reference": data.get("reference"),
            "created_at": datetime.now().isoformat(),
        }

    def health_check(self) -> bool:
        return True
