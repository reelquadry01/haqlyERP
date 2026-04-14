# Author: Quadri Atharu
"""Segregation of Duties (SoD) enforcement for Nigerian financial controls.

Provides role-based duty separation, three-way match enforcement,
and approval chain validation per Nigerian SOX/CAMA requirements.
"""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional, Set, Tuple

from ..core.exceptions import ValidationError
from ..core.logging import get_logger

logger = get_logger(__name__)


class Role(str, Enum):
    ENTRY_CREATOR = "ENTRY_CREATOR"
    REVIEWER = "REVIEWER"
    APPROVER = "APPROVER"
    POSTER = "POSTER"


SOD_CONFLICT_PAIRS: Dict[Role, Set[Role]] = {
    Role.ENTRY_CREATOR: {Role.POSTER},
    Role.REVIEWER: {Role.ENTRY_CREATOR},
    Role.APPROVER: {Role.ENTRY_CREATOR, Role.POSTER},
    Role.POSTER: {Role.ENTRY_CREATOR},
}

THREE_WAY_MATCH_TOLERANCE_PCT = 0.02


def check_sod_violation(
    user_id: str,
    current_role: Role,
    assigned_roles: List[Role],
    transaction_id: Optional[str] = None,
) -> Dict[str, Any]:
    """Check if a user's role assignment violates segregation of duties.

    Returns a dict with violation details if any conflicts are found.
    """
    conflicts: List[Dict[str, Any]] = []
    all_roles = set(assigned_roles) | {current_role}

    for role in all_roles:
        conflicting = SOD_CONFLICT_PAIRS.get(role, set()) & all_roles
        for conflict_role in conflicting:
            conflicts.append({
                "role_1": role.value,
                "role_2": conflict_role.value,
                "severity": "high" if role == Role.APPROVER and conflict_role == Role.ENTRY_CREATOR else "medium",
                "description": f"User cannot hold both {role.value} and {conflict_role.value} roles",
            })

    has_violation = len(conflicts) > 0

    result: Dict[str, Any] = {
        "user_id": user_id,
        "transaction_id": transaction_id,
        "current_role": current_role.value,
        "assigned_roles": [r.value for r in assigned_roles],
        "all_roles": [r.value for r in all_roles],
        "has_violation": has_violation,
        "violations": conflicts,
        "violation_count": len(conflicts),
        "checked_at": datetime.now().isoformat(),
    }

    if has_violation:
        logger.warning("sod_violation_detected", user_id=user_id, violation_count=len(conflicts))

    return result


def enforce_three_way_match(
    purchase_order: Dict[str, Any],
    goods_receipt: Dict[str, Any],
    invoice: Dict[str, Any],
    tolerance_pct: float = THREE_WAY_MATCH_TOLERANCE_PCT,
) -> Dict[str, Any]:
    """Enforce three-way match: PO vs GRN vs Invoice.

    Validates that quantities and amounts match across all three documents
    within a configurable tolerance percentage.
    """
    po_amount = float(purchase_order.get("amount", 0))
    po_quantity = float(purchase_order.get("quantity", 0))
    po_vendor = purchase_order.get("vendor_id", "")

    grn_amount = float(goods_receipt.get("amount", 0))
    grn_quantity = float(goods_receipt.get("quantity", 0))
    grn_vendor = goods_receipt.get("vendor_id", "")

    inv_amount = float(invoice.get("amount", 0))
    inv_quantity = float(invoice.get("quantity", 0))
    inv_vendor = invoice.get("vendor_id", "")

    discrepancies: List[Dict[str, Any]] = []

    amount_match_po_grn = _within_tolerance(po_amount, grn_amount, tolerance_pct)
    amount_match_grn_inv = _within_tolerance(grn_amount, inv_amount, tolerance_pct)
    amount_match_po_inv = _within_tolerance(po_amount, inv_amount, tolerance_pct)

    if not amount_match_po_grn:
        discrepancies.append({
            "check": "PO vs GRN Amount",
            "po_amount": po_amount,
            "grn_amount": grn_amount,
            "difference": round(po_amount - grn_amount, 2),
        })
    if not amount_match_grn_inv:
        discrepancies.append({
            "check": "GRN vs Invoice Amount",
            "grn_amount": grn_amount,
            "invoice_amount": inv_amount,
            "difference": round(grn_amount - inv_amount, 2),
        })
    if not amount_match_po_inv:
        discrepancies.append({
            "check": "PO vs Invoice Amount",
            "po_amount": po_amount,
            "invoice_amount": inv_amount,
            "difference": round(po_amount - inv_amount, 2),
        })

    qty_match_po_grn = _within_tolerance(po_quantity, grn_quantity, tolerance_pct)
    qty_match_grn_inv = _within_tolerance(grn_quantity, inv_quantity, tolerance_pct)
    qty_match_po_inv = _within_tolerance(po_quantity, inv_quantity, tolerance_pct)

    if not qty_match_po_grn:
        discrepancies.append({
            "check": "PO vs GRN Quantity",
            "po_quantity": po_quantity,
            "grn_quantity": grn_quantity,
            "difference": round(po_quantity - grn_quantity, 2),
        })
    if not qty_match_grn_inv:
        discrepancies.append({
            "check": "GRN vs Invoice Quantity",
            "grn_quantity": grn_quantity,
            "invoice_quantity": inv_quantity,
            "difference": round(grn_quantity - inv_quantity, 2),
        })
    if not qty_match_po_inv:
        discrepancies.append({
            "check": "PO vs Invoice Quantity",
            "po_quantity": po_quantity,
            "invoice_quantity": inv_quantity,
            "difference": round(po_quantity - inv_quantity, 2),
        })

    vendor_consistent = (po_vendor == grn_vendor == inv_vendor) if po_vendor and grn_vendor and inv_vendor else True
    if not vendor_consistent:
        discrepancies.append({
            "check": "Vendor Consistency",
            "po_vendor": po_vendor,
            "grn_vendor": grn_vendor,
            "invoice_vendor": inv_vendor,
            "difference": "Vendors do not match across documents",
        })

    all_matched = len(discrepancies) == 0

    result: Dict[str, Any] = {
        "match_status": "MATCHED" if all_matched else "DISCREPANCY",
        "tolerance_pct": tolerance_pct,
        "purchase_order": {"amount": po_amount, "quantity": po_quantity, "vendor_id": po_vendor, "reference": purchase_order.get("reference")},
        "goods_receipt": {"amount": grn_amount, "quantity": grn_quantity, "vendor_id": grn_vendor, "reference": goods_receipt.get("reference")},
        "invoice": {"amount": inv_amount, "quantity": inv_quantity, "vendor_id": inv_vendor, "reference": invoice.get("reference")},
        "discrepancies": discrepancies,
        "discrepancy_count": len(discrepancies),
        "vendor_consistent": vendor_consistent,
        "amount_checks": {
            "po_vs_grn": "PASS" if amount_match_po_grn else "FAIL",
            "grn_vs_invoice": "PASS" if amount_match_grn_inv else "FAIL",
            "po_vs_invoice": "PASS" if amount_match_po_inv else "FAIL",
        },
        "quantity_checks": {
            "po_vs_grn": "PASS" if qty_match_po_grn else "FAIL",
            "grn_vs_invoice": "PASS" if qty_match_grn_inv else "FAIL",
            "po_vs_invoice": "PASS" if qty_match_po_inv else "FAIL",
        },
        "checked_at": datetime.now().isoformat(),
    }

    if not all_matched:
        logger.warning("three_way_match_discrepancy", discrepancy_count=len(discrepancies))

    return result


def validate_approval_chain(
    entry_id: str,
    approver_role: Role,
    entry_status: str,
    previous_approvers: List[Dict[str, Any]],
    amount: float = 0,
    amount_thresholds: Optional[Dict[str, float]] = None,
) -> Dict[str, Any]:
    """Validate that the approval chain follows proper sequence and SoD rules.

    Enforces: Creator -> Reviewer -> Approver -> Poster chain.
    Also enforces amount-based approval thresholds.
    """
    if amount_thresholds is None:
        amount_thresholds = {
            "reviewer_max": 1_000_000,
            "approver_max": 10_000_000,
            "board_required_above": 50_000_000,
        }

    chain_sequence = [Role.ENTRY_CREATOR, Role.REVIEWER, Role.APPROVER, Role.POSTER]
    chain_errors: List[str] = []

    if approver_role not in chain_sequence:
        chain_errors.append(f"Invalid role in approval chain: {approver_role.value}")

    prev_roles = {Role(a["role"]) for a in previous_approvers if a.get("role")}

    if approver_role == Role.REVIEWER:
        if Role.ENTRY_CREATOR not in prev_roles and entry_status not in ("DRAFT", "SUBMITTED"):
            chain_errors.append("Entry must be created/submitted before review")
        for pa in previous_approvers:
            if pa.get("role") == Role.REVIEWER.value and pa.get("user_id") == previous_approvers[-1].get("user_id"):
                chain_errors.append("Same user cannot review twice")

    if approver_role == Role.APPROVER:
        if Role.REVIEWER not in prev_roles:
            chain_errors.append("Entry must be reviewed before approval")
        for pa in previous_approvers:
            if pa.get("role") == Role.ENTRY_CREATOR.value:
                if pa.get("user_id") in {a.get("user_id") for a in previous_approvers if a.get("role") == approver_role.value}:
                    chain_errors.append("Creator cannot approve their own entry (SoD violation)")

    if approver_role == Role.POSTER:
        if Role.APPROVER not in prev_roles:
            chain_errors.append("Entry must be approved before posting")
        if entry_status != "APPROVED":
            chain_errors.append(f"Entry status must be APPROVED for posting, got: {entry_status}")

    if amount > amount_thresholds.get("board_required_above", 50_000_000):
        has_board = any(a.get("role") == "BOARD" or a.get("is_board_approval") for a in previous_approvers)
        if not has_board:
            chain_errors.append(f"Amount {amount} exceeds board approval threshold ({amount_thresholds['board_required_above']})")

    is_valid = len(chain_errors) == 0

    result: Dict[str, Any] = {
        "entry_id": entry_id,
        "approver_role": approver_role.value,
        "entry_status": entry_status,
        "amount": amount,
        "previous_approvers": previous_approvers,
        "is_valid": is_valid,
        "chain_errors": chain_errors,
        "error_count": len(chain_errors),
        "amount_thresholds": amount_thresholds,
        "validated_at": datetime.now().isoformat(),
    }

    if not is_valid:
        logger.warning("approval_chain_invalid", entry_id=entry_id, errors=chain_errors)

    return result


def _within_tolerance(value_a: float, value_b: float, tolerance_pct: float) -> bool:
    """Check if two values are within tolerance percentage of each other."""
    if value_a == 0 and value_b == 0:
        return True
    if value_a == 0 or value_b == 0:
        return False
    diff_pct = abs(value_a - value_b) / max(abs(value_a), abs(value_b))
    return diff_pct <= tolerance_pct
