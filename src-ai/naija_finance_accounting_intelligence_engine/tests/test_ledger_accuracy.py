# Author: Quadri Atharu
"""Pytest tests for ledger engine accuracy, T-account posting, and trial balance."""

import pytest

from ..accounting.ledger_engine import LedgerEngine, DEBIT_NORMAL, CREDIT_NORMAL


@pytest.fixture
def ledger():
    return LedgerEngine()


@pytest.fixture
def sales_journal():
    return {
        "id": "JE-001",
        "company_id": "HAQLY-001",
        "entry_date": "2025-01-15T00:00:00",
        "description": "Sale of goods to Customer A",
        "entry_number": "JE-001",
        "lines": [
            {"account_code": "1100", "description": "Accounts Receivable", "debit": 5000000, "credit": 0.0},
            {"account_code": "4000", "description": "Sales Revenue", "debit": 0.0, "credit": 5000000},
        ],
    }


@pytest.fixture
def purchase_journal():
    return {
        "id": "JE-002",
        "company_id": "HAQLY-001",
        "entry_date": "2025-01-16T00:00:00",
        "description": "Purchase of raw materials",
        "entry_number": "JE-002",
        "lines": [
            {"account_code": "1200", "description": "Inventory", "debit": 3000000, "credit": 0.0},
            {"account_code": "2100", "description": "Accounts Payable", "debit": 0.0, "credit": 3000000},
        ],
    }


@pytest.fixture
def fx_journal():
    return {
        "id": "JE-FX-001",
        "company_id": "HAQLY-001",
        "entry_date": "2025-02-01T00:00:00",
        "description": "USD receivable settled at ₦1,550/USD",
        "entry_number": "JE-FX-001",
        "lines": [
            {"account_code": "1020", "description": "Bank", "debit": 155000000, "credit": 0.0},
            {"account_code": "1100", "description": "Accounts Receivable", "debit": 0.0, "credit": 150000000},
            {"account_code": "4100", "description": "Other Income - FX Gain", "debit": 0.0, "credit": 5000000},
        ],
    }


class TestDoubleEntryBalance:
    def test_double_entry_balance(self, ledger, sales_journal):
        result = ledger.post_journal_entry(sales_journal)
        assert result["posted_to_gl"] == 2

        ar_balance = ledger.compute_account_balance("1100")
        rev_balance = ledger.compute_account_balance("4000")

        assert ar_balance["closing_balance"] == 5000000
        assert rev_balance["closing_balance"] == 5000000
        assert ar_balance["total_debit"] == rev_balance["total_credit"]

    def test_multi_line_entry_debits_equal_credits(self, ledger, fx_journal):
        result = ledger.post_journal_entry(fx_journal)
        assert result["posted_to_gl"] == 3

        all_balances = ledger.get_all_balances()
        total_debits = sum(b["total_debit"] for b in all_balances)
        total_credits = sum(b["total_credit"] for b in all_balances)
        assert abs(total_debits - total_credits) < 0.01


class TestLedgerPosting:
    def test_ledger_posting_updates_t_account(self, ledger, sales_journal):
        ledger.post_journal_entry(sales_journal)
        t_account = ledger.get_t_account("1100")

        assert t_account["account_code"] == "1100"
        assert t_account["account_type"] == "ASSET"
        assert t_account["balance"] == 5000000
        assert t_account["balance_type"] == "debit"
        assert len(t_account["debit_side"]) == 1
        assert t_account["debit_side"][0]["amount"] == 5000000

    def test_ledger_posting_credit_normal_account(self, ledger, purchase_journal):
        ledger.post_journal_entry(purchase_journal)
        ap_balance = ledger.compute_account_balance("2100")

        assert ap_balance["account_type"] == "LIABILITY"
        assert ap_balance["normal_balance"] == "credit"
        assert ap_balance["closing_balance"] == 3000000
        assert ap_balance["total_credit"] == 3000000

    def test_ledger_posting_multiple_entries(self, ledger, sales_journal, purchase_journal):
        ledger.post_journal_entry(sales_journal)
        ledger.post_journal_entry(purchase_journal)

        ar = ledger.compute_account_balance("1100")
        inv = ledger.compute_account_balance("1200")
        ap = ledger.compute_account_balance("2100")
        rev = ledger.compute_account_balance("4000")

        assert ar["closing_balance"] == 5000000
        assert inv["closing_balance"] == 3000000
        assert ap["closing_balance"] == 3000000
        assert rev["closing_balance"] == 5000000

    def test_sub_ledger_created_for_control_accounts(self, ledger, sales_journal):
        ledger.post_journal_entry(sales_journal)
        sub_ledger = ledger.get_sub_ledger("1100")

        assert sub_ledger["sub_ledger_type"] == "accounts_receivable"
        assert sub_ledger["entry_count"] == 1
        assert sub_ledger["total_balance"] == 5000000


class TestTrialBalanceExtraction:
    def test_trial_balance_debits_equal_credits(self, ledger, sales_journal, purchase_journal):
        ledger.post_journal_entry(sales_journal)
        ledger.post_journal_entry(purchase_journal)

        all_balances = ledger.get_all_balances()
        total_debit_balance = sum(b["closing_balance"] for b in all_balances if b["normal_balance"] == "debit")
        total_credit_balance = sum(b["closing_balance"] for b in all_balances if b["normal_balance"] == "credit")

        assert abs(total_debit_balance - total_credit_balance) < 0.01

    def test_trial_balance_by_account_type(self, ledger, sales_journal, purchase_journal):
        ledger.post_journal_entry(sales_journal)
        ledger.post_journal_entry(purchase_journal)

        assets = ledger.get_all_balances(account_type_filter="ASSET")
        liabilities = ledger.get_all_balances(account_type_filter="LIABILITY")
        revenue = ledger.get_all_balances(account_type_filter="REVENUE")

        assert all(a["account_type"] == "ASSET" for a in assets)
        assert all(l["account_type"] == "LIABILITY" for l in liabilities)
        assert all(r["account_type"] == "REVENUE" for r in revenue)


class TestLedgerReconciliation:
    def test_reconciled_accounts_match(self, ledger, sales_journal):
        ledger.post_journal_entry(sales_journal)
        reconciliation = ledger.reconcile_sub_ledger("1100")

        assert reconciliation["reconciled"] is True
        assert abs(reconciliation["difference"]) < 0.01
        assert reconciliation["gl_closing_balance"] == reconciliation["sub_ledger_closing_balance"]

    def test_reconciliation_non_control_account(self, ledger):
        reconciliation = ledger.reconcile_sub_ledger("4000")
        assert reconciliation["sub_ledger_type"] is None
        assert reconciliation["total_balance"] == 0.0


class TestMultiCurrencyLedger:
    def test_fx_transaction_maintains_balance(self, ledger, fx_journal):
        ledger.post_journal_entry(fx_journal)

        bank = ledger.compute_account_balance("1020")
        ar = ledger.compute_account_balance("1100")
        fx_gain = ledger.compute_account_balance("4100")

        assert bank["closing_balance"] == 155000000
        assert ar["closing_balance"] == -150000000
        assert fx_gain["closing_balance"] == 5000000

        net_asset_change = bank["closing_balance"] + ar["closing_balance"]
        net_liab_eq_change = fx_gain["closing_balance"]
        assert abs(net_asset_change - net_liab_eq_change) < 0.01

    @pytest.mark.parametrize("usd_amount,rate,base_amount", [
        (100000, 1500, 150000000),
        (50000, 1550, 77500000),
        (200000, 1600, 320000000),
    ])
    def test_fx_conversion_in_base_currency(self, ledger, usd_amount, rate, base_amount):
        fx_entry = {
            "id": f"JE-FX-{usd_amount}",
            "company_id": "HAQLY-001",
            "entry_date": "2025-03-01T00:00:00",
            "description": f"USD {usd_amount} settled at ₦{rate}/USD",
            "entry_number": f"JE-FX-{usd_amount}",
            "lines": [
                {"account_code": "1020", "description": "Bank", "debit": base_amount, "credit": 0.0},
                {"account_code": "1100", "description": "Accounts Receivable", "debit": 0.0, "credit": base_amount},
            ],
        }
        result = ledger.post_journal_entry(fx_entry)
        assert result["posted_to_gl"] == 2

        bank = ledger.compute_account_balance("1020")
        assert bank["total_debit"] == base_amount
