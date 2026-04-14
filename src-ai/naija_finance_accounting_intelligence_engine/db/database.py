# Author: Quadri Atharu
"""SQLAlchemy async engine and declarative base for the HAQLY AI Finance Engine."""

from __future__ import annotations

from typing import Optional

from sqlalchemy.ext.asyncio import create_async_engine as _create_async_engine, AsyncEngine
from sqlalchemy.orm import declarative_base

Base = declarative_base()

_DEFAULT_DATABASE_URL = "postgresql+asyncpg://haqly:haqly_secret@localhost:5432/haqly_finance"


def create_async_engine(database_url: Optional[str] = None, **kwargs) -> AsyncEngine:
    """Create an async SQLAlchemy engine using asyncpg.

    Args:
        database_url: PostgreSQL connection string. If None, uses default.
        **kwargs: Additional arguments passed to SQLAlchemy create_async_engine.

    Returns:
        AsyncEngine ready for async session creation.
    """
    url = database_url or _DEFAULT_DATABASE_URL

    defaults = {
        "echo": False,
        "pool_size": 10,
        "max_overflow": 20,
        "pool_pre_ping": True,
        "pool_recycle": 3600,
    }
    defaults.update(kwargs)

    engine = _create_async_engine(url, **defaults)
    return engine


async def init_db(engine: AsyncEngine) -> None:
    """Initialize the database schema (create all tables)."""
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)


async def drop_db(engine: AsyncEngine) -> None:
    """Drop all tables (for testing only)."""
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
