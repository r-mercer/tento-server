# Architecture

## Overview
REST/GraphQL API server for user management built with Rust and Actix-web.

## Technology Stack
- **Web Framework**: Actix-web 4.12
- **Database**: MongoDB 3.5 (async driver)
- **GraphQL**: async-graphql 7.0
- **Runtime**: Tokio
- **Validation**: validator
- **Serialization**: Serde

## Directory Structure
```
src/
├── main.rs              # Application entry point
├── handlers/            # HTTP endpoint handlers
├── services/            # Business logic layer
├── repositories/        # Data access layer (trait-based)
├── models/
│   ├── domain/          # Domain entities
│   └── dto/             # Request/response objects
├── graphql/             # GraphQL schema
├── db/                  # Database connection management
├── config.rs            # Environment configuration
├── errors.rs            # Unified error handling
└── app_state.rs         # Shared application state
```

## Architecture Patterns

### Layered Architecture
Request flow: Handlers → Services → Repositories → Database

### Repository Pattern
Trait-based abstraction for data access with MongoDB implementation.

### Dual API Interface
- REST endpoints for CRUD operations
- GraphQL interface for flexible querying

### Dependency Injection
AppState holds Arc-wrapped services injected via `web::Data`.

### Error Handling
Unified `AppError` enum implements both HTTP `ResponseError` and GraphQL `ErrorExtensions`.

### Configuration Management
Environment-based configuration with sensible defaults.

## Key Components

| Component | Responsibility |
|-----------|---------------|
| Handlers | HTTP request routing and response formatting |
| Services | Business logic and validation |
| Repositories | Data persistence abstraction |
| Models | Domain entities and DTOs |
| GraphQL | Alternative query interface |
| Database | Connection pooling and management |
