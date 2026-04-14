# Author: Quadri Atharu
"""Pytest tests for AI agents — base, journal, tax, reporting, finance, and audit agents."""

import pytest

from ..agents.base_agent import BaseAgent
from ..agents.journal_agent import JournalAgent
from ..agents.tax_agent import TaxAgent
from ..agents.reporting_agent import ReportingAgent
from ..agents.finance_agent import FinanceAgent
from ..agents.audit_agent import AuditAgent


@pytest.fixture
def base_agent():
    return BaseAgent()


@pytest.fixture
def journal_agent():
    return JournalAgent()


@pytest.fixture
def tax_agent():
    return TaxAgent()


@pytest.fixture
def reporting_agent():
    return ReportingAgent()


@pytest.fixture
def finance_agent():
    return FinanceAgent()


@pytest.fixture
def audit_agent():
    return AuditAgent()


class TestBaseAgentInitialization:
    def test_base_agent_initialization(self, base_agent):
        assert base_agent.agent_name == "base_agent"
        assert base_agent.list_skills() == []
        assert base_agent.health_check() is True
        log = base_agent.get_activity_log()
        assert isinstance(log, list)
        assert len(log) == 0

    def test_base_agent_skill_registration(self, base_agent):
        base_agent.register_skill("test_skill", lambda d: {"result": d.get("value", 0)})
        assert "test_skill" in base_agent.list_skills()

        result = base_agent.execute("test_skill", {"value": 50000000})
        assert result["result"] == 50000000
        assert result["_meta"]["success"] is True
        assert result["_meta"]["agent"] == "base_agent"

    def test_base_agent_unknown_skill_returns_error(self, base_agent):
        result = base_agent.execute("nonexistent", {})
        assert result["success"] is False
        assert "not found" in result["error"]["message"].lower()


class TestJournalAgent:
    def test_journal_agent_entry_suggestion(self, journal_agent):
        result = journal_agent.execute("suggest_accounts", {
            "transaction_type": "SALES",
            "description": "Sale of goods to customer",
        })
        assert result["success"] is True
        assert "suggestion" in result

    def test_journal_agent_validation(self, journal_agent):
        result = journal_agent.execute("validate_journal", {
            "lines": [
                {"account_code": "1100", "description": "AR", "debit": 100000000, "credit": 0},
                {"account_code": "4000", "description": "Revenue", "debit": 0, "credit": 100000000},
            ],
        })
        assert result["valid"] is True
        assert result["balanced"] is True
        assert result["total_debit"] == 100000000

    def test_journal_agent_validation_unbalanced(self, journal_agent):
        result = journal_agent.execute("validate_journal", {
            "lines": [
                {"account_code": "1100", "description": "AR", "debit": 100000000, "credit": 0},
                {"account_code": "4000", "description": "Revenue", "debit": 0, "credit": 80000000},
            ],
        })
        assert result["valid"] is False
        assert result["balanced"] is False
        assert len(result["errors"]) > 0

    def test_journal_agent_process_transaction(self, journal_agent):
        result = journal_agent.execute("process_transaction", {
            "transaction_type": "SALES",
            "amount": 250000000,
            "description": "Cement sales to Dangote",
            "company_id": "HAQLY-001",
            "debit_account_code": "1100",
            "credit_account_code": "4000",
        })
        assert result["success"] is True
        assert result["total_debit"] == 250000000
        assert result["status"] == "DRAFT"


class TestTaxAgentComputation:
    def test_tax_agent_computation(self, tax_agent):
        result = tax_agent.execute("compute_all_taxes", {
            "vat": {"output_vat": 93750000, "input_vat": 56250000},
            "cit": {"profit_before_tax": 1500000000, "turnover": 5000000000},
        })
        assert result["success"] is True
        assert "vat" in result["taxes"]
        assert "cit" in result["taxes"]
        assert result["total_tax_liability"] > 0

    def test_tax_agent_vat_only(self, tax_agent):
        result = tax_agent.execute("compute_all_taxes", {
            "vat": {"output_vat": 75000000, "input_vat": 30000000},
        })
        assert result["success"] is True
        assert "vat" in result["taxes"]

    def test_tax_agent_risk_detection(self, tax_agent):
        result = tax_agent.execute("detect_risks", {
            "tax_data": {"vat_ratio": 0.01, "profit_margin": 0.80},
        })
        assert result["success"] is True
        assert "risk_assessment" in result


class TestReportingAgent:
    def test_reporting_agent_statement_generation(self, reporting_agent):
        assert reporting_agent.agent_name == "reporting_agent"
        skills = reporting_agent.list_skills()
        assert "generate_income_statement" in skills
        assert "generate_balance_sheet" in skills
        assert "generate_cash_flow" in skills
        assert "generate_ratio_analysis" in skills


class TestFinanceAgent:
    def test_finance_agent_analysis(self, finance_agent):
        assert finance_agent.agent_name == "finance_agent"
        skills = finance_agent.list_skills()
        assert "analyze_financials" in skills
        assert "compute_valuation" in skills
        assert "assess_risk" in skills
        assert "forecast" in skills

    def test_finance_agent_forecast(self, finance_agent):
        result = finance_agent.execute("forecast", {
            "company_id": "HAQLY-001",
            "historical_revenue": [2000000000, 2200000000, 2500000000],
            "historical_expenses": [1500000000, 1600000000, 1800000000],
            "periods_ahead": 3,
            "method": "linear",
        })
        assert result["success"] is True
        assert len(result["profit_forecast"]) == 3
        assert result["method"] == "linear"


class TestAuditAgent:
    def test_audit_agent_anomaly_detection(self, audit_agent):
        assert audit_agent.agent_name == "audit_agent"
        skills = audit_agent.list_skills()
        assert "detect_exceptions" in skills
        assert "sample_transactions" in skills
        assert "generate_audit_trail" in skills

    def test_audit_agent_exception_detection(self, audit_agent):
        amounts = [100000, 200000, 300000, 150000, 250000, 100000, 5000000, 180000, 220000, 160000]
        result = audit_agent.execute("detect_exceptions", {
            "method": "benford",
            "amounts": amounts,
        })
        assert result["success"] is True
        assert "benford" in result["results"]
