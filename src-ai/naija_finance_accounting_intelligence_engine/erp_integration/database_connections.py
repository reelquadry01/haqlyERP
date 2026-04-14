# Author: Quadri Atharu
"""Database connection management for external data sources using SQLAlchemy."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from sqlalchemy import create_engine, text
from sqlalchemy.engine import Engine
from sqlalchemy.pool import QueuePool

from ..core.exceptions import EngineError
from ..core.logging import get_logger

logger = get_logger(__name__)

_engine_cache: Dict[str, Engine] = {}


def connect_to_external_db(
    url: str,
    pool_size: int = 5,
    max_overflow: int = 10,
    pool_timeout: int = 30,
    pool_recycle: int = 3600,
    echo: bool = False,
) -> Engine:
    if url in _engine_cache:
        cached = _engine_cache[url]
        try:
            with cached.connect() as conn:
                conn.execute(text("SELECT 1"))
            return cached
        except Exception:
            del _engine_cache[url]

    try:
        engine = create_engine(
            url,
            poolclass=QueuePool,
            pool_size=pool_size,
            max_overflow=max_overflow,
            pool_timeout=pool_timeout,
            pool_recycle=pool_recycle,
            pool_pre_ping=True,
            echo=echo,
        )
        with engine.connect() as conn:
            conn.execute(text("SELECT 1"))
        _engine_cache[url] = engine
        logger.info("external_db_connected", url=_mask_url(url))
        return engine
    except Exception as exc:
        raise EngineError(
            f"Failed to connect to external database: {exc}",
            code="DB_CONNECTION_ERROR",
            details={"url": _mask_url(url)},
        )


def read_external_data(
    query: str,
    engine: Engine,
    params: Optional[Dict[str, Any]] = None,
    max_rows: int = 10000,
) -> List[Dict[str, Any]]:
    if not query.strip().upper().startswith("SELECT"):
        raise EngineError(
            "Only SELECT queries are permitted for external data reads",
            code="INVALID_QUERY_TYPE",
            details={"query_prefix": query[:50]},
        )

    bounded_query = query
    if "LIMIT" not in query.upper():
        bounded_query = f"{query.rstrip(';')} LIMIT {max_rows}"

    try:
        with engine.connect() as conn:
            result = conn.execute(text(bounded_query), params or {})
            columns = list(result.keys())
            rows = result.fetchall()

        data = [dict(zip(columns, row)) for row in rows]
        logger.info("external_data_read", rows=len(data), columns=len(columns))
        return data
    except Exception as exc:
        raise EngineError(
            f"Failed to read external data: {exc}",
            code="DB_QUERY_ERROR",
            details={"query": query[:200]},
        )


def close_engine(url: str) -> None:
    engine = _engine_cache.pop(url, None)
    if engine:
        engine.dispose()
        logger.info("external_db_disposed", url=_mask_url(url))


def close_all_engines() -> None:
    for url, engine in list(_engine_cache.items()):
        engine.dispose()
        logger.info("external_db_disposed", url=_mask_url(url))
    _engine_cache.clear()


def _mask_url(url: str) -> str:
    if "@" in url:
        parts = url.split("@")
        credentials_part = parts[0]
        if ":" in credentials_part.split("://")[-1]:
            prefix = credentials_part.rsplit(":", 1)[0]
            return f"{prefix}:****@{parts[1]}"
    return url
