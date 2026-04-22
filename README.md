# NUPEVID API

Sistema de gestão operacional da **Patrulha Maria da Penha** da Polícia Militar, com foco em atendimento pós-violência doméstica, controle de medidas protetivas e inteligência operacional por município.

## Sumário

1. [Visão Geral do Projeto](#1-visão-geral-do-projeto)
2. [Problema](#2-problema)
3. [Solução](#3-solução)
4. [Funcionalidades](#4-funcionalidades)
5. [Arquitetura do Sistema](#5-arquitetura-do-sistema)
6. [Stack Tecnológica](#6-stack-tecnológica)
7. [Estrutura do Projeto](#7-estrutura-do-projeto)
8. [Desenho Multi-Tenant](#8-desenho-multi-tenant)
9. [Instalação](#9-instalação)
10. [Variáveis de Ambiente](#10-variáveis-de-ambiente)
11. [Execução da Aplicação](#11-execução-da-aplicação)
12. [Banco de Dados](#12-banco-de-dados)
13. [Documentação da API](#13-documentação-da-api)
14. [Considerações de Segurança](#14-considerações-de-segurança)
15. [Melhorias Futuras](#15-melhorias-futuras)
16. [Contribuição](#16-contribuição)
17. [Licença](#17-licença)

---

## 1. Visão Geral do Projeto

O **NUPEVID** é uma API para digitalizar o fluxo de trabalho da Patrulha Maria da Penha, substituindo controles manuais por uma base única de dados e operações.

O sistema foi construído para:

- cadastrar vítimas, agressores e medidas protetivas;
- registrar atendimentos de campo;
- gerenciar equipes em sessão de serviço;
- controlar acesso por perfil e cidade;
- oferecer base de dados confiável para análise operacional e estatística.

Embora o contexto inicial seja **Ariquemes/RO**, a API foi implementada com isolamento lógico por cidade, permitindo uso simultâneo por múltiplos municípios.

---

## 2. Problema

No processo manual atual, há limitações operacionais relevantes:

- risco de inconsistência e perda de histórico;
- baixa rastreabilidade das ações da patrulha;
- dificuldade de verificar cumprimento/violação de medidas protetivas;
- baixa capacidade de geração de indicadores para decisão institucional.

---

## 3. Solução

A API NUPEVID centraliza o domínio operacional em módulos especializados (usuários, cidades, vítimas, agressores, medidas protetivas, atendimentos e sessões de trabalho), com:

- autenticação via JWT;
- autorização por políticas (`permission_policies`) vinculadas a cidades;
- filtros multi-tenant por cidade em consultas paginadas;
- soft delete para preservação histórica;
- validações de negócio e integridade relacional no banco.

---

## 4. Funcionalidades

### Gestão de Casos

- Cadastro, consulta, atualização e exclusão lógica de vítimas.
- Cadastro de telefones e endereços de vítimas.
- Cadastro, consulta, atualização e exclusão lógica de agressores.
- Cadastro de telefones e endereços de agressores.
- Registro de medidas protetivas com status (`Valid`, `Revoked`).

### Operação da Patrulha

- Registro de atendimentos a vítimas.
- Registro de atendimentos a agressores.
- Registro de endereço do atendimento.
- Vinculação de membros da equipe aos atendimentos.
- Exigência de sessão de trabalho ativa para criação de atendimento.

### Sessões de Trabalho e Equipes

- Criação de sessão de trabalho com membros.
- Encerramento de sessão.
- Gestão de membros (adicionar/remover/atualizar função).
- Regras de equipe: exatamente 1 `Commander`; no máximo 1 `Driver`.
- Regra de cidade: equipe deve ser da mesma cidade (exceto cenários ROOT sem cidade).

### Usuários, Perfis e Políticas

- Perfis: `ROOT`, `CITY_ADMIN`, `CITY_USER`.
- Políticas por cidade via JSONB (`permission_policies`).
- Atribuição/remoção de cidades em uma política de usuário.
- Reset de senha com senha temporária e expiração.

### Regras de validação de domínio

- Cidades, batalhões e estado são validados por catálogo interno (estado permitido: `RO`).
- Posto/graduação e perfil de usuário são validados contra listas permitidas.
- Matrícula deve seguir prefixo e formato definidos pelo domínio.
- CPF de vítima/agressor deve estar no formato mascarado e com dígitos verificadores válidos.

### Busca, Paginação e Respostas

- Paginação padronizada em listagens (`page`, `page_size`).
- Busca por nome/matrícula (usuários), nome/CPF (vítimas/agressores).
- Envelope de resposta padronizado (`message`, `status`, `data`).

### Documentação de API

- Swagger UI e arquivo OpenAPI YAML servidos pela própria aplicação.

---

## 5. Arquitetura do Sistema

A API segue uma **arquitetura em camadas** com elementos de **Ports and Adapters**:

### Fluxo de requisição

`Route -> Controller -> Service -> Repository -> PostgreSQL`

### Camadas

- **Routes (`src/routes`)**
  - Define endpoints HTTP e composição por escopo (`/api/v1`).
- **Controllers (`src/controllers`)**
  - Faz parsing de path/query/body e delega regras ao service.
- **Services (`src/services`)**
  - Concentra regras de negócio, autorização, validações de domínio e orquestração transacional.
- **Repositories (`src/repositories`)**
  - Implementa acesso ao banco via SQLx.
- **Queries (`src/config/querys`)**
  - Centraliza SQL bruto por módulo.
- **Core (`src/core`)**
  - Entidades de domínio e contratos (traits) de repositório/adapters.
- **Adapters (`src/adapters`)**
  - Implementações concretas de hashing de senha e geração de token.
- **Middleware (`src/middleware`)**
  - `AuthMiddleware`: valida `api_key` e JWT para rotas protegidas.
- **Validators (`src/validators`)**
  - Regras de validação de campos, CPF, políticas, sessão de trabalho etc.
- **Utils (`src/utils`)**
  - Helpers de autorização, paginação, mapeamento de erro e respostas HTTP.

### Conceitos de domínio

- **Cidade**: unidade de isolamento lógico do tenant.
- **Usuário**: operador do sistema com perfil e políticas por cidade.
- **Vítima e Agressor**: entidades centrais do caso, com dados pessoais e contatos.
- **Medida Protetiva**: vínculo judicial com status e prazo indeterminado.
- **Atendimento**: registro operacional em campo, com membros de equipe.
- **Sessão de Trabalho**: contexto ativo de equipe para operação diária.

### Módulos funcionais da API

- `auth`
- `users`
- `cities`
- `victims`
- `offenders`
- `protective-measures`
- `attendance-victims`
- `attendance-offenders`
- `work-sessions`

---

## 6. Stack Tecnológica

- **Linguagem**: Rust (edition 2024)
- **Framework HTTP**: Actix Web
- **Banco de dados**: PostgreSQL
- **Acesso a dados**: SQLx (queries SQL explícitas)
- **Autenticação**: JWT (`jsonwebtoken`)
- **Autorização**: RBAC + políticas por cidade (JSONB)
- **Hash de senha**: Argon2
- **Serialização**: Serde / Serde JSON
- **Logs**: `log` + `env_logger`
- **Containerização**: Docker e Docker Compose
- **Documentação OpenAPI**: `swagger.yml` servido em runtime
- **Testes**: integração com Actix test + PostgreSQL real
- **Serviços externos**: nenhum obrigatório para regra de negócio; a interface Swagger UI usa assets de CDN (unpkg/cdnjs)

---

## 7. Estrutura do Projeto

```text
.
├── src/
│   ├── main.rs                     # bootstrap da aplicação
│   ├── lib.rs                      # exportação dos módulos
│   ├── adapters/                   # implementações de token/hash
│   ├── config/
│   │   ├── config_env.rs           # leitura de variáveis de ambiente
│   │   ├── database.rs             # pool PostgreSQL
│   │   └── querys/                 # SQL por módulo
│   ├── core/
│   │   ├── entities/               # modelos de domínio
│   │   └── contracts/              # portas (traits) de repositório/adapters
│   ├── middleware/                 # autenticação por api_key + JWT
│   ├── routes/                     # mapeamento de rotas
│   ├── controllers/                # camada HTTP
│   ├── services/                   # regras de negócio
│   ├── repositories/               # persistência SQLx
│   ├── validators/                 # validações de payload/domínio
│   └── utils/                      # helpers comuns
├── migrations/                     # migrações SQL versionadas
├── tests/
│   ├── common/                     # helpers/fixtures
│   └── integration/                # testes de integração
├── swagger.yml                     # contrato OpenAPI
├── docker-compose.yml
├── Dockerfile
├── .env.example
└── TESTING.md
```

---

## 8. Desenho Multi-Tenant

O projeto implementa multi-tenant com **isolamento lógico por cidade**.

### Como o isolamento acontece

- A maior parte das entidades possui `city_id` direto (ex.: `users`, `victims`, `offenders`) ou indireto (ex.: medidas e atendimentos vinculados a vítima/agressor).
- Usuários não-ROOT possuem políticas por cidade no campo `permission_policies` (JSONB).
- Services aplicam `check_policy(...)` antes de operações sensíveis.
- Listagens paginadas usam `allowed_cities` para filtrar no SQL.

### Perfis

- **ROOT**
  - Acesso global implícito.
  - Único perfil que cria/atualiza/exclui cidades.
- **CITY_ADMIN**
  - Acesso administrativo por cidade conforme políticas atribuídas.
- **CITY_USER**
  - Perfil operacional com escopo controlado por políticas.

---

## 9. Instalação

### Pré-requisitos

- Rust (compatível com edition 2024; recomendado `rustc >= 1.90`)
- Cargo
- PostgreSQL (recomendado 16+, conforme `docker-compose.yml`)
- (Opcional, recomendado) `sqlx-cli` para rodar migrações

### 1) Clonar e entrar no projeto

```bash
git clone <repo-url>
cd nupevid-api
```

### 2) Configurar ambiente

```bash
cp .env.example .env
```

Edite o `.env` com valores válidos (ver seção [Variáveis de Ambiente](#10-variáveis-de-ambiente)).

### 3) Subir PostgreSQL

Opção A: localmente (serviço próprio)

Opção B: com Docker

```bash
docker compose up -d postgres
```

### 4) Rodar migrações (obrigatório)

A aplicação **não** executa migrações automaticamente em runtime. Execute antes de iniciar a API.

Com `sqlx-cli`:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx database create
sqlx migrate run
```

Alternativa via `psql`:

```bash
for f in $(ls migrations/*.sql | sort); do
  psql "$DATABASE_URL" -f "$f"
done
```

---

## 10. Variáveis de Ambiente

Arquivo de referência: `.env.example`

### Obrigatórias para a API

```env
DATABASE_URL=postgresql://user:password@localhost:5432/database
SERVER_ADDR=0.0.0.0:8080
JWT_SECRET=your_jwt_secret_key_here
JWT_ISSUER=nupevid-api
JWT_AUDIENCE=nupevid-api
API_KEY=your_api_key_here
```

### Usadas em testes

```env
DATABASE_TEST_URL=postgresql://user:password@localhost:5432/database_test
```

### Auxiliares/operacionais

```env
RUST_LOG=info
DB_MAX_CONNECTIONS=20
ENABLE_BOOTSTRAP_ROOT=false
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your_database_name
DB_USER=your_database_user
DB_PASSWORD=your_database_password
```

### Explicação

- `DATABASE_URL`: conexão principal com PostgreSQL.
- `SERVER_ADDR`: bind HTTP da aplicação.
- `JWT_SECRET`: segredo de assinatura/validação dos tokens JWT.
- `JWT_ISSUER`: emissor esperado nos tokens JWT.
- `JWT_AUDIENCE`: audiência esperada nos tokens JWT.
- `API_KEY`: chave exigida no header `api_key`.
- `DATABASE_TEST_URL`: banco usado na suíte de integração.
- `RUST_LOG`: nível de log.
- `DB_MAX_CONNECTIONS`: tamanho máximo do pool de conexões com o banco.
- `ENABLE_BOOTSTRAP_ROOT`: quando `true`, cria/atualiza o usuário administrador inicial no boot.
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER` e `DB_PASSWORD`: úteis em docker-compose/interpolação.

---

## 11. Execução da Aplicação

### Rodar localmente

```bash
cargo run
```

API disponível em:

```text
http://<SERVER_ADDR>/api/v1
```

### Rodar com Docker Compose

```bash
docker compose up --build
```

Observação: o `docker-compose.yml` referencia `./scripts/init-db.sql`. Se esse arquivo não existir no ambiente, ajuste/remova esse bind mount antes de subir a stack.

### Seed automático

No startup, a API faz seed de um usuário ROOT padrão caso não exista:

- `email`: `admin@email.com`
- `senha`: `admin@123`

Altere imediatamente em ambiente não-local.

---

## 12. Banco de Dados

### SGBD

- PostgreSQL

### Estratégia

- Migrações SQL versionadas em `migrations/`.
- Soft delete (`is_deleted`) nas principais entidades.
- Índices para busca, paginação e integridade por cidade.
- Uso da extensão PostgreSQL `uuid-ossp` (criada na migração inicial).
- Restrições de negócio no banco, por exemplo:
  - 1 `CITY_ADMIN` ativo por cidade (`users`)
  - 1 medida protetiva ativa (`status = Valid`) por par vítima/agressor

### Principais tabelas

- `users`, `cities`
- `victims`, `victim_phones`, `victim_addresses`
- `offenders`, `offender_phones`, `offender_addresses`
- `protective_measures`
- `attendance_victims`, `attendance_victim_addresses`, `attendance_victim_members`
- `attendance_offenders`, `attendance_offender_addresses`, `attendance_offender_members`
- `work_sessions`, `work_session_members`

### Inicialização

1. criar banco;
2. aplicar migrações em ordem;
3. iniciar aplicação.

Nos testes de integração, as migrações são aplicadas automaticamente pelos helpers.

---

## 13. Documentação da API

A aplicação expõe Swagger/OpenAPI em:

- `GET /api/swagger`
- `GET /api/swagger/swagger.yaml`
- `GET /api/swagger/simple`
- `GET /api/swagger/raw`

Base das rotas de negócio:

- `/api/v1/auth/login`
- `/api/v1/users`
- `/api/v1/cities`
- `/api/v1/victims`
- `/api/v1/offenders`
- `/api/v1/protective-measures`
- `/api/v1/attendance-victims`
- `/api/v1/attendance-offenders`
- `/api/v1/work-sessions`

### Autenticação nas requisições

Headers esperados para rotas protegidas:

```http
api_key: <API_KEY>
Authorization: Bearer <JWT>
```

Rotas públicas sem JWT:

- `/api/v1/auth/login`
- `/api/swagger*`

---

## 14. Considerações de Segurança

### Implementado

- JWT para autenticação de usuário.
- API key obrigatória por header para rotas da API (exceto Swagger).
- Autorização por perfil + política por cidade.
- Verificação de escopo por cidade em services e queries.
- Senhas com hash Argon2.
- Controle de senha temporária com expiração no reset.

### Pontos de atenção

- Seed de credencial ROOT padrão no startup (`admin@email.com` / `admin@123`).
- CORS atualmente permissivo (`allow_any_origin`, `allow_any_method`, `allow_any_header`).
- Sem rate limit nativo para login no código atual.

---

## 15. Melhorias Futuras

- Automatizar migrações no startup/entrypoint de produção.
- Endurecer política de CORS por domínio confiável.
- Remover seed padrão de admin e adotar bootstrap seguro (one-time setup).
- Adicionar rate limiting e proteção anti brute-force no login.
- Incluir trilha de auditoria para ações críticas (ex.: alterações de políticas e medidas).
- Incluir pipeline CI com lint, testes e validação de migrações.
- Ajustar `docker-compose` para não depender de arquivo de init ausente por padrão.

---

## 16. Contribuição

1. Crie uma branch de feature/correção.
2. Garanta que as migrações estejam aplicadas no ambiente local.
3. Rode testes:

```bash
cargo test -- --test-threads=1
```

4. Para testes unitários:

```bash
cargo test --lib
```

5. Abra PR com descrição clara de contexto, impacto e estratégia de validação.

Referência adicional: `TESTING.md`.

---

## 17. Licença

Não há arquivo de licença (`LICENSE`) definido no repositório atualmente.
