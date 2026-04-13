"""Telecommunications industry profile.

Author: Quadri Atharu

Covers Nigerian telecoms with subscription, data, voice, VAS revenue,
network OPEX, spectrum fees, NCC levy, and key telecom KPIs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class TelecommunicationsProfile:
    name: str = "Telecommunications"
    typical_revenue_models: dict[str, str] = field(default_factory=lambda: {
        "subscription_recurring": "Monthly/weekly subscription fees",
        "data_revenue": "Data bundle and pay-as-you-go revenue",
        "voice_revenue": "Voice call revenue (on-net, off-net, international)",
        "vas": "Value-added services (SMS, USSD, mobile money, content)",
        "interconnect": "Interconnect revenue from other operators",
        "device_sales": "Handset and device sales revenue",
    })
    cost_structures: dict[str, str] = field(default_factory=lambda: {
        "network_opex": "Network operating costs (tower, power, maintenance)",
        "spectrum_fees": "Annual spectrum licence fees to NCC",
        "interconnect_cost": "Interconnect charges paid to other operators",
        "sales_marketing": "Customer acquisition and retention costs",
        "it_systems": "IT infrastructure and platform costs",
    })
    tax_implications: dict[str, Decimal] = field(default_factory=lambda: {
        "vat": Decimal("7.5"),
        "cit": Decimal("30"),
        "ncc_levy": Decimal("2.5"),
        "education_tax": Decimal("2"),
    })
    compliance_requirements: list[str] = field(default_factory=lambda: [
        "NCC licence compliance",
        "NCC annual operating levy",
        "Spectrum licence conditions",
        "Consumer protection (NCC regulations)",
        "NIMC NIN-SIM linkage compliance",
        "DND (Do Not Disturb) compliance",
        "FIRS tax returns",
    ])
    inventory_logic: str = "Device inventory valued at FIFO. SIM cards and airtime pins at cost."
    depreciation_patterns: dict[str, str] = field(default_factory=lambda: {
        "network_equipment": "Straight-line over 7-15 years",
        "towers": "Straight-line over 15-20 years",
        "spectrum_licence": "Amortised over licence period",
        "it_systems": "Straight-line over 3-5 years",
    })
    typical_coa_ranges: dict[str, tuple[int, int]] = field(default_factory=lambda: {
        "network_assets": (2100, 2499),
        "spectrum_licence": (2500, 2599),
        "revenue": (6100, 6499),
        "network_opex": (7100, 7399),
        "interconnect": (7400, 7499),
    })
    key_kpis: list[str] = field(default_factory=lambda: [
        "arpu",
        "churn_rate",
        "subscriber_growth",
        "network_uptime",
    ])

    def get_posting_suggestions(self, transaction: dict[str, Any]) -> list[dict[str, str]]:
        txn_type = transaction.get("type", "")
        posting_map: dict[str, list[dict[str, str]]] = {
            "subscription_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Subscription Revenue"},
            ],
            "data_revenue": [
                {"debit_account": "Bank / Receivables", "credit_account": "Data Revenue"},
            ],
            "network_opex": [
                {"debit_account": "Network Operating Expense", "credit_account": "Bank / AP"},
            ],
            "spectrum_fee": [
                {"debit_account": "Spectrum Licence (Prepaid)", "credit_account": "Bank"},
            ],
            "interconnect_cost": [
                {"debit_account": "Interconnect Expense", "credit_account": "Bank / Interconnect Payable"},
            ],
            "device_sale": [
                {"debit_account": "Bank / Receivables", "credit_account": "Device Sales Revenue"},
                {"debit_account": "Cost of Devices Sold", "credit_account": "Device Inventory"},
            ],
        }
        return posting_map.get(txn_type, [{"debit_account": "Operating Expense", "credit_account": "Bank / AP"}])

    def get_industry_kpis(self, financial_data: dict[str, Any]) -> dict[str, Decimal]:
        total_revenue = _d(financial_data.get("total_revenue", 0))
        total_subscribers = _d(financial_data.get("total_subscribers", 1))
        subscribers_lost = _d(financial_data.get("subscribers_lost", 0))
        avg_subscribers = _d(financial_data.get("avg_subscribers", 1))
        new_subscribers = _d(financial_data.get("new_subscribers", 0))
        uptime_pct = _d(financial_data.get("network_uptime_pct", 0))

        arpu = (total_revenue / total_subscribers / Decimal("12")).quantize(TWO_PLACES) if total_subscribers > 0 else Decimal("0")
        churn = (subscribers_lost / avg_subscribers * Decimal("100")).quantize(TWO_PLACES) if avg_subscribers > 0 else Decimal("0")
        growth = (new_subscribers / avg_subscribers * Decimal("100")).quantize(TWO_PLACES) if avg_subscribers > 0 else Decimal("0")

        return {
            "arpu_monthly": arpu,
            "churn_rate_pct": churn,
            "subscriber_growth_pct": growth,
            "network_uptime_pct": uptime_pct,
        }
