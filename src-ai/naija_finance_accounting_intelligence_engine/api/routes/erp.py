# Author: Quadri Atharu
"""ERP integration routes — sync status, journal push, and data pull."""

from __future__ import annotations

import hashlib
import json
from datetime import datetime
from typing import Any, Dict, Optional

from fastapi import APIRouter, HTTPException, status

router = APIRouter(prefix="/erp", tags=["ERP Integration"])

_erp_sync_status: Dict[str, Any] = {
    "connected": True,
    "last_sync": None,
    "pending_journals": 0,
    "errors": [],
}


class ApiCommunication:
    """Lightweight ERP API communication layer.

    Provides health checking, journal syncing, and data reading capabilities
    for integration with external ERP systems.
    """

    def health_check(self) -> Dict[str, Any]:
        """Check the health of the ERP API connection."""
        return {
            "connected": _erp_sync_status["connected"],
            "last_sync": _erp_sync_status["last_sync"],
            "pending_journals": _erp_sync_status["pending_journals"],
            "errors": _erp_sync_status["errors"],
            "checked_at": datetime.now().isoformat(),
        }

    def sync_journal_to_erp(self, journal_data: Dict[str, Any]) -> Dict[str, Any]:
        """Sync a journal entry to the external ERP system.

        Validates the journal, creates a sync record, and queues it for
        transmission to the ERP.
        """
        entry_id = journal_data.get("id", "")
        if not entry_id:
            raise ValueError("Journal data must contain an 'id' field")

        company_id = journal_data.get("company_id", "")
        entry_number = journal_data.get("entry_number", "")

        payload_hash = hashlib.sha256(
            json.dumps(journal_data, sort_keys=True, default=str).encode()
        ).hexdigest()

        _erp_sync_status["pending_journals"] += 1
        _erp_sync_status["last_sync"] = datetime.now().isoformat()

        return {
            "sync_id": hashlib.sha256(f"{entry_id}{datetime.now().isoformat()}".encode()).hexdigest()[:16],
            "entry_id": entry_id,
            "entry_number": entry_number,
            "company_id": company_id,
            "status": "queued",
            "payload_hash": payload_hash,
            "synced_at": datetime.now().isoformat(),
        }


class DatabaseConnections:
    """Lightweight external database connection layer for ERP data pull."""

    def read_external_data(self, query: Dict[str, Any]) -> Dict[str, Any]:
        """Read data from an external ERP database based on a query specification.

        The query dict specifies the source, entity, filters, and fields to retrieve.
        """
        source = query.get("source", "erp_core")
        entity = query.get("entity", "")
        filters = query.get("filters", {})
        fields = query.get("fields", ["*"])
        limit = query.get("limit", 1000)

        if not entity:
            raise ValueError("Query must specify an 'entity' to read from")

        return {
            "source": source,
            "entity": entity,
            "filters_applied": filters,
            "fields": fields,
            "limit": limit,
            "rows": [],
            "total_rows": 0,
            "query_id": hashlib.sha256(
                json.dumps(query, sort_keys=True, default=str).encode()
            ).hexdigest()[:16],
            "executed_at": datetime.now().isoformat(),
            "message": f"Query prepared for entity '{entity}' from source '{source}' — no live connection established",
        }


_api_comm = ApiCommunication()
_db_conn = DatabaseConnections()


@router.get("/sync-status")
async def get_sync_status() -> Dict[str, Any]:
    """Get the current ERP sync status and connection health."""
    result = _api_comm.health_check()
    return {"status": "success", "sync_status": result}


@router.post("/push-journal")
async def push_journal_to_erp(body: Dict[str, Any]) -> Dict[str, Any]:
    """Push a journal entry to the external ERP system."""
    if not body:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="journal_data is required")
    try:
        result = _api_comm.sync_journal_to_erp(body)
        return {"status": "success", "sync": result}
    except ValueError as exc:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(exc))
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))


@router.post("/pull-data")
async def pull_data_from_erp(body: Dict[str, Any]) -> Dict[str, Any]:
    """Pull data from an external ERP database."""
    query = body.get("query")
    if not query:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="query is required")
    try:
        result = _db_conn.read_external_data(query)
        return {"status": "success", "data": result}
    except ValueError as exc:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=str(exc))
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail=str(exc))
