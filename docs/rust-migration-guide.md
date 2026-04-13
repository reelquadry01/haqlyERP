# HAQLY ERP — NestJS → Rust Migration Guide

**Author:** Quadri Atharu  
**Version:** 0.1.0  
**Date:** 2026-04-13

---

## 1. Overview

This guide maps the existing HaqlyERP NestJS/TypeScript backend to its Rust (Axum) equivalent. Each section covers module mapping, DTO translation, service patterns, and framework-specific idioms.

---

## 2. Framework Mapping

| NestJS Concept | Rust/Axum Equivalent |
|---|---|
| `@Module` | Module directory with `mod.rs` |
| `@Controller` | `Router` function returning `axum::Router` |
| `@Injectable` (Service) | Struct with `impl` methods, constructed via `Arc` |
| `@Inject` | Function parameters (dependency injection via state) |
| `@Body()` | `Json<T>` extractor |
| `@Param()` | `Path<T>` extractor |
| `@Query()` | `Query<T>` extractor |
| `@UseGuards()` | Middleware layer or `Extension<T>` extractor |
| `DTO class` | `serde::Deserialize` / `serde::Serialize` struct |
| `class-validator` | `validator::Validate` derive + custom validations |
| `TypeORM Entity` | `sqlx::FromRow` struct + SQL migration |
| `TypeORM Repository` | Custom repository functions using `sqlx::PgPool` |
| `Express Request/Response` | `axum::extract::Request` / `axum::response::Response` |
| `HttpException` | Custom error enum implementing `IntoResponse` |
| `AuthGuard` | `auth::AuthMiddleware` layer |
| `Passport.js strategy` | JWT verification in middleware |
| `ConfigModule` | `dotenv` + `std::env` + `config` crate |
| `EventEmitter` | `tokio::sync::broadcast` or `tokio::mpsc` |
| `Bull Queue` | `tokio::task::spawn` + `sqlx` queue table |
| `Logger (winston)` | `tracing` + `tracing-subscriber` |
| `jest` | `#[tokio::test]` + `sqlx::test` |

---

## 3. Project Structure Mapping

### NestJS
```
src/
├── auth/
│   ├── auth.module.ts
│   ├── auth.controller.ts
│   ├── auth.service.ts
│   ├── dto/
│   │   ├── login.dto.ts
│   │   └── mfa-verify.dto.ts
│   ├── guards/
│   │   ├── jwt-auth.guard.ts
│   │   └── roles.guard.ts
│   └── strategies/
│       └── jwt.strategy.ts
├── accounting/
│   ├── accounting.module.ts
│   ├── accounting.controller.ts
│   ├── accounting.service.ts
│   ├── dto/
│   └── entities/
└── ...
```

### Rust
```
src/
├── modules/
│   ├── auth/
│   │   ├── mod.rs              # Module registration, router
│   │   ├── handlers.rs         # Route handler functions
│   │   ├── service.rs          # Business logic
│   │   ├── dto.rs              # Request/response types
│   │   └── repository.rs       # Database queries
│   ├── accounting/
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── service.rs
│   │   ├── dto.rs
│   │   └── repository.rs
│   └── ...
├── middleware/
│   ├── auth.rs
│   ├── rbac.rs
│   └── company.rs
├── shared/
│   ├── error.rs
│   ├── types.rs
│   ├── currency.rs
│   └── dates.rs
└── main.rs
```

---

## 4. DTO Mapping

### 4.1 Login DTO

**TypeScript (NestJS):**
```typescript
export class LoginDto {
  @IsEmail()
  email: string;

  @IsString()
  @MinLength(8)
  password: string;
}
```

**Rust (Axum):**
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}
```

### 4.2 Create Journal Entry DTO

**TypeScript:**
```typescript
export class CreateJournalEntryDto {
  @IsDateString()
  entryDate: string;

  @IsString()
  description: string;

  @IsArray()
  @ValidateNested({ each: true })
  lines: CreateJournalLineDto[];
}

export class CreateJournalLineDto {
  @IsUUID()
  accountId: string;

  @IsNumber()
  debit: number;

  @IsNumber()
  credit: number;

  @IsOptional()
  @IsString()
  description?: string;
}
```

**Rust:**
```rust
use serde::Deserialize;
use validator::Validate;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Deserialize, Validate)]
pub struct CreateJournalEntryRequest {
    pub entry_date: chrono::NaiveDate,
    pub description: String,
    #[validate(length(min = 2))]
    pub lines: Vec<CreateJournalLineRequest>,
}

#[derive(Deserialize, Validate)]
pub struct CreateJournalLineRequest {
    pub account_id: Uuid,
    #[validate(range(min = 0))]
    pub debit: Decimal,
    #[validate(range(min = 0))]
    pub credit: Decimal,
    pub description: Option<String>,
}
```

### 4.3 Response DTOs

**TypeScript:**
```typescript
export class JournalEntryResponse {
  id: string;
  entryNumber: string;
  entryDate: string;
  status: string;
  totalDebit: number;
  totalCredit: number;
  lines: JournalLineResponse[];
}
```

**Rust:**
```rust
use serde::Serialize;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Serialize)]
pub struct JournalEntryResponse {
    pub id: Uuid,
    pub entry_number: String,
    pub entry_date: chrono::NaiveDate,
    pub status: String,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub lines: Vec<JournalLineResponse>,
}

#[derive(Serialize)]
pub struct JournalLineResponse {
    pub id: Uuid,
    pub line_number: i32,
    pub account_id: Uuid,
    pub account_name: String,
    pub debit: Decimal,
    pub credit: Decimal,
    pub description: Option<String>,
}
```

---

## 5. Service Pattern Mapping

### 5.1 NestJS Service

```typescript
@Injectable()
export class JournalService {
  constructor(
    @InjectRepository(JournalEntry)
    private entryRepo: Repository<JournalEntry>,
    @InjectRepository(JournalLine)
    private lineRepo: Repository<JournalLine>,
  ) {}

  async create(dto: CreateJournalEntryDto, companyId: string): Promise<JournalEntry> {
    const entry = this.entryRepo.create({ ...dto, companyId });
    return this.entryRepo.save(entry);
  }

  async findById(id: string, companyId: string): Promise<JournalEntry> {
    const entry = await this.entryRepo.findOne({ where: { id, companyId } });
    if (!entry) throw new NotFoundException();
    return entry;
  }
}
```

### 5.2 Rust Service

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::shared::error::AppError;

pub struct JournalService;

impl JournalService {
    pub async fn create(
        pool: &PgPool,
        dto: CreateJournalEntryRequest,
        company_id: Uuid,
    ) -> Result<JournalEntryResponse, AppError> {
        let entry = sqlx::query_as!(
            JournalEntryRow,
            r#"INSERT INTO journal_entries (company_id, entry_date, description, status)
               VALUES ($1, $2, $3, 'draft')
               RETURNING id, entry_number, entry_date, status, total_debit, total_credit"#,
            company_id, dto.entry_date, dto.description,
        )
        .fetch_one(pool)
        .await?;

        // Insert lines...
        Ok(entry.into_response())
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
        company_id: Uuid,
    ) -> Result<JournalEntryResponse, AppError> {
        let entry = sqlx::query_as!(
            JournalEntryRow,
            r#"SELECT id, entry_number, entry_date, status, total_debit, total_credit
               FROM journal_entries
               WHERE id = $1 AND company_id = $2 AND deleted_at IS NULL"#,
            id, company_id,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Journal entry not found".into()))?;

        Ok(entry.into_response())
    }
}
```

---

## 6. Handler/Controller Mapping

### 6.1 NestJS Controller

```typescript
@Controller('journal-entries')
@UseGuards(JwtAuthGuard, RolesGuard)
export class JournalController {
  constructor(private journalService: JournalService) {}

  @Post()
  @Permissions('journal:create')
  async create(@Body() dto: CreateJournalEntryDto, @Req() req) {
    return this.journalService.create(dto, req.user.companyId);
  }

  @Get(':id')
  @Permissions('journal:view')
  async findById(@Param('id') id: string, @Req() req) {
    return this.journalService.findById(id, req.user.companyId);
  }
}
```

### 6.2 Rust Handler

```rust
use axum::{
    extract::{Path, State, Extension},
    Json,
    routing::{get, post},
    Router,
};
use crate::middleware::auth::AuthUser;
use crate::shared::error::AppError;

pub fn journal_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_entry))
        .route("/:id", get(get_entry))
}

async fn create_entry(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateJournalEntryRequest>,
) -> Result<Json<JournalEntryResponse>, AppError> {
    let entry = JournalService::create(&state.pool, dto, auth_user.company_id).await?;
    Ok(Json(entry))
}

async fn get_entry(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<JournalEntryResponse>, AppError> {
    let entry = JournalService::find_by_id(&state.pool, id, auth_user.company_id).await?;
    Ok(Json(entry))
}
```

---

## 7. Error Handling Mapping

### 7.1 NestJS Exception Filter

```typescript
@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost) {
    const status = exception instanceof HttpException
      ? exception.getStatus() : 500;
    const message = exception instanceof HttpException
      ? exception.message : 'Internal server error';
    // ...
  }
}
```

### 7.2 Rust Error Enum

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
    Validation(Vec<validator::ValidationErrors>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Validation(errors) => {
                let msg = errors.iter()
                    .flat_map(|(_, e)| e.errors().iter())
                    .map(|(field, errs)| format!("{}: {}", field, errs[0]))
                    .collect::<Vec<_>>()
                    .join(", ");
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "VALIDATION_ERROR",
                    "message": msg,
                }))).into_response();
            }
        };

        (status, Json(serde_json::json!({
            "error": status.canonical_reason().unwrap_or("ERROR"),
            "message": message,
        }))).into_response()
    }
}
```

---

## 8. Key Crate Dependencies

| Purpose | NestJS Package | Rust Crate |
|---|---|---|
| HTTP Framework | `@nestjs/core` + `express` | `axum` |
| Async Runtime | Node.js event loop | `tokio` |
| Serialization | `class-transformer` | `serde` + `serde_json` |
| Validation | `class-validator` | `validator` |
| Database | `typeorm` + `pg` | `sqlx` (postgres) |
| UUID | `uuid` (npm) | `uuid` |
| Date/Time | `date-fns` / `moment` | `chrono` |
| Decimal | `decimal.js` | `rust_decimal` |
| JWT | `jsonwebtoken` | `jsonwebtoken` |
| Password Hashing | `bcrypt` | `argon2` |
| Logging | `winston` / `nestjs-pino` | `tracing` + `tracing-subscriber` |
| CORS | `@nestjs/cors` | `tower-http` (cors) |
| Env Config | `@nestjs/config` | `dotenv` + `std::env` |
| HTTP Client | `axios` | `reqwest` |
| PDF Generation | `pdfkit` | Server-side: `genpdf` |
| QR Code | `qrcode` (npm) | `qrcode` (crate) |

---

## 9. Migration Checklist Per Module

For each module, follow this checklist:

1. [ ] Create module directory under `src/modules/{module}/`
2. [ ] Port DTO classes → `dto.rs` with `Serialize`/`Deserialize`/`Validate`
3. [ ] Port Entity → `sqlx::FromRow` struct + write SQL migration
4. [ ] Port Repository → `repository.rs` with `sqlx` queries
5. [ ] Port Service → `service.rs` with business logic
6. [ ] Port Controller → `handlers.rs` with Axum route handlers
7. [ ] Register router in `mod.rs`
8. [ ] Wire router into `main.rs` application router
9. [ ] Port unit tests → `#[tokio::test]` with `sqlx::test`
10. [ ] Verify all endpoints match the existing API contract

---

## 10. Common Patterns

### 10.1 Pagination

**NestJS:**
```typescript
async findAll(query: PaginationDto): Promise<PaginatedResult<T>> {
  const [data, total] = await this.repo.findAndCount({
    skip: (query.page - 1) * query.limit,
    take: query.limit,
  });
  return { data, total, page: query.page, limit: query.limit };
}
```

**Rust:**
```rust
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

pub async fn paginate<T: for<'r> FromRow<'r, PgRow>>(
    pool: &PgPool,
    query: &str,
    count_query: &str,
    page: i32,
    limit: i32,
) -> Result<PaginatedResult<T>, AppError> {
    let offset = (page - 1) * limit;
    let total: (i64,) = sqlx::query_as(count_query).fetch_one(pool).await?;
    let data = sqlx::query_as::<_, T>(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(PaginatedResult { data, total: total.0, page, limit })
}
```

### 10.2 Company Context Filtering

Every query MUST include `WHERE company_id = $1`. The `AuthUser` extractor guarantees this is always available:

```rust
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub permissions: Vec<String>,
}
```

### 10.3 Soft Delete

Replace TypeORM's `@DeleteDateColumn()` with explicit `deleted_at` checks:

```sql
WHERE deleted_at IS NULL AND company_id = $1
```

In Rust:
```rust
sqlx::query!("UPDATE journal_entries SET deleted_at = now() WHERE id = $1 AND company_id = $2", id, company_id)
    .execute(pool).await?;
```
