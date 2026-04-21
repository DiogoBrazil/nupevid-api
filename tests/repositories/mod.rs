//! Phase 5 — Repository tests.
//!
//! Testes que executam contra o banco real (via `test_helpers::setup_test_db`),
//! mas chamam os repositórios PostgreSQL **diretamente** (não via HTTP).
//! Objetivo: proteger a camada de persistência contra regressões em SQL,
//! joins, filtros dinâmicos e soft-delete. Cada arquivo cobre um repositório
//! crítico identificado no plano (§7).

pub mod attendance_members_repository_test;
pub mod protective_measures_repository_test;
pub mod users_repository_test;
pub mod work_sessions_repository_test;
