# Author: Quadri Atharu
"""Role definitions with permissions per role for HAQLY ERP internal controls.

Implements role-based access control aligned with Nigerian financial
regulations and CAMA 2020 requirements.
"""

from __future__ import annotations

from enum import Enum
from typing import Any, Dict, FrozenSet, List, Optional, Set

from ..core.logging import get_logger

logger = get_logger(__name__)


class Permission(str, Enum):
    CREATE_JOURNAL = "CREATE_JOURNAL"
    REVIEW_JOURNAL = "REVIEW_JOURNAL"
    APPROVE_JOURNAL = "APPROVE_JOURNAL"
    POST_JOURNAL = "POST_JOURNAL"
    VOID_JOURNAL = "VOID_JOURNAL"
    REVERSE_JOURNAL = "REVERSE_JOURNAL"
    VIEW_LEDGER = "VIEW_LEDGER"
    POST_LEDGER = "POST_LEDGER"
    COMPUTE_TAX = "COMPUTE_TAX"
    FILE_TAX_RETURN = "FILE_TAX_RETURN"
    GENERATE_REPORT = "GENERATE_REPORT"
    APPROVE_REPORT = "APPROVE_REPORT"
    MANAGE_USERS = "MANAGE_USERS"
    MANAGE_ROLES = "MANAGE_ROLES"
    ACCESS_ADMIN = "ACCESS_ADMIN"
    VIEW_AUDIT_TRAIL = "VIEW_AUDIT_TRAIL"
    EXPORT_DATA = "EXPORT_DATA"
    IMPORT_DATA = "IMPORT_DATA"
    MANAGE_CHART_OF_ACCOUNTS = "MANAGE_CHART_OF_ACCOUNTS"
    PERIOD_CLOSE = "PERIOD_CLOSE"
    YEAR_END_CLOSE = "YEAR_END_CLOSE"
    MANAGE_BUDGET = "MANAGE_BUDGET"
    APPROVE_BUDGET = "APPROVE_BUDGET"
    TREASURY_OPS = "TREASURY_OPS"
    BANK_RECONCILIATION = "BANK_RECONCILIATION"


class RoleName(str, Enum):
    ACCOUNTANT = "ACCOUNTANT"
    SENIOR_ACCOUNTANT = "SENIOR_ACCOUNTANT"
    TAX_SPECIALIST = "TAX_SPECIALIST"
    AUDITOR = "AUDITOR"
    FINANCE_MANAGER = "FINANCE_MANAGER"
    CFO = "CFO"
    SYSTEM_ADMIN = "SYSTEM_ADMIN"
    READ_ONLY = "READ_ONLY"


ROLE_PERMISSIONS: Dict[RoleName, FrozenSet[Permission]] = {
    RoleName.ACCOUNTANT: frozenset({
        Permission.CREATE_JOURNAL,
        Permission.VIEW_LEDGER,
        Permission.COMPUTE_TAX,
        Permission.GENERATE_REPORT,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
        Permission.MANAGE_BUDGET,
    }),
    RoleName.SENIOR_ACCOUNTANT: frozenset({
        Permission.CREATE_JOURNAL,
        Permission.REVIEW_JOURNAL,
        Permission.VIEW_LEDGER,
        Permission.POST_LEDGER,
        Permission.COMPUTE_TAX,
        Permission.GENERATE_REPORT,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
        Permission.IMPORT_DATA,
        Permission.MANAGE_BUDGET,
        Permission.MANAGE_CHART_OF_ACCOUNTS,
        Permission.BANK_RECONCILIATION,
    }),
    RoleName.TAX_SPECIALIST: frozenset({
        Permission.COMPUTE_TAX,
        Permission.FILE_TAX_RETURN,
        Permission.GENERATE_REPORT,
        Permission.VIEW_LEDGER,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
    }),
    RoleName.AUDITOR: frozenset({
        Permission.VIEW_LEDGER,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.GENERATE_REPORT,
        Permission.EXPORT_DATA,
        Permission.REVIEW_JOURNAL,
    }),
    RoleName.FINANCE_MANAGER: frozenset({
        Permission.REVIEW_JOURNAL,
        Permission.APPROVE_JOURNAL,
        Permission.VOID_JOURNAL,
        Permission.REVERSE_JOURNAL,
        Permission.VIEW_LEDGER,
        Permission.POST_LEDGER,
        Permission.COMPUTE_TAX,
        Permission.GENERATE_REPORT,
        Permission.APPROVE_REPORT,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
        Permission.IMPORT_DATA,
        Permission.MANAGE_CHART_OF_ACCOUNTS,
        Permission.PERIOD_CLOSE,
        Permission.MANAGE_BUDGET,
        Permission.APPROVE_BUDGET,
        Permission.TREASURY_OPS,
        Permission.BANK_RECONCILIATION,
    }),
    RoleName.CFO: frozenset({
        Permission.CREATE_JOURNAL,
        Permission.REVIEW_JOURNAL,
        Permission.APPROVE_JOURNAL,
        Permission.POST_JOURNAL,
        Permission.VOID_JOURNAL,
        Permission.REVERSE_JOURNAL,
        Permission.VIEW_LEDGER,
        Permission.POST_LEDGER,
        Permission.COMPUTE_TAX,
        Permission.FILE_TAX_RETURN,
        Permission.GENERATE_REPORT,
        Permission.APPROVE_REPORT,
        Permission.MANAGE_USERS,
        Permission.MANAGE_ROLES,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
        Permission.IMPORT_DATA,
        Permission.MANAGE_CHART_OF_ACCOUNTS,
        Permission.PERIOD_CLOSE,
        Permission.YEAR_END_CLOSE,
        Permission.MANAGE_BUDGET,
        Permission.APPROVE_BUDGET,
        Permission.TREASURY_OPS,
        Permission.BANK_RECONCILIATION,
    }),
    RoleName.SYSTEM_ADMIN: frozenset({
        Permission.MANAGE_USERS,
        Permission.MANAGE_ROLES,
        Permission.ACCESS_ADMIN,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.EXPORT_DATA,
        Permission.IMPORT_DATA,
        Permission.YEAR_END_CLOSE,
    }),
    RoleName.READ_ONLY: frozenset({
        Permission.VIEW_LEDGER,
        Permission.VIEW_AUDIT_TRAIL,
        Permission.GENERATE_REPORT,
    }),
}


class RoleDefinition:
    """Complete role definition with permissions and metadata for Nigerian ERP."""

    def __init__(self, role_name: RoleName, department: str = "", description: str = "") -> None:
        self.role_name = role_name
        self.department = department
        self.description = description or self._default_description(role_name)
        self.permissions = set(ROLE_PERMISSIONS.get(role_name, frozenset()))

    def has_permission(self, permission: Permission) -> bool:
        return permission in self.permissions

    def add_permission(self, permission: Permission) -> None:
        self.permissions.add(permission)

    def remove_permission(self, permission: Permission) -> None:
        self.permissions.discard(permission)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "role_name": self.role_name.value,
            "department": self.department,
            "description": self.description,
            "permissions": sorted(p.value for p in self.permissions),
            "permission_count": len(self.permissions),
        }

    @staticmethod
    def _default_description(role_name: RoleName) -> str:
        descriptions: Dict[RoleName, str] = {
            RoleName.ACCOUNTANT: "Entry-level accounting staff — journal creation and basic reporting",
            RoleName.SENIOR_ACCOUNTANT: "Senior accounting staff — review, ledger posting, and bank reconciliation",
            RoleName.TAX_SPECIALIST: "Tax computation and filing specialist — Nigerian tax compliance",
            RoleName.AUDITOR: "Internal/external auditor — read-only access with review capabilities",
            RoleName.FINANCE_MANAGER: "Finance manager — approval authority, period close, and treasury",
            RoleName.CFO: "Chief Financial Officer — full financial control and oversight",
            RoleName.SYSTEM_ADMIN: "System administrator — user/role management and system config",
            RoleName.READ_ONLY: "Read-only access — view ledger, audit trail, and reports",
        }
        return descriptions.get(role_name, "Custom role")


def get_permissions(role_name: RoleName) -> Set[Permission]:
    """Return the set of permissions assigned to a role."""
    return set(ROLE_PERMISSIONS.get(role_name, frozenset()))


def can_perform_action(role_name: RoleName, permission: Permission) -> bool:
    """Check if a role has a specific permission."""
    return permission in ROLE_PERMISSIONS.get(role_name, frozenset())


def get_roles_with_permission(permission: Permission) -> List[RoleName]:
    """Return all roles that have a given permission."""
    return [role for role, perms in ROLE_PERMISSIONS.items() if permission in perms]


def get_approval_roles(amount: float, thresholds: Optional[Dict[str, float]] = None) -> List[RoleName]:
    """Determine required approval roles based on transaction amount thresholds."""
    if thresholds is None:
        thresholds = {
            "senior_accountant_max": 500_000,
            "finance_manager_max": 5_000_000,
            "cfo_max": 50_000_000,
        }

    if amount <= thresholds.get("senior_accountant_max", 500_000):
        return [RoleName.SENIOR_ACCOUNTANT]
    elif amount <= thresholds.get("finance_manager_max", 5_000_000):
        return [RoleName.SENIOR_ACCOUNTANT, RoleName.FINANCE_MANAGER]
    elif amount <= thresholds.get("cfo_max", 50_000_000):
        return [RoleName.SENIOR_ACCOUNTANT, RoleName.FINANCE_MANAGER, RoleName.CFO]
    else:
        return [RoleName.SENIOR_ACCOUNTANT, RoleName.FINANCE_MANAGER, RoleName.CFO, RoleName.CFO]
