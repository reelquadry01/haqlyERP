# HAQLY ERP — AI Intelligence Engine Architecture

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

The AI Intelligence Engine is a Python sidecar that provides machine learning and LLM-powered capabilities to HAQLY ERP. It runs as an independent FastAPI service on port 8200, communicating with the main Axum backend via REST. The engine supports document classification, OCR-enhanced extraction, anomaly detection, predictive cash flow analysis, and conversational AI assistance.

---

## 2. Architecture

```
┌─────────────────────────────────────────────┐
│              AI Engine (Python)              │
│                                             │
│  ┌───────────┐  ┌───────────┐  ┌──────────┐│
│  │  Module    │  │  Module   │  │  Module  ││
│  │ Registry   │  │ Router    │  │ Config   ││
│  └─────┬─────┘  └─────┬─────┘  └────┬─────┘│
│        │               │              │      │
│  ┌─────▼───────────────▼──────────────▼───┐ │
│  │          Agent Framework               │ │
│  │  ┌────────┐ ┌────────┐ ┌────────────┐ │ │
│  │  │ Classi │ │  Anoma │ │  Predicti  │ │ │
│  │  │ fier   │ │  ly    │ │  ve        │ │ │
│  │  │ Agent  │ │  Agent │ │  Agent     │ │ │
│  │  └────────┘ └────────┘ └────────────┘ │ │
│  │  ┌────────┐ ┌────────┐ ┌────────────┐ │ │
│  │  │ Extra  │ │  Cash  │ │  Conver   │ │ │
│  │  │ ction  │ │  Flow  │ │  sational │ │ │
│  │  │ Agent  │ │  Agent │ │  Agent     │ │ │
│  │  └────────┘ └────────┘ └────────────┘ │ │
│  └────────────────────────────────────────┘ │
│                                             │
│  ┌─────────────────────────────────────────┐│
│  │         Dataset Management              ││
│  │  ┌──────────┐ ┌───────────┐ ┌────────┐││
│  │  │ Training │ │ Validation│ │  Featu  │││
│  │  │ Data     │ │   Sets    │ │  re     │││
│  │  └──────────┘ └───────────┘ │  Store  │││
│  │                             └────────┘││
│  └─────────────────────────────────────────┘│
│                                             │
│  ┌─────────────────────────────────────────┐│
│  │      Dynamic Update Framework           ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐││
│  │  │ Model    │ │  Hot     │ │  Rollbac │││
│  │  │ Versioni │ │  Reload  │ │  k       │││
│  │  └──────────┘ └──────────┘ └──────────┘││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
        │
        │ REST (port 8200)
        ▼
┌──────────────────┐
│  Axum Backend    │
│  (port 8100)     │
└──────────────────┘
```

---

## 3. Module Registry

The module registry tracks all available AI capabilities, their health status, and configuration.

```python
class ModuleConfig:
    id: str
    name: str
    version: str
    enabled: bool
    model_path: str | None
    dependencies: list[str]
    health_check_url: str
    config: dict

class ModuleRegistry:
    _modules: dict[str, ModuleConfig]

    def register(self, module: ModuleConfig) -> None
    def unregister(self, module_id: str) -> None
    def get(self, module_id: str) -> ModuleConfig | None
    def list_enabled(self) -> list[ModuleConfig]
    def health_check(self, module_id: str) -> dict
    def health_check_all(self) -> dict[str, dict]
```

**API:**
| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/ai/modules` | List all registered modules |
| GET | `/api/v1/ai/modules/{id}` | Get module details |
| GET | `/api/v1/ai/modules/{id}/health` | Health check |
| PATCH | `/api/v1/ai/modules/{id}` | Enable/disable module |
| POST | `/api/v1/ai/modules/{id}/reload` | Hot-reload module |

---

## 4. Agent Framework

### 4.1 Base Agent

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass

@dataclass
class AgentResult:
    success: bool
    data: dict
    confidence: float
    model_used: str
    tokens_used: int
    processing_time_ms: int
    errors: list[str]

class BaseAgent(ABC):
    agent_id: str
    agent_name: str
    version: str

    @abstractmethod
    async def execute(self, input_data: dict, context: dict) -> AgentResult

    @abstractmethod
    async def validate_input(self, input_data: dict) -> bool

    async def health_check(self) -> dict:
        return {"agent_id": self.agent_id, "status": "healthy"}
```

### 4.2 Registered Agents

| Agent ID | Name | Purpose |
|---|---|---|
| `classifier` | Document Classifier | Classifies uploaded documents by type |
| `extractor` | Data Extractor | Extracts structured data from OCR text |
| `anomaly` | Anomaly Detector | Flags unusual transactions, duplicates, amount outliers |
| `predictive` | Cash Flow Predictor | Forecasts cash position using historical patterns |
| `conversational` | AI Assistant | Natural language Q&A over financial data |
| `reconciliation` | Auto Reconciler | Suggests matches for bank reconciliation |
| `compliance` | Compliance Checker | Validates transactions against tax/NRS rules |

### 4.3 Classifier Agent

Input:
```json
{
  "text": "raw OCR text from document",
  "companyId": "uuid",
  "hint": null
}
```

Output:
```json
{
  "success": true,
  "data": {
    "docType": "sales_invoice",
    "confidence": 0.97,
    "alternatives": [
      {"docType": "purchase_bill", "confidence": 0.02}
    ]
  },
  "confidence": 0.97,
  "model_used": "distilbert-financial-ng-v1",
  "tokens_used": 0,
  "processing_time_ms": 45
}
```

### 4.4 Anomaly Detector Agent

Input:
```json
{
  "transactions": [
    {
      "id": "uuid",
      "date": "2026-04-13",
      "amount": 5000000.00,
      "accountId": "uuid",
      "description": "Consulting fee"
    }
  ],
  "companyId": "uuid",
  "lookbackDays": 90
}
```

Output:
```json
{
  "success": true,
  "data": {
    "anomalies": [
      {
        "transactionId": "uuid",
        "type": "amount_outlier",
        "severity": "high",
        "score": 0.89,
        "explanation": "Amount ₦5,000,000 is 4.2 standard deviations above the mean for this account over 90 days",
        "suggestion": "Verify this transaction with the approving manager"
      }
    ]
  },
  "confidence": 0.89,
  "model_used": "isolation-forest-v1",
  "tokens_used": 0,
  "processing_time_ms": 120
}
```

### 4.5 Predictive Agent (Cash Flow)

Input:
```json
{
  "companyId": "uuid",
  "forecastDays": 30,
  "includeReceivables": true,
  "includePayables": true,
  "confidence": 0.95
}
```

Output:
```json
{
  "success": true,
  "data": {
    "forecastDate": "2026-05-13",
    "predictedBalance": 12500000.00,
    "confidenceInterval": {
      "low": 9500000.00,
      "high": 15500000.00
    },
    "dailyForecast": [
      {"date": "2026-04-14", "predictedBalance": 12200000.00, "confidence": 0.98},
      {"date": "2026-04-15", "predictedBalance": 12100000.00, "confidence": 0.97}
    ],
    "riskFactors": [
      "3 large payables due in next 7 days totaling ₦4,200,000",
      "Receivables collection rate has dropped 15% this month"
    ]
  },
  "confidence": 0.95,
  "model_used": "prophet-cashflow-v1",
  "tokens_used": 0,
  "processing_time_ms": 3500
}
```

### 4.6 Conversational Agent

Input:
```json
{
  "query": "What was our total revenue last quarter?",
  "companyId": "uuid",
  "conversationId": "uuid-or-null",
  "userPermissions": ["accounting:view", "reports:view"]
}
```

Output:
```json
{
  "success": true,
  "data": {
    "response": "Total revenue for Q1 2026 was ₦45,200,000, which is a 12% increase from Q4 2025 (₦40,357,000). The top revenue sources were: Consulting Services (₦28M), Software Licenses (₦12M), and Training (₦5.2M).",
    "sources": [
      {"type": "journal_entry", "id": "uuid", "description": "Q1 Revenue Summary"}
    ],
    "suggestedFollowups": [
      "Show me the revenue breakdown by customer",
      "Compare Q1 2026 vs Q1 2025"
    ]
  },
  "confidence": 0.92,
  "model_used": "gpt-4o-mini",
  "tokens_used": 1450,
  "processing_time_ms": 2800
}
```

---

## 5. Dataset Management

### 5.1 Dataset Registry

```python
class Dataset:
    id: str
    name: str
    version: str
    description: str
    record_count: int
    created_at: datetime
    updated_at: datetime
    tags: list[str]
```

### 5.2 Dataset Types

| Dataset | Purpose | Update Frequency |
|---|---|---|
| `nigerian_invoices` | Training data for document classifier | Weekly |
| `bank_statements_ng` | Bank statement parsing training | Monthly |
| `receipts_ng` | Receipt extraction training | Weekly |
| `transaction_patterns` | Anomaly detection baseline | Daily |
| `cash_flow_history` | Cash flow prediction training | Daily |
| `tax_rules_ng` | Nigerian tax compliance rules | Quarterly (FIRS updates) |
| `nrs_schemas` | E-invoicing validation schemas | As regulations change |

### 5.3 Dataset API

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/ai/datasets` | List all datasets |
| GET | `/api/v1/ai/datasets/{id}` | Get dataset details |
| POST | `/api/v1/ai/datasets/{id}/refresh` | Trigger dataset refresh |
| GET | `/api/v1/ai/datasets/{id}/stats` | Get dataset statistics |

### 5.4 Feature Store

Key-value store for computed features used by multiple agents:

```sql
CREATE TABLE ai_features (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id      UUID NOT NULL,
    entity_type     VARCHAR(50) NOT NULL,
    entity_id       UUID NOT NULL,
    feature_name    VARCHAR(100) NOT NULL,
    feature_value   JSONB NOT NULL,
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ,
    UNIQUE(company_id, entity_type, entity_id, feature_name)
);
```

---

## 6. Dynamic Update Framework

### 6.1 Model Versioning

Every model artifact is versioned:
```
models/
├── classifier/
│   ├── v1.0.0/
│   │   ├── model.onnx
│   │   ├── tokenizer.json
│   │   └── metadata.json
│   └── v1.1.0/
│       ├── model.onnx
│       ├── tokenizer.json
│       └── metadata.json
├── anomaly/
│   └── v1.0.0/
│       ├── model.pkl
│       └── metadata.json
└── cashflow/
    └── v1.0.0/
        ├── model.pkl
        └── metadata.json
```

`metadata.json`:
```json
{
  "version": "1.1.0",
  "modelType": "distilbert",
  "trainedOn": "2026-03-15",
  "accuracy": 0.973,
  "f1Score": 0.968,
  "datasetVersion": "nigerian_invoices_v3",
  "changelog": "Fine-tuned on credit notes and delivery notes"
}
```

### 6.2 Hot Reload

Models can be reloaded without restarting the sidecar:

```python
class ModelManager:
    _active_models: dict[str, str]  # module_id → version

    async def load_model(self, module_id: str, version: str) -> bool
    async def reload_model(self, module_id: str) -> bool
    async def rollback_model(self, module_id: str, version: str) -> bool
    def get_active_version(self, module_id: str) -> str
```

**Hot Reload Flow:**
1. Admin uploads new model artifact to `~/.haqly/ai/models/{module}/{version}/`.
2. Admin calls `POST /api/v1/ai/modules/{id}/reload` with `version` parameter.
3. The ModelManager loads the new model into memory.
4. A health check runs against a validation set.
5. If health check passes, the new model becomes active.
6. If health check fails, the previous model is restored and an error is returned.

### 6.3 Rollback

If a newly deployed model causes issues:
1. Call `POST /api/v1/ai/modules/{id}/rollback` with the target `version`.
2. The ModelManager unloads the current model and loads the specified version.
3. The rollback is recorded in the audit log.

---

## 7. Axum Proxy Endpoints

The Axum backend proxies AI engine requests through its own API to maintain the single-origin pattern:

| Method | Path | Proxies To |
|---|---|---|
| POST | `/api/v1/ai/analyze` | Sidecar `/api/v1/ai/agents/{agentId}/execute` |
| POST | `/api/v1/ai/classify` | Sidecar `/api/v1/classify` |
| POST | `/api/v1/ai/extract` | Sidecar `/api/v1/extract` |
| GET | `/api/v1/ai/insights` | Sidecar `/api/v1/ai/insights/{companyId}` |
| POST | `/api/v1/ai/chat` | Sidecar `/api/v1/ai/agents/conversational/execute` |
| GET | `/api/v1/ai/modules` | Sidecar `/api/v1/ai/modules` |

All proxy endpoints add the JWT auth context and company_id before forwarding to the sidecar.

---

## 8. Configuration

| Variable | Default | Description |
|---|---|---|
| `AI_SIDECAR_URL` | `http://localhost:8200` | Sidecar base URL |
| `AI_LLM_PROVIDER` | `openai` | LLM provider: `openai`, `ollama`, `azure` |
| `AI_LLM_MODEL` | `gpt-4o-mini` | Default LLM model |
| `AI_LLM_API_KEY` | — | API key for cloud LLM |
| `AI_LLM_BASE_URL` | — | Custom LLM endpoint (for Ollama) |
| `AI_MODELS_DIR` | `~/.haqly/ai/models` | Model artifacts directory |
| `AI_DATASETS_DIR` | `~/.haqly/ai/datasets` | Dataset storage directory |
| `AI_MAX_TOKENS` | `4096` | Maximum LLM tokens per request |
| `AI_TIMEOUT_SECONDS` | `30` | Request timeout for LLM calls |
| `AI_LOG_LEVEL` | `info` | Sidecar logging level |
