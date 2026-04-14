# Author: Quadri Atharu
"""Async session factory and FastAPI dependency injection for database sessions."""

from __future__ import annotations

from typing import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, AsyncEngine

from .database import create_async_engine, Base

_engine = create_async_engine()

AsyncSessionLocal = async_sessionmaker(
    bind=_engine,
    class_=AsyncSession,
    expire_on_commit=False,
    autoflush=False,
)


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    """FastAPI dependency that yields an async database session."""
    async with AsyncSessionLocal() as session:
        try:
            yield session
            await session.commit()
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()


def set_engine(engine: AsyncEngine) -> None:
    """Replace the default engine (useful for testing with a different DB)."""
    global _engine, AsyncSessionLocal
    _engine = engine
    AsyncSessionLocal = async_sessionmaker(
        bind=_engine,
        class_=AsyncSession,
        expire_on_commit=False,
        autoflush=False,
    )


def get_engine() -> AsyncEngine:
    """Return the current engine instance."""
    return _engine
