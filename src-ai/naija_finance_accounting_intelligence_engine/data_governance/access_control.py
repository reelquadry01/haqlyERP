# Author: Quadri Atharu
"""Role-based access control for financial data governance."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from ..core.exceptions import EngineError
from ..core.logging import get_logger

logger = get_logger(__name__)

ROLES_PERMISSIONS: Dict[str, Dict[str, List[str]]] = {
    "super_admin": {
        "journal": ["create", "read", "update", "delete", "approve", "post", "void", "reverse"],
        "ledger": ["read", "post", "reconcile", "adjust"],
        "tax": ["compute", "read", "file", "approve", "adjust"],
        "reporting": ["generate", "read", "export", "approve", "publish"],
        "financial_analysis": ["analyze", "read", "export", "forecast"],
        "valuation": ["compute", "read", "export"],
        "risk": ["assess", "read", "mitigate", "approve"],
        "audit": ["read", "generate_trail", "sample", "detect_exceptions", "generate_papers"],
        "budget": ["create", "read", "update", "delete", "approve"],
        "treasury": ["create", "read", "update", "delete", "approve"],
        "settings": ["read", "update"],
        "data_governance": ["read", "update", "administer"],
        "users": ["create", "read", "update", "delete", "manage_roles"],
    },
    "finance_manager": {
        "journal": ["create", "read", "update", "approve", "post", "void", "reverse"],
        "ledger": ["read", "post", "reconcile", "adjust"],
        "tax": ["compute", "read", "file", "approve"],
        "reporting": ["generate", "read", "export", "approve"],
        "financial_analysis": ["analyze", "read", "export", "forecast"],
        "valuation": ["compute", "read", "export"],
        "risk": ["assess", "read", "mitigate"],
        "audit": ["read", "generate_trail", "sample", "detect_exceptions", "generate_papers"],
        "budget": ["create", "read", "update", "approve"],
        "treasury": ["create", "read", "update", "approve"],
        "settings": ["read"],
        "data_governance": ["read"],
        "users": [],
    },
    "accountant": {
        "journal": ["create", "read", "update"],
        "ledger": ["read", "post", "reconcile"],
        "tax": ["compute", "read"],
        "reporting": ["generate", "read", "export"],
        "financial_analysis": ["analyze", "read"],
        "valuation": ["compute", "read"],
        "risk": ["assess", "read"],
        "audit": ["read", "generate_trail"],
        "budget": ["create", "read", "update"],
        "treasury": ["read", "create"],
        "settings": [],
        "data_governance": [],
        "users": [],
    },
    "auditor": {
        "journal": ["read"],
        "ledger": ["read"],
        "tax": ["read"],
        "reporting": ["read", "export"],
        "financial_analysis": ["read", "analyze"],
        "valuation": ["read"],
        "risk": ["assess", "read"],
        "audit": ["read", "generate_trail", "sample", "detect_exceptions", "generate_papers"],
        "budget": ["read"],
        "treasury": ["read"],
        "settings": [],
        "data_governance": ["read"],
        "users": [],
    },
    "tax_specialist": {
        "journal": ["read"],
        "ledger": ["read"],
        "tax": ["compute", "read", "file"],
        "reporting": ["generate", "read", "export"],
        "financial_analysis": ["read"],
        "valuation": ["read"],
        "risk": ["assess", "read"],
        "audit": ["read"],
        "budget": ["read"],
        "treasury": ["read"],
        "settings": [],
        "data_governance": [],
        "users": [],
    },
    "viewer": {
        "journal": ["read"],
        "ledger": ["read"],
        "tax": ["read"],
        "reporting": ["read"],
        "financial_analysis": ["read"],
        "valuation": ["read"],
        "risk": ["read"],
        "audit": ["read"],
        "budget": ["read"],
        "treasury": ["read"],
        "settings": [],
        "data_governance": [],
        "users": [],
    },
}

_DEFAULT_ROLE = "viewer"


def check_access(user_role: str, resource: str, action: str) -> bool:
    role_permissions = ROLES_PERMISSIONS.get(user_role.lower().strip())
    if role_permissions is None:
        logger.warning("access_check_unknown_role", user_role=user_role, resource=resource, action=action)
        role_permissions = ROLES_PERMISSIONS.get(_DEFAULT_ROLE, {})

    resource_permissions = role_permissions.get(resource.lower().strip())
    if resource_permissions is None:
        logger.warning("access_check_unknown_resource", user_role=user_role, resource=resource, action=action)
        return False

    has_access = action.lower().strip() in resource_permissions
    if not has_access:
        logger.info(
            "access_denied",
            user_role=user_role,
            resource=resource,
            action=action,
        )
    return has_access


def enforce_rbac(user_role: str, resource: str, required_actions: Optional[List[str]] = None) -> bool:
    role_permissions = ROLES_PERMISSIONS.get(user_role.lower().strip())
    if role_permissions is None:
        role_permissions = ROLES_PERMISSIONS.get(_DEFAULT_ROLE, {})

    resource_permissions = role_permissions.get(resource.lower().strip())
    if resource_permissions is None:
        logger.warning("rbac_enforcement_unknown_resource", user_role=user_role, resource=resource)
        return False

    if required_actions is None:
        has_any = len(resource_permissions) > 0
        if not has_any:
            logger.info("rbac_enforcement_denied", user_role=user_role, resource=resource, reason="no_permissions_on_resource")
        return has_any

    for action in required_actions:
        if action.lower().strip() not in resource_permissions:
            logger.info(
                "rbac_enforcement_denied",
                user_role=user_role,
                resource=resource,
                missing_action=action,
            )
            return False

    return True


def get_user_permissions(user_role: str) -> Dict[str, List[str]]:
    return ROLES_PERMISSIONS.get(user_role.lower().strip(), ROLES_PERMISSIONS.get(_DEFAULT_ROLE, {}))


def get_roles_with_access(resource: str, action: str) -> List[str]:
    matching_roles: List[str] = []
    for role, resources in ROLES_PERMISSIONS.items():
        actions = resources.get(resource.lower().strip(), [])
        if action.lower().strip() in actions:
            matching_roles.append(role)
    return matching_roles
