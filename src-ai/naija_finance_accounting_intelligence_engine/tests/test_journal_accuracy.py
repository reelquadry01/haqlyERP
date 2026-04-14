# Author: Quadri Atharu
"""Pytest tests for double-entry journal accuracy and integrity."""

import pytest

from ..accounting.journal_engine import JournalEngine
from ..accounting.ledger_engine import LedgerEngine
from ..core.exceptions import AccountingError


@pytest.fixture
def journal_engine():
    return JournalEngine()


@pytest.fixture
def balanced_journal_data():
    return {
        "transaction_type": "SALES",
        "amount": 100000,
        "description": "Sale of goods to Customer A",
        "company_id": "HAQLY-001",
        "entry_date": "2025-01-15T00:00:00",
        "lines": [
            {"account_code": "1100", "description": "Accounts Receivable", "debit": 100000, "credit": 0.0},
            {"account_code": "4000", "description": "Sales Revenue", "debit": 0.0, "credit": 100000},
        ],
    }


class TestDoubleEntryBalance:
    def test_double_entry_balance(self, journal_engine, balanced_journal_data):
        entry = journal_engine.create_journal_entry(balanced_journal_data)
        assert entry["total_debit"] == entry["total_credit"]
        assert abs(entry["total_debit"] - 100000) < 0.01
        assert entry["status"] == "DRAFT"
        assert len(entry["lines"]) == 2

    def test_auto_generated_lines_balance(self, journal_engine):
        data = {
            "transaction_type": "PURCHASE",
            "amount": 50000,
            "description": "Purchase from Supplier B",
            "company_id": "HAQLY-001",
            "debit_account_code": "5000",
            "credit_account_code": "2100",
        }
        entry = journal_engine.create_journal_entry(data)
        assert entry["total_debit"] == entry["total_credit"]
        assert abs(entry["total_debit"] - 50000) < 0.01


class TestUnbalancedJournalRejected:
    def test_unbalanced_journal_rejected(self, journal_engine):
        data = {
            "transaction_type": "JOURNAL",
            "amount": 100000,
            "description": "Intentionally unbalanced entry",
            "company_id": "HAQLY-001",
            "lines": [
                {"account_code": "1100", "description": "AR", "debit": 100000, "credit": 0.0},
                {"account_code": "4000", "description": "Revenue", "debit": 0.0, "credit": 80000},
            ],
        }
        with pytest.raises(AccountingError, match="does not balance"):
            journal_engine.create_journal_entry(data)

    def test_line_with_both_debit_and_credit_rejected(self, journal_engine):
        data = {
            "transaction_type": "JOURNAL",
            "amount": 100000,
            "description": "Bad line",
            "company_id": "HAQLY-001",
            "lines": [
                {"account_code": "1100", "description": "AR", "debit": 50000, "credit": 50000},
                {"account_code": "4000", "description": "Revenue", "debit": 0.0, "credit": 50000},
            ],
        }
        with pytest.raises(AccountingError, match="cannot have both debit"):
            journal_engine.create_journal_entry(data)

    def test_zero_amount_line_rejected(self, journal_engine):
        data = {
            "transaction_type": "JOURNAL",
            "amount": 100000,
            "description": "Zero line",
            "company_id": "HAQLY-001",
            "lines": [
                {"account_code": "1100", "description": "AR", "debit": 100000, "credit": 0.0},
                {"account_code": "4000", "description": "Revenue", "debit": 0.0, "credit": 0.0},
            ],
        }
        with pytest.raises(AccountingError, match="must have either a debit or credit"):
            journal_engine.create_journal_entry(data)


class TestJournalPosting:
    def test_journal_posting_workflow(self, journal_engine, balanced_journal_data):
        entry = journal_engine.create_journal_entry(balanced_journal_data)
        assert entry["status"] == "DRAFT"

        approved = journal_engine.approve_entry(entry["id"], "admin@haqly.com")
        assert approved["status"] == "APPROVED"
        assert approved["approved_by"] == "admin@haqly.com"

        posted = journal_engine.post_entry(entry["id"])
        assert posted["status"] == "POSTED"

    def test_cannot_post_draft_directly(self, journal_engine, balanced_journal_data):
        entry = journal_engine.create_journal_entry(balanced_journal_data)
        with pytest.raises(AccountingError, match="Only approved entries can be posted"):
            journal_engine.post_entry(entry["id"])


class TestJournalReversal:
    def test_journal_reversal_creates_mirror_entries(self, journal_engine, balanced_journal_data):
        entry = journal_engine.create_journal_entry(balanced_journal_data)
        approved = journal_engine.approve_entry(entry["id"], "admin@haqly.com")

        reversal = journal_engine.reverse_entry(entry["id"], reason="Correcting error")
        assert reversal["is_reversing"] is True
        assert reversal["reversing_entry_id"] == entry["id"]

        original_lines = entry["lines"]
        reversal_lines = reversal["lines"]
        assert len(original_lines) == len(reversal_lines)

        for orig, rev in zip(original_lines, reversal_lines):
            assert rev["debit"] == orig["credit"]
            assert rev["credit"] == orig["debit"]
            assert rev["account_code"] == orig["account_code"]

        assert entry["status"] == "REVERSED"

    def test_cannot_reverse_voided_entry(self, journal_engine, balanced_journal_data):
        entry = journal_engine.create_journal_entry(balanced_journal_data)
        journal_engine.void_entry(entry["id"])
        with pytest.raises(AccountingError, match="already voided"):
            journal_engine.reverse_entry(entry["id"])
