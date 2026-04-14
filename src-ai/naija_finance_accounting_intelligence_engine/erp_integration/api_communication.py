# Author: Quadri Atharu
"""API communication with the HAQLY Rust/Tauri backend."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, Optional

import httpx

from ..core.config import settings
from ..core.exceptions import EngineError
from ..core.logging import get_logger

logger = get_logger(__name__)

_DEFAULT_TIMEOUT = 30.0
_MAX_RETRIES = 3


async def call_rust_backend(
    endpoint: str,
    method: str = "POST",
    data: Optional[Dict[str, Any]] = None,
    token: Optional[str] = None,
    timeout: float = _DEFAULT_TIMEOUT,
) -> Dict[str, Any]:
    base_url = settings.rust_backend_url.rstrip("/")
    url = f"{base_url}/{endpoint.lstrip('/')}"

    headers: Dict[str, str] = {
        "Content-Type": "application/json",
        "Accept": "application/json",
        "X-Engine-Source": "naija_finance_intelligence",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"

    async with httpx.AsyncClient(timeout=timeout) as client:
        last_error: Optional[Exception] = None
        for attempt in range(1, _MAX_RETRIES + 1):
            try:
                method_upper = method.upper()
                if method_upper == "GET":
                    response = await client.get(url, headers=headers, params=data)
                elif method_upper == "POST":
                    response = await client.post(url, headers=headers, json=data)
                elif method_upper == "PUT":
                    response = await client.put(url, headers=headers, json=data)
                elif method_upper == "PATCH":
                    response = await client.patch(url, headers=headers, json=data)
                elif method_upper == "DELETE":
                    response = await client.delete(url, headers=headers, json=data or {})
                else:
                    raise EngineError(f"Unsupported HTTP method: {method}", code="INVALID_METHOD")

                if response.status_code >= 500:
                    last_error = EngineError(
                        f"Backend server error: {response.status_code}",
                        code="BACKEND_SERVER_ERROR",
                        details={"status": response.status_code, "body": response.text[:500]},
                    )
                    if attempt < _MAX_RETRIES:
                        logger.warning("backend_retry", attempt=attempt, status=response.status_code, url=url)
                        continue
                    raise last_error

                if response.status_code >= 400:
                    raise EngineError(
                        f"Backend client error: {response.status_code}",
                        code="BACKEND_CLIENT_ERROR",
                        details={"status": response.status_code, "body": response.text[:500]},
                    )

                result = response.json()
                logger.info("backend_call_success", method=method_upper, endpoint=endpoint, status=response.status_code)
                return result

            except httpx.TimeoutException as exc:
                last_error = exc
                logger.warning("backend_timeout", attempt=attempt, url=url)
                if attempt >= _MAX_RETRIES:
                    raise EngineError(
                        f"Backend request timed out after {attempt} attempts",
                        code="BACKEND_TIMEOUT",
                        details={"url": url, "timeout": timeout},
                    )

            except httpx.ConnectError as exc:
                last_error = exc
                logger.warning("backend_connection_failed", attempt=attempt, url=url)
                if attempt >= _MAX_RETRIES:
                    raise EngineError(
                        f"Cannot connect to backend at {url}",
                        code="BACKEND_UNREACHABLE",
                        details={"url": url},
                    )

    raise EngineError("Unexpected backend communication failure", code="BACKEND_ERROR")


async def sync_journal_to_erp(journal: Dict[str, Any]) -> Dict[str, Any]:
    entry_id = journal.get("id", "")
    entry_number = journal.get("entry_number", "")
    company_id = journal.get("company_id", "")
    status = journal.get("status", "DRAFT")

    sync_payload: Dict[str, Any] = {
        "journal_entry_id": entry_id,
        "entry_number": entry_number,
        "company_id": company_id,
        "entry_date": journal.get("entry_date"),
        "description": journal.get("description", ""),
        "lines": journal.get("lines", []),
        "total_debit": journal.get("total_debit", 0),
        "total_credit": journal.get("total_credit", 0),
        "status": status,
        "source_type": journal.get("source_type"),
        "reference": journal.get("reference"),
        "is_adjusting": journal.get("is_adjusting", False),
        "sync_source": "naija_finance_intelligence",
        "sync_timestamp": datetime.now().isoformat(),
    }

    result = await call_rust_backend(
        endpoint="/api/v1/journals/sync",
        method="POST",
        data=sync_payload,
    )

    sync_result: Dict[str, Any] = {
        "success": True,
        "journal_entry_id": entry_id,
        "entry_number": entry_number,
        "synced_at": datetime.now().isoformat(),
        "backend_response": result,
    }

    logger.info("journal_synced_to_erp", entry_number=entry_number, company_id=company_id)
    return sync_result
