"""Multi-currency transaction processing and statement translation.

Author: Quadri Atharu

Processes foreign currency transactions by converting them to the
functional currency using spot rates, translates financial statements
using closing/average rate methods per IAS 21, and computes net
FX exposure across currency positions.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal
from typing import Any


TWO_PLACES = Decimal("0.01")


def _d(value: Any) -> Decimal:
    return Decimal(str(value)).quantize(TWO_PLACES)


@dataclass
class JournalEntry:
    date: str
    description: str
    debit_account: str
    credit_account: str
    amount: Decimal
    currency: str = "NGN"
    reference: str = ""


@dataclass
class FXTransactionResult:
    original_amount: Decimal
    original_currency: str
    functional_amount: Decimal
    functional_currency: str
    spot_rate: Decimal
    journal_entries: list[JournalEntry] = field(default_factory=list)


@dataclass
class TranslatedStatements:
    closing_rate: Decimal
    average_rate: Decimal
    translated_income_statement: dict[str, Decimal] = field(default_factory=dict)
    translated_balance_sheet: dict[str, Decimal] = field(default_factory=dict)
    translation_reserve: Decimal = Decimal("0")


@dataclass
class FXExposure:
    currency: str
    position: Decimal
    rate: Decimal
    ngn_equivalent: Decimal
    net_exposure: Decimal = Decimal("0")


def process_fx_transaction(
    transaction: dict[str, Any],
    spot_rate: Decimal,
) -> FXTransactionResult:
    """Process a foreign currency transaction by converting to functional
    currency and generating journal entries.

    Per IAS 21.21, a foreign currency transaction is recorded using the
    spot exchange rate at the date of the transaction.

    Args:
        transaction: Dict with 'amount', 'currency', 'type' (purchase/sale/
                    receipt/payment), 'date', 'account_debit', 'account_credit',
                    'description', 'reference'.
        spot_rate: Spot exchange rate (foreign units per 1 NGN, or NGN per
                  1 foreign unit depending on quotation convention).

    Returns:
        FXTransactionResult with converted amounts and journal entries.
    """
    original_amount = _d(transaction["amount"])
    original_currency = transaction.get("currency", "USD")
    spot_rate = _d(spot_rate)
    functional_amount = (original_amount * spot_rate).quantize(TWO_PLACES)

    entry = JournalEntry(
        date=transaction.get("date", ""),
        description=transaction.get("description", f"FX transaction {original_currency} -> NGN"),
        debit_account=transaction.get("account_debit", "Foreign Currency Receivable"),
        credit_account=transaction.get("account_credit", "Bank"),
        amount=functional_amount,
        currency="NGN",
        reference=transaction.get("reference", ""),
    )

    return FXTransactionResult(
        original_amount=original_amount,
        original_currency=original_currency,
        functional_amount=functional_amount,
        functional_currency="NGN",
        spot_rate=spot_rate,
        journal_entries=[entry],
    )


def translate_financial_statements(
    statements: dict[str, Any],
    closing_rate: Decimal,
    average_rate: Decimal,
) -> TranslatedStatements:
    """Translate foreign entity financial statements to the presentation
    currency per IAS 21.

    Income statement items: translated at average rate for the period.
    Balance sheet assets and liabilities: translated at closing rate.
    Equity items (share capital): translated at historical rate.
    The translation difference is recognized in OCI as a translation reserve.

    Args:
        statements: Dict with 'income_statement' (dict of line items),
                   'balance_sheet' with 'assets', 'liabilities', 'equity',
                   and 'historical_rates' for equity items.
        closing_rate: Exchange rate at the reporting date.
        average_rate: Average exchange rate for the period.

    Returns:
        TranslatedStatements with all items converted and translation reserve.
    """
    closing_rate = _d(closing_rate)
    average_rate = _d(average_rate)

    translated_is: dict[str, Decimal] = {}
    total_is = Decimal("0")

    for key, value in statements.get("income_statement", {}).items():
        translated = (_d(value) * average_rate).quantize(TWO_PLACES)
        translated_is[key] = translated
        total_is += translated

    translated_bs: dict[str, Decimal] = {}
    total_assets = Decimal("0")
    total_liabilities = Decimal("0")
    total_equity_historical = Decimal("0")
    total_equity_closing = Decimal("0")

    for key, value in statements.get("balance_sheet", {}).get("assets", {}).items():
        translated = (_d(value) * closing_rate).quantize(TWO_PLACES)
        translated_bs[f"asset_{key}"] = translated
        total_assets += translated

    for key, value in statements.get("balance_sheet", {}).get("liabilities", {}).items():
        translated = (_d(value) * closing_rate).quantize(TWO_PLACES)
        translated_bs[f"liability_{key}"] = translated
        total_liabilities += translated

    historical_rates = statements.get("historical_rates", {})
    for key, value in statements.get("balance_sheet", {}).get("equity", {}).items():
        rate = _d(historical_rates.get(key, average_rate))
        translated = (_d(value) * rate).quantize(TWO_PLACES)
        translated_bs[f"equity_{key}"] = translated
        total_equity_historical += _d(value) * average_rate
        total_equity_closing += translated

    implied_equity = total_assets - total_liabilities
    translation_reserve = (implied_equity - total_equity_closing).quantize(TWO_PLACES)

    return TranslatedStatements(
        closing_rate=closing_rate,
        average_rate=average_rate,
        translated_income_statement=translated_is,
        translated_balance_sheet=translated_bs,
        translation_reserve=translation_reserve,
    )


def compute_fx_exposure(
    positions: list[dict[str, Any]],
    rates: dict[str, Decimal],
) -> list[FXExposure]:
    """Compute net FX exposure across all foreign currency positions.

    Aggregates long and short positions by currency and converts to
    NGN equivalent using provided rates. Net exposure shows the
    potential gain/loss from a 1-unit change in each exchange rate.

    Args:
        positions: List of dicts with 'currency', 'amount' (positive for
                  assets, negative for liabilities), 'entity'.
        rates: Dict mapping currency code to current NGN rate.

    Returns:
        List of FXExposure objects, one per currency with net position.
    """
    currency_positions: dict[str, Decimal] = {}

    for pos in positions:
        currency = pos["currency"]
        amount = _d(pos["amount"])
        currency_positions[currency] = currency_positions.get(currency, Decimal("0")) + amount

    exposures: list[FXExposure] = []
    for currency, net_position in currency_positions.items():
        rate = _d(rates.get(currency, Decimal("1")))
        ngn_equivalent = (abs(net_position) * rate).quantize(TWO_PLACES)
        net_exposure = (net_position * rate).quantize(TWO_PLACES)

        exposures.append(FXExposure(
            currency=currency,
            position=net_position,
            rate=rate,
            ngn_equivalent=ngn_equivalent,
            net_exposure=net_exposure,
        ))

    return exposures
