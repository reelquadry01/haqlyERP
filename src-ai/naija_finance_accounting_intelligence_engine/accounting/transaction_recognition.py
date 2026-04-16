# Author: Quadri Atharu
"""IFRS-aligned transaction recognition rules for Nigerian entities."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import AccountingError, IFRSError
from ..core.logging import get_logger
from decimal import Decimal, ROUND_HALF_UP


def _money_round(value) -> Decimal:
    if isinstance(value, Decimal):
        return value.quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)
    return Decimal(str(value)).quantize(Decimal('0.01'), rounding=ROUND_HALF_UP)


logger = get_logger(__name__)

REVENUE_CRITERIA = ["performance_obligation_satisfied", "amount_can_be_measured", "probable_economic_benefit", "cost_can_be_measured"]
EXPENSE_CRITERIA = ["obligation_exists", "amount_can_be_measured", "probable_outflow", "reliable_measurement"]
ASSET_CRITERIA = ["control_exists", "future_economic_benefit", "cost_can_be_measured"]
LIABILITY_CRITERIA = ["present_obligation_exists", "outflow_probable", "amount_can_be_measured"]


class TransactionRecognition:
    """IFRS-aligned transaction recognition engine.

    Implements recognition criteria per IFRS 15 (Revenue), IAS 37 (Provisions),
    IAS 16 (PPE), IFRS 9 (Financial Instruments), and the Framework.
    """

    def recognize_transaction(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Evaluate and classify a transaction for IFRS-aligned recognition."""
        txn_type = transaction.get("transaction_type", "").upper()
        amount = float(transaction.get("amount", 0))
        if amount <= 0:
            raise AccountingError("Transaction amount must be positive for recognition")

        recognizer_map = {
            "SALES": self._recognize_revenue,
            "INVOICE": self._recognize_revenue,
            "CASH_RECEIPT": self._recognize_revenue,
            "PURCHASE": self._recognize_expense,
            "BILL": self._recognize_expense,
            "EXPENSE": self._recognize_expense,
            "DEPRECIATION": self._recognize_expense,
            "SALARY": self._recognize_expense,
            "TRANSFER": self._recognize_transfer,
            "JOURNAL": self._recognize_adjustment,
            "ADJUSTMENT": self._recognize_adjustment,
            "CREDIT_NOTE": self._recognize_revenue_reversal,
            "DEBIT_NOTE": self._recognize_expense_reversal,
        }

        recognizer = recognizer_map.get(txn_type)
        if recognizer is None:
            raise AccountingError(f"Unknown transaction type for recognition: {txn_type}")

        result = recognizer(transaction)
        result["transaction_type"] = txn_type
        result["recognized_at"] = datetime.now().isoformat()
        logger.info("transaction_recognized", transaction_type=txn_type, recognized=result.get("recognized", False))
        return result

    def _recognize_revenue(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Apply IFRS 15 five-step revenue recognition model."""
        amount = float(transaction.get("amount", 0))
        description = transaction.get("description", "")
        contract_terms = transaction.get("contract_terms", {})
        performance_obligations = contract_terms.get("performance_obligations", ["transfer_of_goods"])
        has_multiple_obligations = len(performance_obligations) > 1

        criteria_met: Dict[str, bool] = {
            "identified_contract": bool(contract_terms or transaction.get("counterparty")),
            "identified_performance_obligations": len(performance_obligations) > 0,
            "determined_transaction_price": amount > 0,
            "allocated_transaction_price": amount > 0,
            "satisfied_performance_obligation": not transaction.get("deferred", False),
        }

        all_met = all(criteria_met.values())
        allocation = self._allocate_transaction_price(amount, performance_obligations) if has_multiple_obligations else {"default": amount}

        if not all_met:
            return {
                "recognized": False,
                "standard": "IFRS 15",
                "criteria": criteria_met,
                "reason": self._unmet_criteria_reason(criteria_met, "IFRS 15"),
                "suggested_account": "1400",
                "suggested_treatment": "defer_revenue_until_obligation_satisfied",
                "allocation": allocation,
            }

        return {
            "recognized": True,
            "standard": "IFRS 15",
            "criteria": criteria_met,
            "suggested_debit_account": transaction.get("debit_account_code", "1100"),
            "suggested_credit_account": "4000",
            "recognition_point": "point_in_time" if "transfer_of_goods" in performance_obligations else "over_time",
            "allocation": allocation,
        }

    def _recognize_expense(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Apply IFRS expense recognition criteria (Framework + IAS 37 for provisions)."""
        amount = float(transaction.get("amount", 0))
        is_provision = transaction.get("is_provision", False)

        if is_provision:
            return self._recognize_provision(transaction)

        criteria_met: Dict[str, bool] = {
            "obligation_exists": True,
            "amount_can_be_measured": amount > 0,
            "probable_outflow": not transaction.get("contingent", False),
            "reliable_measurement": amount > 0,
        }

        all_met = all(criteria_met.values())

        if not all_met:
            return {
                "recognized": False,
                "standard": "IAS 1 / Framework",
                "criteria": criteria_met,
                "reason": self._unmet_criteria_reason(criteria_met, "Framework"),
                "suggested_treatment": "disclose_as_contingent_liability",
            }

        return {
            "recognized": True,
            "standard": "IAS 1 / Framework",
            "criteria": criteria_met,
            "suggested_debit_account": transaction.get("debit_account_code", "5000"),
            "suggested_credit_account": transaction.get("credit_account_code", "2100"),
            "recognition_point": "period_incurrred",
        }

    def _recognize_provision(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Apply IAS 37 provision recognition criteria."""
        amount = float(transaction.get("amount", 0))

        criteria_met: Dict[str, bool] = {
            "present_obligation": transaction.get("present_obligation", True),
            "probable_outflow": transaction.get("probable_outflow", True),
            "reliable_estimate": amount > 0 or transaction.get("reliable_estimate", False),
        }

        all_met = all(criteria_met.values())

        if not all_met:
            return {
                "recognized": False,
                "standard": "IAS 37",
                "criteria": criteria_met,
                "reason": self._unmet_criteria_reason(criteria_met, "IAS 37"),
                "suggested_treatment": "disclose_as_contingent_liability_if_possible",
            }

        return {
            "recognized": True,
            "standard": "IAS 37",
            "criteria": criteria_met,
            "suggested_debit_account": transaction.get("debit_account_code", "5900"),
            "suggested_credit_account": "2900",
            "recognition_point": "obligating_event",
        }

    def _recognize_transfer(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Transfers between accounts — no P&L impact, always recognizable."""
        return {
            "recognized": True,
            "standard": "Framework",
            "criteria": {"valid_transfer": True},
            "suggested_debit_account": transaction.get("debit_account_code", "1010"),
            "suggested_credit_account": transaction.get("credit_account_code", "1020"),
            "recognition_point": "immediate",
        }

    def _recognize_adjustment(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Adjusting entries for accruals, prepayments, provisions."""
        adjustment_type = transaction.get("adjustment_type", "accrual").lower()
        amount = float(transaction.get("amount", 0))

        adjustment_map = {
            "accrual": {"debit": "5200", "credit": "2100", "standard": "IAS 1 / Accrual Basis"},
            "prepayment": {"debit": "1300", "credit": "5000", "standard": "IAS 1 / Accrual Basis"},
            "provision": {"debit": "5900", "credit": "2900", "standard": "IAS 37"},
            "depreciation": {"debit": "5200", "credit": "1610", "standard": "IAS 16"},
            "impairment": {"debit": "5900", "credit": "1610", "standard": "IAS 36"},
            "correction": {"debit": transaction.get("debit_account_code", "9900"), "credit": transaction.get("credit_account_code", "9900"), "standard": "IAS 8"},
            "revaluation": {"debit": "1600", "credit": "2300", "standard": "IAS 16"},
        }

        adj = adjustment_map.get(adjustment_type)
        if adj is None:
            raise IFRSError(f"Unknown adjustment type: {adjustment_type}", standard="IAS 1")

        return {
            "recognized": True,
            "standard": adj["standard"],
            "criteria": {"adjustment_required": True, "amount_measurable": amount > 0},
            "suggested_debit_account": adj["debit"],
            "suggested_credit_account": adj["credit"],
            "recognition_point": "period_end",
            "adjustment_type": adjustment_type,
        }

    def _recognize_revenue_reversal(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Credit note — partial or full revenue reversal under IFRS 15."""
        amount = float(transaction.get("amount", 0))
        return {
            "recognized": True,
            "standard": "IFRS 15",
            "criteria": {"reversal_of_revenue": True, "amount_measurable": amount > 0},
            "suggested_debit_account": "4000",
            "suggested_credit_account": "1100",
            "recognition_point": "immediate",
            "adjustment_type": "revenue_reversal",
        }

    def _recognize_expense_reversal(self, transaction: Dict[str, Any]) -> Dict[str, Any]:
        """Debit note — partial or full expense reversal."""
        amount = float(transaction.get("amount", 0))
        return {
            "recognized": True,
            "standard": "Framework",
            "criteria": {"reversal_of_expense": True, "amount_measurable": amount > 0},
            "suggested_debit_account": "2100",
            "suggested_credit_account": "5000",
            "recognition_point": "immediate",
            "adjustment_type": "expense_reversal",
        }

    @staticmethod
    def _allocate_transaction_price(total_amount: float, obligations: List[str]) -> Dict[str, float]:
        """Allocate transaction price across multiple performance obligations (equal split default)."""
        if not obligations:
            return {"unallocated": total_amount}
        per_obligation = _money_round(total_amount / len(obligations))
        remainder = _money_round(total_amount - per_obligation * len(obligations))
        allocation: Dict[str, float] = {}
        for i, obl in enumerate(obligations):
            allocation[obl] = per_obligation + (remainder if i == 0 else 0.0)
        return allocation

    @staticmethod
    def _unmet_criteria_reason(criteria: Dict[str, bool], standard: str) -> str:
        """Return a human-readable explanation of unmet criteria."""
        unmet = [k for k, v in criteria.items() if not v]
        return f"{standard}: Unmet criteria — {', '.join(unmet)}"

    def evaluate_lease_classification(self, lease: Dict[str, Any]) -> Dict[str, Any]:
        """Classify lease as operating or finance under IFRS 16."""
        lease_term = int(lease.get("lease_term_months", 0))
        asset_useful_life = int(lease.get("asset_useful_life_months", 0))
        present_value = float(lease.get("present_value_of_payments", 0))
        fair_value = float(lease.get("fair_value_of_asset", 0))
        purchase_option = lease.get("contains_purchase_option", False)
        specialized = lease.get("asset_is_specialized", False)

        criteria: Dict[str, bool] = {
            "ownership_transfers": lease.get("ownership_transfers", False),
            "purchase_option_reasonably_certain": purchase_option and lease.get("purchase_option_exercisable", False),
            "lease_term_majority_of_life": lease_term >= asset_useful_life * 0.75 if asset_useful_life > 0 else False,
            "present_value_substantially_all": fair_value > 0 and present_value / fair_value >= 0.90,
            "specialized_asset": specialized,
        }

        is_finance = any(criteria.values())
        classification = "finance_lease" if is_finance else "operating_lease"

        return {
            "classification": classification,
            "standard": "IFRS 16",
            "criteria": criteria,
            "right_of_use_asset_account": "1600" if is_finance else "5600",
            "lease_liability_account": "2600" if is_finance else "2100",
            "depreciation_account": "1610" if is_finance else None,
            "interest_account": "5700" if is_finance else None,
        }

    def assess_impairment(self, asset: Dict[str, Any]) -> Dict[str, Any]:
        """Assess asset impairment under IAS 36."""
        carrying_amount = float(asset.get("carrying_amount", 0))
        recoverable_amount = float(asset.get("recoverable_amount", 0))
        value_in_use = float(asset.get("value_in_use", 0))
        fair_value_less_costs = float(asset.get("fair_value_less_costs_of_disposal", 0))

        if recoverable_amount == 0 and (value_in_use > 0 or fair_value_less_costs > 0):
            recoverable_amount = max(value_in_use, fair_value_less_costs)

        impairment_loss = max(carrying_amount - recoverable_amount, 0)
        impaired = impairment_loss > 0

        return {
            "standard": "IAS 36",
            "carrying_amount": carrying_amount,
            "recoverable_amount": recoverable_amount,
            "value_in_use": value_in_use,
            "fair_value_less_costs": fair_value_less_costs,
            "impairment_loss": _money_round(impairment_loss),
            "impaired": impaired,
            "suggested_debit_account": "5900" if impaired else None,
            "suggested_credit_account": "1610" if impaired else None,
            "reversal_permitted": asset.get("is_ppe_not_goodwill", True),
        }

    def health_check(self) -> bool:
        return True
