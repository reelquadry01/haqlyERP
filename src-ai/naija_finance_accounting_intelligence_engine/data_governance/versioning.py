# Author: Quadri Atharu
"""Data versioning — entity version tracking, history, and rollback."""

from __future__ import annotations

import copy
import uuid
from datetime import datetime
from typing import Any, Dict, List, Optional

from ..core.exceptions import EngineError
from ..core.logging import get_logger

logger = get_logger(__name__)

_version_store: Dict[str, List[Dict[str, Any]]] = {}


class VersionEntry:
    """A single version record for an entity."""

    __slots__ = ("version_id", "entity_type", "entity_id", "version_number", "data", "snapshot", "created_at", "created_by", "change_description")

    def __init__(
        self,
        entity_type: str,
        entity_id: str,
        version_number: int,
        data: Dict[str, Any],
        created_by: str = "",
        change_description: str = "",
    ) -> None:
        self.version_id = str(uuid.uuid4())
        self.entity_type = entity_type
        self.entity_id = entity_id
        self.version_number = version_number
        self.data = data
        self.snapshot = copy.deepcopy(data)
        self.created_at = datetime.now().isoformat()
        self.created_by = created_by
        self.change_description = change_description

    def to_dict(self) -> Dict[str, Any]:
        return {
            "version_id": self.version_id,
            "entity_type": self.entity_type,
            "entity_id": self.entity_id,
            "version_number": self.version_number,
            "data": self.snapshot,
            "created_at": self.created_at,
            "created_by": self.created_by,
            "change_description": self.change_description,
        }


def _storage_key(entity_type: str, entity_id: str) -> str:
    return f"{entity_type.lower().strip()}:{entity_id.lower().strip()}"


def create_version(
    entity_type: str,
    entity_id: str,
    data: Dict[str, Any],
    created_by: str = "",
    change_description: str = "",
) -> Dict[str, Any]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.setdefault(key, [])
    version_number = len(versions) + 1

    entry = VersionEntry(
        entity_type=entity_type,
        entity_id=entity_id,
        version_number=version_number,
        data=data,
        created_by=created_by,
        change_description=change_description,
    )

    versions.append(entry.to_dict())
    logger.info(
        "version_created",
        entity_type=entity_type,
        entity_id=entity_id,
        version_number=version_number,
    )

    return entry.to_dict()


def get_version_history(
    entity_type: str,
    entity_id: str,
    limit: Optional[int] = None,
    offset: int = 0,
) -> List[Dict[str, Any]]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.get(key, [])

    if offset:
        versions = versions[offset:]
    if limit:
        versions = versions[:limit]

    return list(versions)


def rollback(
    entity_type: str,
    entity_id: str,
    version_number: int,
    rolled_back_by: str = "",
) -> Dict[str, Any]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.get(key, [])

    if not versions:
        raise EngineError(
            f"No version history found for {entity_type}/{entity_id}",
            code="VERSION_NOT_FOUND",
        )

    target = None
    for v in versions:
        if v["version_number"] == version_number:
            target = v
            break

    if target is None:
        raise EngineError(
            f"Version {version_number} not found for {entity_type}/{entity_id}",
            code="VERSION_NOT_FOUND",
            details={"available_versions": [v["version_number"] for v in versions]},
        )

    rollback_data = copy.deepcopy(target["snapshot"])

    rollback_version = create_version(
        entity_type=entity_type,
        entity_id=entity_id,
        data=rollback_data,
        created_by=rolled_back_by,
        change_description=f"Rollback to version {version_number}",
    )

    rollback_version["rollback_from_version"] = versions[-1]["version_number"] if len(versions) > 1 else 1
    rollback_version["rollback_to_version"] = version_number

    logger.info(
        "version_rollback",
        entity_type=entity_type,
        entity_id=entity_id,
        rolled_back_to=version_number,
    )

    return {
        "success": True,
        "entity_type": entity_type,
        "entity_id": entity_id,
        "rolled_back_to_version": version_number,
        "new_version_number": rollback_version["version_number"],
        "data": rollback_data,
        "rollback_entry": rollback_version,
    }


def get_latest_version(entity_type: str, entity_id: str) -> Optional[Dict[str, Any]]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.get(key, [])
    if not versions:
        return None
    return versions[-1]


def get_version(entity_type: str, entity_id: str, version_number: int) -> Optional[Dict[str, Any]]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.get(key, [])
    for v in versions:
        if v["version_number"] == version_number:
            return v
    return None


def diff_versions(
    entity_type: str,
    entity_id: str,
    version_a: int,
    version_b: int,
) -> Dict[str, Any]:
    key = _storage_key(entity_type, entity_id)
    versions = _version_store.get(key, [])

    data_a = None
    data_b = None
    for v in versions:
        if v["version_number"] == version_a:
            data_a = v["snapshot"]
        if v["version_number"] == version_b:
            data_b = v["snapshot"]

    if data_a is None or data_b is None:
        raise EngineError(
            "One or both versions not found for diff",
            code="VERSION_NOT_FOUND",
        )

    differences: List[Dict[str, Any]] = []
    all_keys = set(list(data_a.keys()) + list(data_b.keys()))

    for k in sorted(all_keys):
        val_a = data_a.get(k)
        val_b = data_b.get(k)
        if val_a != val_b:
            differences.append({
                "field": k,
                "version_a": val_a,
                "version_b": val_b,
            })

    return {
        "entity_type": entity_type,
        "entity_id": entity_id,
        "version_a": version_a,
        "version_b": version_b,
        "differences": differences,
        "fields_changed": len(differences),
    }
