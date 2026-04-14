# HAQLY ERP — Commercialization Plan

**Author**: Quadri Atharu
**Version**: 1.0.0
**Last Updated**: 2026-04-14

---

## 1. Product Overview

- **Product**: HAQLY ERP Desktop
- **Stack**: Tauri (Desktop Shell) + Rust (Backend) + Next.js/React (Frontend) + Python (AI Engine)
- **Tagline**: "Enterprise Intelligence for Nigerian Businesses"
- **Unique Value**: First Nigerian-built ERP with FIRS e-invoicing, AI-powered tax advisory, and full Nigerian compliance out-of-the-box

---

## 2. Target Market

### Primary Segments
| Segment | Size | Description |
|---|---|---|
| Nigerian SMBs (1-50 employees) | ~41M registered SMEs | Need basic accounting + tax + e-invoicing |
| Mid-market enterprises (50-500 employees) | ~40,000 companies | Need multi-branch, BI, CRM, payroll |
| Accounting firms | ~5,000 firms | Need multi-company management, client portals |
| Government agencies | ~1,000 MDAs | Need compliance-specific, audit-ready, on-premise |

### Buyer Personas
1. **Chief Accountant Adebayo** — 45, manages 20-person finance team, frustrated with QuickBooks limitations, needs FIRS compliance
2. **Founder Chidinma** — 32, tech startup with 15 employees, wants automated tax and invoicing
3. **Audit Partner Emeka** — 52, manages 50 client companies, needs multi-tenant view with consolidated reporting
4. **Director General Musa** — 58, government MDA, needs on-premise deployment with CBN compliance

---

## 3. Pricing Tiers

### 3.1 Subscription Tiers

| Feature | Starter | Professional | Enterprise | Government |
|---|---|---|---|---|
| **Monthly Price** | ₦50,000 | ₦150,000 | ₦500,000 | Custom |
| **Annual Price** | ₦500,000 | ₦1,500,000 | ₦5,000,000 | Custom |
| **Users** | 5 | 20 | Unlimited | Unlimited |
| **Companies** | 1 | 3 | Unlimited | Unlimited |
| **Branches** | 1 | 5 | Unlimited | Unlimited |
| **Accounting** | Full | Full | Full | Full |
| **Tax Module** | VAT, PAYE, WHT | All tax types | All + AI advisory | All + custom |
| **E-Invoicing** | Basic | Full FIRS integration | Full + API access | Custom integration |
| **Payroll** | - | Full Nigerian payroll | Full + loan mgmt | Full + pension focus |
| **BI/Dashboards** | - | Basic KPI dashboards | Full BI suite | Custom dashboards |
| **CRM** | - | Basic | Full pipeline | Custom |
| **AI Intelligence** | - | - | Full AI agents | Custom |
| **OCR** | - | - | Included | Included |
| **Support** | Email (48h) | Email + Chat (24h) | Dedicated manager | On-site support |
| **Custom Integrations** | - | - | Available | Full custom dev |
| **Training** | Self-service | 2 sessions | 5 sessions + onboarding | Unlimited |

### 3.2 Add-On Services

| Add-On | Price | Unit |
|---|---|---|
| OCR Processing | ₦10,000/month | per 1,000 documents |
| AI Agent Packs | ₦25,000/month | per agent type |
| Premium Support | ₦75,000/month | dedicated engineer |
| Data Migration | ₦200,000 one-time | per company |
| Custom Report Development | ₦100,000 one-time | per report |
| API Access | ₦50,000/month | for external integrations |
| On-Premise Deployment | ₦1,500,000 one-time | + 20% annual maintenance |

---

## 4. Licensing Model

### 4.1 Subscription SaaS (Primary)
- Monthly or annual billing
- License key validated on startup and periodically (every 24 hours)
- Grace period: 7 days offline before features lock
- Auto-renewal with 30-day cancellation notice

### 4.2 Desktop Perpetual License (Secondary)
- One-time purchase: ₦2,000,000 (Enterprise tier equivalent)
- Annual maintenance: 20% of purchase price for updates + support
- License key: perpetual, tied to machine fingerprint
- Upgrade pricing: 50% of new tier price

### 4.3 License Key System
- **Algorithm**: RSA-2048 signed license keys
- **Key format**: `HAQLY-{TIER}-{FEATURE_FLAGS}-{USER_LIMIT}-{COMPANY_LIMIT}-{EXPIRY}-{SIGNATURE}`
- **Feature flags**: Bitmask encoding of enabled features
- **Validation**: Signature verification on startup + periodic online check
- **Offline mode**: 7-day grace period with cached validation
- **Machine binding**: Optional hardware fingerprint binding for perpetual licenses

---

## 5. Distribution Channels

### 5.1 Direct Sales
- Dedicated sales team (3 account executives initially)
- Inbound: website, SEO, content marketing
- Outbound: targeted outreach to accounting firms and mid-market companies

### 5.2 Accounting Firm Partnerships
- Reseller program: 20% commission on first-year subscription
- Training program for partner firms
- Co-branded marketing materials
- Target: 50 partner firms by Year 2

### 5.3 CBN-Registered Fintech Channels
- Integration partnerships with Nigerian fintechs
- Bundle deals with banking platforms
- Payment processor partnerships (Paystack, Flutterwave)

### 5.4 Government Procurement
- Registration on Bureau of Public Procurement (BPP) portal
- GEF (Government Enterprise Fund) eligibility
- State-level ICT agency partnerships

---

## 6. Revenue Projections

### Year 1 (2026)
| Segment | Customers | ARPU/month | Monthly Rev | Annual Rev |
|---|---|---|---|---|
| Starter | 100 | ₦50,000 | ₦5,000,000 | ₦60,000,000 |
| Professional | 30 | ₦150,000 | ₦4,500,000 | ₦54,000,000 |
| Enterprise | 5 | ₦500,000 | ₦2,500,000 | ₦30,000,000 |
| Add-ons | - | - | ₦1,500,000 | ₦18,000,000 |
| **Total Y1** | **135** | - | **₦13,500,000** | **₦162,000,000** |

### Year 2 (2027)
| Segment | Customers | ARPU/month | Monthly Rev | Annual Rev |
|---|---|---|---|---|
| Starter | 300 | ₦50,000 | ₦15,000,000 | ₦180,000,000 |
| Professional | 80 | ₦150,000 | ₦12,000,000 | ₦144,000,000 |
| Enterprise | 15 | ₦500,000 | ₦7,500,000 | ₦90,000,000 |
| Government | 3 | ₦750,000 | ₦2,250,000 | ₦27,000,000 |
| Add-ons | - | - | ₦5,000,000 | ₦60,000,000 |
| **Total Y2** | **398** | - | **₦41,750,000** | **₦501,000,000** |

### Year 3 (2028)
| Segment | Customers | ARPU/month | Monthly Rev | Annual Rev |
|---|---|---|---|---|
| Starter | 700 | ₦55,000 | ₦38,500,000 | ₦462,000,000 |
| Professional | 200 | ₦165,000 | ₦33,000,000 | ₦396,000,000 |
| Enterprise | 40 | ₦550,000 | ₦22,000,000 | ₦264,000,000 |
| Government | 10 | ₦800,000 | ₦8,000,000 | ₦96,000,000 |
| Add-ons | - | - | ₦12,000,000 | ₦144,000,000 |
| **Total Y3** | **950** | - | **₦113,500,000** | **₦1,362,000,000** |

---

## 7. Competitive Positioning

### vs Sage 50/200
| Factor | HAQLY ERP | Sage |
|---|---|---|
| Nigerian Tax Compliance | Native (all tax types + e-invoicing) | Partial (requires add-ons) |
| FIRS E-Invoicing | Built-in NRS integration | Not available |
| AI Intelligence | Native AI agents | Not available |
| Pricing | ₦50K-500K/month | ₦100K-1M/month + implementation |
| Deployment | Desktop + optional cloud | Cloud only or expensive on-premise |
| Nigerian Payroll | Full (PAYE, NHF, NSITF, ITF, Pension) | Basic PAYE only |
| Local Support | Nigeria-based team | Limited Nigerian support |

### vs QuickBooks
| Factor | HAQLY ERP | QuickBooks |
|---|---|---|
| Nigerian Accounting Standards | Full compliance | US-centric |
| Multi-branch | Native | Limited |
| Tax Module | Native Nigerian tax | Basic VAT only |
| E-Invoicing | FIRS NRS | Not available |
| Payroll | Full Nigerian payroll | Limited |
| Desktop Offline | Full functionality | Online only |
| Data Sovereignty | Local storage | US servers |

### vs Xero
| Factor | HAQLY ERP | Xero |
|---|---|---|
| E-Invoicing | FIRS NRS | Not available |
| Nigerian Payroll | Full | Not available |
| AI Features | Native | Limited |
| Offline | Desktop native | Cloud only |
| Pricing (Nigeria) | ₦ NGN pricing | USD pricing (expensive) |

### vs Zoho Books
| Factor | HAQLY ERP | Zoho Books |
|---|---|---|
| E-Invoicing | FIRS NRS | Not available |
| Desktop | Tauri native | Browser only |
| AI Intelligence | Native | Basic |
| Nigerian Compliance | Full | Partial |

### vs SAP Business One
| Factor | HAQLY ERP | SAP B1 |
|---|---|---|
| Pricing | ₦50K-500K/month | ₦2M+/month |
| Implementation | Self-service + minimal setup | 6-12 month implementation |
| Nigerian Compliance | Native | Requires customization |
| Target Market | SMB to mid-market | Mid-market to enterprise |
| Speed to Value | Days | Months |

---

## 8. Go-to-Market Strategy

### Phase 1: Launch (Months 1-6)
- Target: 50 Starter, 10 Professional, 2 Enterprise customers
- Tactics: Free trial (14 days), accounting firm demos, LinkedIn marketing
- Key metric: Customer acquisition cost < ₦50,000

### Phase 2: Growth (Months 7-18)
- Target: 200 Starter, 50 Professional, 10 Enterprise customers
- Tactics: Partner program launch, content marketing, industry events
- Key metric: Monthly recurring revenue > ₦10M

### Phase 3: Scale (Months 19-36)
- Target: 500+ total customers, government contracts
- Tactics: Government procurement, API ecosystem, international expansion (Ghana, Kenya)
- Key metric: Annual recurring revenue > ₦500M

---

## 9. Key Metrics & KPIs

| Metric | Year 1 Target | Year 2 Target | Year 3 Target |
|---|---|---|---|
| MRR | ₦13.5M | ₦41.75M | ₦113.5M |
| ARR | ₦162M | ₦501M | ₦1.36B |
| Customers | 135 | 398 | 950 |
| Churn Rate | <5%/month | <3%/month | <2%/month |
| NPS | >40 | >50 | >60 |
| CAC | <₦50K | <₦40K | <₦30K |
| LTV:CAC | >3:1 | >5:1 | >8:1 |
| Gross Margin | >70% | >75% | >80% |
