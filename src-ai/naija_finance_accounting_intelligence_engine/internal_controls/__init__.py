"""Internal controls module for SoD enforcement, role management, and audit trails.

Author: Quadri Atharu

Provides segregation of duties enforcement, role-based access control,
audit trail generation, and user accountability tracking for
HAQLY ERP financial operations.
"""

from .segregation_of_duties import check_sod_violation, enforce_three_way_match, validate_approval_chain
from .roles import RoleDefinition, get_permissions, can_perform_action
from .audit_trail import record_action, generate_audit_trail, detect_suspicious_patterns, generate_audit_summary
from .user_accountability import track_user_action, compute_user_activity_summary, flag_dormant_controls

__all__ = [
    "check_sod_violation", "enforce_three_way_match", "validate_approval_chain",
    "RoleDefinition", "get_permissions", "can_perform_action",
    "record_action", "generate_audit_trail", "detect_suspicious_patterns", "generate_audit_summary",
    "track_user_action", "compute_user_activity_summary", "flag_dormant_controls",
]
