# Author: Quadri Atharu
"""Application configuration using Pydantic BaseSettings."""

from __future__ import annotations

import os
from typing import Dict

from pydantic import Field, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Central configuration for the Nigerian Finance & Accounting Intelligence Engine."""

    model_config = SettingsConfigDict(
        env_prefix="NAIJA_ENGINE_",
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    database_url: str = Field(
        default="",
        description="Async PostgreSQL connection string",
    )
    database_url_sync: str = Field(
        default="",
        description="Sync PostgreSQL connection string for migrations",
    )
    api_host: str = Field(default="0.0.0.0", description="API bind host")
    api_port: int = Field(default=8200, description="API bind port")
    jwt_secret: str = Field(
        default_factory=lambda: os.environ.get("NAIJA_ENGINE_JWT_SECRET") or (
            "dev-only-secret" if os.environ.get("NAIJA_ENGINE_ENV") == "development"
            else ""
        ),
        description="JWT signing secret (shared with Rust backend). Must be set in production.",
    )
    jwt_algorithm: str = Field(default="HS256", description="JWT algorithm")
    rust_backend_url: str = Field(
        default="http://localhost:8100",
        description="Base URL of the HAQLY Rust/Tauri backend",
    )
    ollama_url: str = Field(
        default="http://localhost:11434",
        description="Ollama local LLM endpoint",
    )
    ollama_model: str = Field(
        default="llama3",
        description="Default Ollama model for AI tasks",
    )

    vat_rate: float = Field(default=0.075, description="Nigerian VAT standard rate (7.5%)")
    wht_rates: Dict[str, float] = Field(
        default={
            "contractors": 0.05,
            "consultancy": 0.05,
            "management_services": 0.05,
            "technical_services": 0.05,
            "professional_services": 0.05,
            "commission": 0.05,
            "interest": 0.10,
            "dividends": 0.10,
            "rent": 0.10,
            "royalties": 0.05,
        },
        description="Nigerian WHT rates by payment category (company recipients)",
    )
    wht_rates_individual: Dict[str, float] = Field(
        default={
            "interest": 0.05,
            "dividends": 0.05,
            "rent": 0.05,
        },
        description="Nigerian WHT rates for individual recipients (Tax Reform 2025)",
    )
    cit_thresholds: Dict[str, object] = Field(
        default={
            "small_company": {"max_turnover": 50_000_000, "rate": 0.00},
            "medium_company": {"min_turnover": 50_000_001, "max_turnover": 250_000_000, "rate": 0.15},
            "large_company": {"min_turnover": 250_000_001, "rate": 0.25},
            "minimum_tax_rate": 0.005,
            "minimum_tax_threshold": 50_000_000,
        },
        description="CIT thresholds and rates (Tax Reform 2025)",
    )
    edu_tax_rate: float = Field(default=0.01, description="Education tax rate (1% - Tax Reform 2025)")
    cgt_rates: Dict[str, float] = Field(
        default={
            "up_to_50m": 0.10,
            "50m_to_250m": 0.15,
            "above_250m": 0.20,
        },
        description="Progressive CGT rates (Tax Reform 2025)",
    )
    paye_brackets: Dict[str, object] = Field(
        default={
            "band_1": {"max": 800_000, "rate": 0.00},
            "band_2": {"max": 3_200_000, "rate": 0.15},
            "band_3": {"max": 7_200_000, "rate": 0.20},
            "band_4": {"max": 14_000_000, "rate": 0.25},
            "band_5": {"max": 25_000_000, "rate": 0.30},
            "band_6": {"max": None, "rate": 0.35},
        },
        description="PAYE progressive brackets (Tax Reform 2025)",
    )
    vat_registration_threshold: int = Field(default=50_000_000, description="VAT registration threshold NGN 50M (Tax Reform 2025)")
    default_currency: str = Field(default="NGN", description="Default currency code")

    log_level: str = Field(default="INFO", description="Logging level")
    debug: bool = Field(default=False, description="Debug mode toggle")

    cors_origins: list[str] = Field(
        default=["http://localhost:3000", "http://localhost:8100", "tauri://localhost"],
        description="Allowed CORS origins",
    )

    @model_validator(mode="after")
    def _fail_secure_jwt_secret(self) -> "Settings":
        if not self.jwt_secret and os.environ.get("NAIJA_ENGINE_ENV") != "development":
            raise RuntimeError("FATAL: NAIJA_ENGINE_JWT_SECRET must be set in production")
        if not self.database_url and os.environ.get("NAIJA_ENGINE_ENV") != "development":
            raise RuntimeError("FATAL: NAIJA_ENGINE_DATABASE_URL must be set in production")
        if not self.database_url_sync and os.environ.get("NAIJA_ENGINE_ENV") != "development":
            raise RuntimeError("FATAL: NAIJA_ENGINE_DATABASE_URL_SYNC must be set in production")
        return self


settings = Settings()
