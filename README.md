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
8. [Desenho Multi-Tenant e Referência de Domínio](#8-desenho-multi-tenant-e-referência-de-domínio)
9. [Modelo de Autenticação e Sessão](#9-modelo-de-autenticação-e-sessão)
10. [Referência de Endpoints](#10-referência-de-endpoints)
11. [Instalação](#11-instalação)
12. [Variáveis de Ambiente](#12-variáveis-de-ambiente)
13. [Execução da Aplicação](#13-execução-da-aplicação)
14. [Banco de Dados](#14-banco-de-dados)
15. [Documentação da API](#15-documentação-da-api)
16. [Considerações de Segurança](#16-considerações-de-segurança)
17. [Melhorias Futuras](#17-melhorias-futuras)
18. [Contribuição](#18-contribuição)
19. [Licença](#19-licença)

---

## 1. Visão Geral do Projeto

O **NUPEVID** é uma API para digitalizar o fluxo de trabalho da Patrulha Maria da Penha, substituindo controles manuais por uma base única de dados e operações.

O sistema foi construído para:

- cadastrar vítimas, agressores e medidas protetivas;
- registrar atendimentos de campo (a vítimas e a agressores);
- gerenciar equipes em sessão de serviço;
- controlar acesso por perfil e cidade;
- monitorar a saúde do servidor de aplicação;
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

- autenticação por **access token (JWT)** de curta duração + **refresh token opaco** com rotação;
- autorização por políticas granulares (`permission_policies`) vinculadas a cidades;
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
- Consulta de agressores vinculados a uma vítima.
- Registro de medidas protetivas com status (`Valid`, `Revoked`).
- Filtros de medidas protetivas por `victim_id` e/ou `offender_id`.

### Operação da Patrulha

- Registro de atendimentos a vítimas.
- Registro de atendimentos a agressores.
- Registro de endereço do atendimento.
- Vinculação de membros da equipe aos atendimentos.
- Exigência de sessão de trabalho ativa para criação de atendimento.
- Consulta de atendimentos por vítima, por agressor e por medida protetiva.

### Sessões de Trabalho e Equipes

- Criação de sessão de trabalho com membros.
- Encerramento de sessão e consulta da sessão ativa.
- Gestão de membros (adicionar/remover/atualizar função).
- Regras de equipe: exatamente 1 `Commander`; no máximo 1 `Driver`; `Patroller` ilimitado.
- Regra de cidade: equipe deve ser da mesma cidade (exceto cenários ROOT sem cidade).

### Usuários, Perfis e Políticas

- Perfis: `ROOT`, `CITY_ADMIN`, `CITY_USER`.
- Políticas granulares por cidade via JSONB (`permission_policies`).
- Atribuição/remoção de cidades em uma política de usuário.
- Alteração de senha pelo próprio usuário e reset de senha por administrador.

### Autenticação e Sessão

- Login com emissão de access token + refresh token.
- Rotação de refresh token (`/auth/refresh`) e revogação no logout (`/auth/logout`).
- Criação automática (opcional) de sessão de trabalho no login.

### Observabilidade

- Endpoint de informações da máquina (CPU, memória, disco, IP externo e uptime).

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

A API segue uma **arquitetura em camadas** com elementos de **Ports and Adapters** (Clean Architecture). O domínio (`src/core`) não depende de frameworks; as implementações concretas (web, banco, hashing, JWT, métricas) ficam nas bordas.

### Fluxo de requisição

```text
Route → Controller → UseCase → Repository (+ Queries SQL) → PostgreSQL
                        │
                        ├── Validators (campos, CPF, políticas, equipe)
                        ├── Authorization (AuthContext + check_policy)
                        └── Adapters (Argon2, JWT, métricas de sistema)
```

### Camadas

- **Routes (`src/routes`)**
  - Define endpoints HTTP e composição por escopo (`/api` e `/api/v1`).
- **Controllers (`src/controllers`)**
  - Faz parsing de path/query/body, extrai os claims do JWT e delega ao use case. Não contém regra de negócio.
- **Use Cases (`src/usecases`)**
  - Concentra regras de negócio, autorização, validações de domínio e orquestração. Cada caso de uso expõe um método `execute(...)`.
- **Repositories (`src/repositories`)**
  - Implementa acesso ao banco via SQLx. Separados em repositórios de leitura e de escrita por domínio.
  - `src/repositories/queries` — SQL bruto centralizado como constantes por módulo.
  - `src/repositories/models` — mapeamento de linhas do banco para tipos Rust (row mappers).
- **Presenters (`src/presenters`)**
  - Montagem de respostas compostas (ex.: medidas protetivas e sessões de trabalho com entidades relacionadas).
- **Core (`src/core`)** — domínio puro:
  - `entities` — modelos de domínio.
  - `value_objects` — tipos de domínio (perfis, postos, políticas, estados, batalhões, composição de equipe, CPF, matrícula).
  - `commands` — DTOs de entrada (ex.: `CreateUser`, `SaveVictim`).
  - `read_models` — DTOs de saída (inclui respostas de `auth` e `users`).
  - `filters` — parâmetros de busca/paginação.
  - `contracts` — portas (traits) de repositório e de adapters.
  - `authorization`, `auth_context`, `policy_defaults` — RBAC + políticas por cidade.
- **Adapters (`src/adapters`)**
  - Implementações concretas: `password_hasher` (Argon2), `token_generator` (JWT), `system_metrics` (sysinfo).
- **Middleware (`src/middleware`)**
  - `AuthMiddleware`: valida `api_key` e o JWT (access token) para rotas protegidas.
- **Validators (`src/validators`)**
  - Regras de validação de campos, CPF, políticas, sessão de trabalho etc.
- **Utils (`src/utils`)**
  - Helpers de resposta HTTP, paginação, extração de claims e o seeder do admin.
- **Config (`src/config`)**
  - Leitura de variáveis de ambiente e pool de conexões PostgreSQL.

### Conceitos de domínio

- **Cidade**: unidade de isolamento lógico do tenant.
- **Usuário**: operador do sistema com perfil e políticas por cidade.
- **Vítima e Agressor**: entidades centrais do caso, com dados pessoais e contatos.
- **Medida Protetiva**: vínculo judicial com status e prazo indeterminado.
- **Atendimento**: registro operacional em campo, com membros de equipe.
- **Sessão de Trabalho**: contexto ativo de equipe para operação diária.

### Módulos funcionais da API

`auth`, `users`, `cities`, `victims`, `offenders`, `protective-measures`, `attendance-victims`, `attendance-offenders`, `work-sessions`, `machine-information`.

---

## 6. Stack Tecnológica

- **Linguagem**: Rust (edition 2024)
- **Framework HTTP**: Actix Web
- **Banco de dados**: PostgreSQL
- **Acesso a dados**: SQLx (queries SQL explícitas)
- **Autenticação**: JWT (`jsonwebtoken`, HS256) como access token + refresh token opaco
- **Autorização**: RBAC + políticas por cidade (JSONB)
- **Hash de senha / refresh token**: Argon2
- **Métricas de sistema**: `sysinfo` + `num_cpus`; IP externo via `reqwest`
- **Serialização**: Serde / Serde JSON
- **Logs**: `log` + `env_logger`
- **Containerização**: Docker e Docker Compose
- **Proxy reverso**: Traefik v3.0 (+ Let's Encrypt em produção)
- **Documentação OpenAPI**: `swagger.yml` servido em runtime
- **Testes**: integração com Actix test + PostgreSQL real; testes unitários com `mockall`
- **Serviços externos**: nenhum obrigatório para a regra de negócio. O endpoint de máquina consulta um serviço de IP público (ipify); a interface Swagger UI usa assets de CDN (unpkg/cdnjs).

---

## 7. Estrutura do Projeto

```text
.
├── src/
│   ├── main.rs                     # bootstrap da aplicação
│   ├── lib.rs                      # exportação dos módulos
│   ├── app_factory.rs              # injeção de dependências e montagem do App
│   ├── adapters/                   # Argon2, JWT, métricas de sistema
│   ├── config/
│   │   ├── config_env.rs           # leitura de variáveis de ambiente
│   │   └── database.rs             # pool PostgreSQL
│   ├── core/
│   │   ├── entities/               # modelos de domínio
│   │   ├── value_objects/          # perfis, postos, políticas, estados, etc.
│   │   ├── commands/               # DTOs de entrada
│   │   ├── read_models/            # DTOs de leitura/resposta (inclui auth, users)
│   │   ├── filters/                # parâmetros de busca/paginação
│   │   ├── contracts/              # portas (traits) de repositório/adapters
│   │   ├── authorization.rs        # verificação de permissões
│   │   ├── auth_context.rs         # carregamento de políticas do usuário
│   │   └── policy_defaults.rs      # políticas padrão por perfil
│   ├── middleware/                 # autenticação por api_key + JWT
│   ├── routes/                     # mapeamento de rotas
│   ├── controllers/                # camada HTTP
│   ├── usecases/                   # regras de negócio (casos de uso)
│   ├── presenters/                 # montagem de respostas compostas
│   ├── repositories/
│   │   ├── queries/                # SQL bruto por módulo
│   │   ├── models/                 # row mappers
│   │   └── *.rs                    # repositórios de leitura/escrita
│   ├── validators/                 # validações de payload/domínio
│   └── utils/                      # helpers, paginação, respostas, seeder
├── migrations/                     # migrações SQL versionadas
├── tests/                          # testes de integração
├── tests_e2e/                      # testes end-to-end
├── swagger.yml                     # contrato OpenAPI
├── nupevid-backend.postman_collection.json
├── docker-compose.yml              # produção (Traefik por labels + TLS)
├── docker-compose-local.yml        # desenvolvimento (Traefik estático)
├── Dockerfile
├── traefik/                        # configuração de rotas estáticas (local)
├── .env.example
└── TESTING.md
```

> O diretório `src/` contém centenas de arquivos organizados por domínio; a árvore acima mostra os diretórios e exemplos representativos.

---

## 8. Desenho Multi-Tenant e Referência de Domínio

O projeto implementa multi-tenant com **isolamento lógico por cidade**.

### Como o isolamento acontece

- A maior parte das entidades possui `city_id` direto (ex.: `users`, `victims`, `offenders`) ou indireto (ex.: medidas e atendimentos vinculados a vítima/agressor).
- Usuários não-ROOT possuem políticas por cidade no campo `permission_policies` (JSONB).
- Os use cases carregam o `AuthContext` e aplicam `check_policy(...)` antes de operações sensíveis.
- Listagens paginadas filtram por cidades permitidas (`allowed_cities`) diretamente no SQL.

### Perfis

| Perfil | Valor | Escopo |
|--------|-------|--------|
| Root | `ROOT` | Acesso global implícito; não vinculado a cidade. Único perfil que cria/atualiza/exclui **cidades**. |
| Administrador de cidade | `CITY_ADMIN` | Vinculado a uma cidade; gerencia usuários e dados operacionais da(s) sua(s) cidade(s) conforme políticas. |
| Operador | `CITY_USER` | Vinculado a uma cidade; perfil operacional, majoritariamente leitura + operação de sessões de trabalho. |

### Políticas (`permission_policies`)

As permissões são granulares por recurso e armazenadas como JSONB no usuário, mapeando cada política para a lista de cidades (`Uuid`) em que vale:

```json
{
  "create_victims": ["<city-uuid>"],
  "read_victims": ["<city-uuid>"],
  "update_victims": ["<city-uuid>"]
}
```

Há **29 políticas** organizadas por recurso:

| Recurso | Políticas | Observação |
|---------|-----------|------------|
| Cidades | `create_cities`, `read_cities`, `update_cities`, `delete_cities` | `create/update/delete` são **exclusivas de ROOT** (não atribuíveis); `read_cities` é atribuível. |
| Usuários | `create_users`, `read_users`, `update_users`, `delete_users` | atribuíveis |
| Vítimas | `create_victims`, `read_victims`, `update_victims`, `delete_victims` | atribuíveis |
| Agressores | `create_offenders`, `read_offenders`, `update_offenders`, `delete_offenders` | atribuíveis |
| Atendimentos | `create_attendances`, `read_attendances`, `update_attendances`, `delete_attendances` | atribuíveis |
| Medidas protetivas | `create_protective_measures`, `read_protective_measures`, `update_protective_measures`, `delete_protective_measures` | atribuíveis |
| Sessões de trabalho | `create_work_sessions`, `update_work_sessions`, `end_work_sessions`, `view_other_work_sessions` | atribuíveis |
| Membros de atendimento | `manage_attendance_members` | atribuível |

> "Atribuível" significa que a política pode ser concedida a um usuário via `permission_policies`. As políticas de escrita de cidade são reservadas ao ROOT.

### Políticas padrão por perfil

Atribuídas automaticamente na criação do usuário (`src/core/policy_defaults.rs`), aplicadas à cidade do usuário:

- **ROOT** — sem políticas explícitas (acesso implícito a tudo).
- **CITY_ADMIN** — `read_cities` + CRUD completo de usuários, vítimas, agressores, atendimentos e medidas protetivas + `create/update/end_work_sessions` + `view_other_work_sessions` + `manage_attendance_members`.
- **CITY_USER** — `read_cities`, `read_victims`, `read_offenders`, `read_attendances`, `read_protective_measures`, `read_users` + `create/update/end_work_sessions` + `manage_attendance_members`.

### Composição de equipe (sessão de trabalho)

Funções (`TeamMemberFunction`): `Commander`, `Driver`, `Patroller`. Regras validadas em `src/core/value_objects/team_composition.rs`:

- a equipe deve ter **exatamente 1** `Commander`;
- **no máximo 1** `Driver`;
- `Patroller` ilimitado (e membros sem função são permitidos);
- não é possível remover o `Commander` (troque a função antes) nem o último membro da equipe.

### Catálogos e formatos validados

- **Estado permitido**: `RO`.
- **Batalhões**: `1ºBPM`…`11ºBPM`, `BPTRAN`, `BOPE`, `BPCHOQUE`, `BPTAR`, `CIPO/BURITIS`.
- **Postos/graduações (14)**: `CEL PM`, `TC PM`, `MAJ PM`, `CAP PM`, `1º TEN PM`, `2º TEN PM`, `ASP OF PM`, `CAD PM`, `ST PM`, `1º SGT PM`, `2º SGT PM`, `3º SGT PM`, `CB PM`, `SD PM`.
- **Matrícula**: somente dígitos, prefixo `1000` e no máximo 9 caracteres.
- **CPF**: formato mascarado `000.000.000-00` (14 caracteres), com dígitos verificadores válidos (módulo 11) e rejeição de sequências repetidas.

---

## 9. Modelo de Autenticação e Sessão

A API usa um par **access token + refresh token**.

- **Access token**: JWT assinado em HS256, com `iss`/`aud` validados. TTL configurável por `ACCESS_TOKEN_TTL_SECONDS` (padrão **900s / 15 min**). Enviado em cada requisição protegida no header `Authorization: Bearer <access_token>`.
- **Refresh token**: token opaco no formato `{uuid}.{secret}`. Apenas o **hash Argon2** é persistido na tabela `refresh_tokens`. TTL configurável por `REFRESH_TOKEN_TTL_SECONDS` (padrão **604800s / 7 dias**).
- **Rotação**: `POST /auth/refresh` emite um novo par e invalida o refresh token anterior.
- **Revogação**: `POST /auth/logout` invalida o refresh token informado.

Além do JWT, **toda rota da API** (exceto Swagger e LogDock) exige o header `api_key`:

```http
api_key: <API_KEY>
Authorization: Bearer <access_token>
```

Rotas públicas (sem JWT): `/api/v1/auth/login`, `/api/v1/auth/refresh`, `/api/v1/auth/logout`, `/api/swagger*`.

### Exemplos

**Login** — `POST /api/v1/auth/login`

```json
// Request (header: api_key)
{
  "email": "admin@email.com",
  "password": "admin@123",
  "auto_create_session": true
}
```

```json
// Response 200
{
  "message": "Operation successful",
  "status": 200,
  "data": {
    "access_token": "<jwt>",
    "refresh_token": "<uuid>.<secret>",
    "token_type": "Bearer",
    "expires_in": 900,
    "id": "1f0a...",
    "full_name": "admin",
    "email": "admin@email.com",
    "rank": "ST PM",
    "registration": "1000012345",
    "profile": "ROOT",
    "work_session": null
  }
}
```

> `auto_create_session` (padrão `true`) cria/recupera uma sessão de trabalho no login; quando há sessão ativa, ela é retornada em `work_session`.

**Refresh** — `POST /api/v1/auth/refresh`

```json
// Request
{ "refresh_token": "<uuid>.<secret>" }
```

```json
// Response 200 → data
{
  "access_token": "<novo-jwt>",
  "refresh_token": "<novo-uuid>.<novo-secret>",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Logout** — `POST /api/v1/auth/logout`

```json
// Request
{ "refresh_token": "<uuid>.<secret>" }
```

```json
// Response 200
{ "message": "Operation successful", "status": 200, "data": null }
```

### Envelope de resposta padrão

Respostas simples:

```json
{ "message": "Operation successful", "status": 200, "data": { } }
```

Respostas paginadas:

```json
{
  "message": "Operation successful",
  "status": 200,
  "data": [],
  "page": 1,
  "page_size": 10,
  "total_items": 0,
  "total_pages": 0
}
```

### Exemplos de criação

**Criar usuário** — `POST /api/v1/users`

```json
{
  "rank": "SD PM",
  "registration": "1000012346",
  "full_name": "Maria da Silva",
  "profile": "CITY_USER",
  "email": "maria@pm.ro.gov.br",
  "password": "Senha@123",
  "city_id": "<city-uuid>",
  "permission_policies": null
}
```

**Criar vítima** — `POST /api/v1/victims`

```json
{
  "full_name": "Joana de Souza",
  "cpf": "529.982.247-25",
  "birth_date": "1990-05-20",
  "city_id": "<city-uuid>",
  "phones": [{ "number": "69999990000" }],
  "addresses": [{ "street": "Rua A", "number": "100" }],
  "has_children": true,
  "children_count": 2,
  "uses_alcohol": false,
  "uses_drugs": false
}
```

> Campos como `has_children`, `has_special_needs` e `has_psychiatric_issues` têm `default = false`. Consulte o `swagger.yml` para o contrato completo de cada payload.

---

## 10. Referência de Endpoints

Base das rotas de negócio: `/api/v1`. Salvo indicação em contrário, todas exigem `api_key` + `Authorization: Bearer <access_token>`.

Paginação padrão: `page` (padrão `1`) e `page_size` (padrão `10`, máximo `200`).

### Autenticação (`/auth`) — público

| Método | Path | Descrição |
|--------|------|-----------|
| POST | `/api/v1/auth/login` | Autentica e emite access + refresh token |
| POST | `/api/v1/auth/refresh` | Rotaciona o par de tokens |
| POST | `/api/v1/auth/logout` | Revoga o refresh token |

### Usuários (`/users`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/users` | — | Cria usuário |
| GET | `/api/v1/users` | `page`, `page_size` | Lista usuários |
| GET | `/api/v1/users/search` | `name?`, `registration?` | Busca por nome ou matrícula |
| GET | `/api/v1/users/{id}` | — | Obtém usuário |
| PUT | `/api/v1/users/{id}` | — | Atualiza usuário |
| DELETE | `/api/v1/users/{id}` | — | Exclui (soft delete) usuário |
| PATCH | `/api/v1/users/password` | — | Altera a própria senha |
| POST | `/api/v1/users/{id}/password/reset` | — | Reseta a senha de um usuário |
| POST | `/api/v1/users/{id}/policies/{policy}/cities` | — | Adiciona cidades a uma política |
| DELETE | `/api/v1/users/{id}/policies/{policy}/cities` | — | Remove cidades de uma política |

### Cidades (`/cities`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/cities` | — | Cria cidade (ROOT) |
| GET | `/api/v1/cities` | `page`, `page_size` | Lista cidades |
| GET | `/api/v1/cities/{id}` | — | Obtém cidade |
| PUT | `/api/v1/cities/{id}` | — | Atualiza cidade (ROOT) |
| DELETE | `/api/v1/cities/{id}` | — | Exclui cidade (ROOT) |

### Vítimas (`/victims`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/victims` | — | Cria vítima |
| GET | `/api/v1/victims` | `page`, `page_size` | Lista vítimas |
| GET | `/api/v1/victims/search` | `name?`, `cpf?` | Busca por nome ou CPF |
| GET | `/api/v1/victims/{id}` | — | Obtém vítima |
| PUT | `/api/v1/victims/{id}` | — | Atualiza vítima |
| DELETE | `/api/v1/victims/{id}` | — | Exclui (soft delete) vítima |
| POST | `/api/v1/victims/{id}/phones` | — | Adiciona telefone |
| PUT | `/api/v1/victims/{id}/phones/{phone_id}` | — | Atualiza telefone |
| DELETE | `/api/v1/victims/{id}/phones/{phone_id}` | — | Remove telefone |
| POST | `/api/v1/victims/{id}/addresses` | — | Adiciona endereço |
| PUT | `/api/v1/victims/{id}/addresses/{address_id}` | — | Atualiza endereço |
| DELETE | `/api/v1/victims/{id}/addresses/{address_id}` | — | Remove endereço |

### Agressores (`/offenders`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/offenders` | — | Cria agressor |
| GET | `/api/v1/offenders` | `page`, `page_size` | Lista agressores |
| GET | `/api/v1/offenders/search` | `name?`, `cpf?` | Busca por nome ou CPF |
| GET | `/api/v1/offenders/victim/{victim_id}` | — | Agressores de uma vítima |
| GET | `/api/v1/offenders/{id}` | — | Obtém agressor |
| PUT | `/api/v1/offenders/{id}` | — | Atualiza agressor |
| DELETE | `/api/v1/offenders/{id}` | — | Exclui (soft delete) agressor |
| POST | `/api/v1/offenders/{id}/phones` | — | Adiciona telefone |
| PUT | `/api/v1/offenders/{id}/phones/{phone_id}` | — | Atualiza telefone |
| DELETE | `/api/v1/offenders/{id}/phones/{phone_id}` | — | Remove telefone |
| POST | `/api/v1/offenders/{id}/addresses` | — | Adiciona endereço |
| PUT | `/api/v1/offenders/{id}/addresses/{address_id}` | — | Atualiza endereço |
| DELETE | `/api/v1/offenders/{id}/addresses/{address_id}` | — | Remove endereço |

### Medidas Protetivas (`/protective-measures`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/protective-measures` | — | Cria medida protetiva |
| GET | `/api/v1/protective-measures` | `page?`, `page_size?`, `include_related_entities?`, `victim_id?`, `offender_id?` | Lista com filtros opcionais |
| GET | `/api/v1/protective-measures/{id}` | `include_related_entities?` | Obtém medida protetiva |
| PUT | `/api/v1/protective-measures/{id}` | — | Atualiza medida protetiva |
| DELETE | `/api/v1/protective-measures/{id}` | — | Exclui medida protetiva |
| GET | `/api/v1/protective-measures/victim/{victim_id}` | `include_related_entities?` | Medidas de uma vítima |

### Atendimentos a Vítimas (`/attendance-victims`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/attendance-victims` | — | Cria atendimento (exige sessão ativa) |
| GET | `/api/v1/attendance-victims` | `page`, `page_size` | Lista atendimentos |
| GET | `/api/v1/attendance-victims/{id}` | — | Obtém atendimento |
| PUT | `/api/v1/attendance-victims/{id}` | — | Atualiza atendimento |
| DELETE | `/api/v1/attendance-victims/{id}` | — | Exclui atendimento |
| GET | `/api/v1/attendance-victims/{id}/members` | — | Lista membros |
| POST | `/api/v1/attendance-victims/{id}/members` | — | Adiciona membro |
| DELETE | `/api/v1/attendance-victims/{id}/members/{user_id}` | — | Remove membro |
| GET | `/api/v1/attendance-victims/by-measure/{protective_measure_id}` | — | Atendimentos por medida |
| GET | `/api/v1/attendance-victims/by-victim/{victim_id}` | — | Atendimentos por vítima |

### Atendimentos a Agressores (`/attendance-offenders`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/attendance-offenders` | — | Cria atendimento (exige sessão ativa) |
| GET | `/api/v1/attendance-offenders` | `page`, `page_size` | Lista atendimentos |
| GET | `/api/v1/attendance-offenders/{id}` | — | Obtém atendimento |
| PUT | `/api/v1/attendance-offenders/{id}` | — | Atualiza atendimento |
| DELETE | `/api/v1/attendance-offenders/{id}` | — | Exclui atendimento |
| GET | `/api/v1/attendance-offenders/{id}/members` | — | Lista membros |
| POST | `/api/v1/attendance-offenders/{id}/members` | — | Adiciona membro |
| DELETE | `/api/v1/attendance-offenders/{id}/members/{user_id}` | — | Remove membro |
| GET | `/api/v1/attendance-offenders/by-measure/{protective_measure_id}` | — | Atendimentos por medida |
| GET | `/api/v1/attendance-offenders/by-offender/{offender_id}` | — | Atendimentos por agressor |
| GET | `/api/v1/attendance-offenders/by-victim/{victim_id}` | — | Atendimentos relacionados a uma vítima |

### Sessões de Trabalho (`/work-sessions`)

| Método | Path | Query params | Descrição |
|--------|------|--------------|-----------|
| POST | `/api/v1/work-sessions` | — | Cria sessão com membros |
| GET | `/api/v1/work-sessions` | `user_id?`, `start_date?`, `end_date?`, `city_id?`, `page?`, `page_size?`, `include_related_entities?` | Lista sessões com filtros |
| GET | `/api/v1/work-sessions/active` | — | Sessão ativa do usuário |
| POST | `/api/v1/work-sessions/end` | — | Encerra a sessão |
| GET | `/api/v1/work-sessions/{id}` | — | Obtém sessão |
| PUT | `/api/v1/work-sessions/{id}` | — | Atualiza sessão |
| POST | `/api/v1/work-sessions/{id}/members` | — | Adiciona membro |
| DELETE | `/api/v1/work-sessions/{id}/members/{member_id}` | — | Remove membro |
| PUT | `/api/v1/work-sessions/{id}/members` | — | Atualiza membros (lote) |
| PUT | `/api/v1/work-sessions/{id}/members/{user_id}/function` | — | Atualiza função de um membro |

### Informações da Máquina (`/machine-information`)

| Método | Path | Descrição |
|--------|------|-----------|
| GET | `/api/v1/machine-information` | CPU, memória, disco, IP externo e uptime do servidor |

### Documentação (`/api/swagger`) — público

| Método | Path | Descrição |
|--------|------|-----------|
| GET | `/api/swagger` | Swagger UI |
| GET | `/api/swagger/swagger.yaml` | OpenAPI YAML |
| GET | `/api/swagger/simple` | Swagger UI simplificada |
| GET | `/api/swagger/raw` | Conteúdo bruto do YAML |
| GET | `/api/swagger/check` | Verificação de existência/tamanho do YAML |

---

## 11. Instalação

### Pré-requisitos

- Rust (compatível com edition 2024; recomendado `rustc >= 1.90`)
- Cargo
- PostgreSQL (recomendado 16+, conforme `docker-compose.yml`)
- (Opcional, recomendado) `sqlx-cli` para gerenciar migrações externamente

### 1) Clonar e entrar no projeto

```bash
git clone <repo-url>
cd nupevid-api
```

### 2) Configurar ambiente

```bash
cp .env.example .env
```

Edite o `.env` com valores válidos (ver [Variáveis de Ambiente](#12-variáveis-de-ambiente)).

### 3) Subir PostgreSQL

Opção A: localmente (serviço próprio).

Opção B: com Docker:

```bash
docker compose up -d postgres
```

### 4) Migrações

Por padrão, a aplicação **executa as migrações pendentes automaticamente no startup**
(`RUN_MIGRATIONS_ON_STARTUP=true`). As migrations são embutidas no binário em tempo de compilação
(`sqlx::migrate!("./migrations")`), então nada além do banco acessível é necessário — inclusive no
container Docker. A execução é idempotente (controlada pela tabela `_sqlx_migrations`); falha na
migração **aborta o boot**.

Para gerenciá-las externamente, defina `RUN_MIGRATIONS_ON_STARTUP=false` e aplique antes de iniciar
a API. Com `sqlx-cli`:

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

## 12. Variáveis de Ambiente

Arquivo de referência: `.env.example`.

### Obrigatórias (a aplicação não inicia sem elas)

```env
DATABASE_URL=postgresql://user:password@localhost:5432/database
SERVER_ADDR=0.0.0.0:8080
JWT_SECRET=your_jwt_secret_key_here
JWT_ISSUER=nupevid-api
JWT_AUDIENCE=nupevid-api
API_KEY=your_api_key_here
```

### Opcionais com valor padrão

```env
DB_MAX_CONNECTIONS=20            # tamanho máximo do pool
ENABLE_BOOTSTRAP_ROOT=false      # cria o admin padrão no boot quando true
RUN_MIGRATIONS_ON_STARTUP=true   # aplica migrações pendentes no boot
ACCESS_TOKEN_TTL_SECONDS=900     # validade do access token (15 min)
REFRESH_TOKEN_TTL_SECONDS=604800 # validade do refresh token (7 dias)
```

### Auxiliares (compose, testes e serviços de apoio)

Não são lidas diretamente pela API, mas usadas pelo Docker Compose, pelos testes ou por serviços de apoio:

```env
DOCKER_DATABASE_URL=postgresql://user:password@postgres:5432/database
DATABASE_TEST_URL=postgresql://user:password@localhost:5432/database_test
RUST_LOG=info
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your_database_name
DB_USER=your_database_user
DB_PASSWORD=your_database_password
# Serviço LogDock e Traefik (produção)
LOGDOCK_ADMIN=
LOGDOCK_PASSWORD=
LOGDOCK_JWT_SECRET=
LOGDOCK_BASE_PATH=/logdock
TRAEFIK_LOG_LEVEL=INFO
```

### Explicação

- `DATABASE_URL`: conexão principal com PostgreSQL.
- `SERVER_ADDR`: bind HTTP da aplicação.
- `JWT_SECRET` / `JWT_ISSUER` / `JWT_AUDIENCE`: segredo de assinatura e validações `iss`/`aud` do access token.
- `API_KEY`: chave exigida no header `api_key`.
- `DB_MAX_CONNECTIONS`: tamanho máximo do pool de conexões.
- `ENABLE_BOOTSTRAP_ROOT`: quando habilitado, cria o usuário ROOT padrão no boot caso não exista.
- `RUN_MIGRATIONS_ON_STARTUP`: aplica migrações no startup (padrão `true`); `false` para gerenciá-las externamente.
- `ACCESS_TOKEN_TTL_SECONDS` / `REFRESH_TOKEN_TTL_SECONDS`: validade do access e do refresh token.
- `DOCKER_DATABASE_URL`: conexão usada pela API dentro do `docker compose` (host `postgres`).
- `DATABASE_TEST_URL`: banco usado na suíte de integração.
- `RUST_LOG`: nível de log.
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASSWORD`: úteis em docker-compose/interpolação.

> Flags booleanas aceitam `1`, `true`, `yes` ou `on` (case-insensitive).

---

## 13. Execução da Aplicação

### Rodar localmente

```bash
cargo run
```

API disponível em `http://<SERVER_ADDR>/api/v1`.

### Rodar com Docker Compose (local)

```bash
docker compose -f docker-compose-local.yml up --build
```

Sobe a stack local com o Traefik como porta de entrada (porta configurável; padrão `8080`). No ambiente
local, o Traefik usa um **arquivo de rotas estático** (`traefik/dynamic.yml`) para encaminhar as
requisições para `api` e `logdock` na rede Docker interna, **sem TLS**.

- API: `http://localhost:8080/api/v1`
- LogDock: `http://localhost:8080/logdock`

### Produção

O arquivo `docker-compose.yml` é destinado à produção. Nele, o Traefik v3.0 **descobre as rotas por
labels Docker** (então `traefik/dynamic.yml` não precisa existir na VM) e provê HTTPS via **Let's
Encrypt**. O domínio padrão é `nupevid-api.nexuslearn.com.br` e a rede `traefiknet` é externa
(pré-criada). Serviços: `postgres` (16-alpine), `api`, `logdock` e o `reverse-proxy` (Traefik).

### Seed do usuário ROOT

Quando `ENABLE_BOOTSTRAP_ROOT=true`, a API cria no startup um usuário ROOT padrão **caso ainda não
exista** (verificação por e-mail):

- `email`: `admin@email.com`
- `senha`: `admin@123`

Altere imediatamente em ambiente não-local.

---

## 14. Banco de Dados

### SGBD

- PostgreSQL

### Estratégia

- Migrações SQL versionadas em `migrations/`.
- Soft delete (`is_deleted`) nas principais entidades.
- Índices para busca, paginação e integridade por cidade.
- Uso da extensão PostgreSQL `uuid-ossp` (criada na migração inicial).
- Restrições de negócio no banco, por exemplo:
  - 1 `CITY_ADMIN` ativo por cidade (`users`);
  - 1 medida protetiva ativa (`status = Valid`) por par vítima/agressor.

### Principais tabelas

- `users`, `cities`, `refresh_tokens`
- `victims`, `victim_phones`, `victim_addresses`
- `offenders`, `offender_phones`, `offender_addresses`
- `protective_measures`
- `attendance_victims`, `attendance_victim_addresses`, `attendance_victim_members`
- `attendance_offenders`, `attendance_offender_addresses`, `attendance_offender_members`
- `work_sessions`, `work_session_members`

### Inicialização

1. criar banco;
2. aplicar migrações em ordem (ou deixar a API aplicá-las no startup);
3. iniciar aplicação.

Nos testes de integração, as migrações são aplicadas automaticamente pelos helpers.

---

## 15. Documentação da API

A aplicação expõe Swagger/OpenAPI em:

- `GET /api/swagger`
- `GET /api/swagger/swagger.yaml`
- `GET /api/swagger/simple`
- `GET /api/swagger/raw`
- `GET /api/swagger/check`

O contrato completo está em `swagger.yml`. Há também uma coleção do Postman pronta para uso:
`nupevid-backend.postman_collection.json` (+ ambiente `nupevid-backend.postman_environment.json`).

A relação completa de endpoints está na seção [Referência de Endpoints](#10-referência-de-endpoints).

---

## 16. Considerações de Segurança

### Implementado

- Access token (JWT) de curta duração + refresh token opaco com rotação.
- Refresh tokens armazenados apenas como hash Argon2 e revogados no logout.
- API key obrigatória por header para rotas da API (exceto Swagger e LogDock).
- Autorização por perfil + política granular por cidade.
- Verificação de escopo por cidade em use cases e queries.
- Senhas com hash Argon2.
- Comparação de API key em tempo constante (mitiga timing attacks).
- Controle de senha temporária com expiração no reset.

### Pontos de atenção

- Seed de credencial ROOT padrão no startup (`admin@email.com` / `admin@123`) — alterar fora de ambiente local.
- CORS atualmente permissivo (`allow_any_origin`, `allow_any_method`, `allow_any_header`).
- Sem rate limit nativo para login no código atual.

---

## 17. Melhorias Futuras

- Endurecer a política de CORS por domínio confiável.
- Remover o seed padrão de admin e adotar bootstrap seguro (one-time setup).
- Adicionar rate limiting e proteção anti brute-force no login.
- Incluir trilha de auditoria para ações críticas (ex.: alterações de políticas e medidas).
- Incluir pipeline CI com lint, testes e validação de migrações.

---

## 18. Contribuição

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

## 19. Licença

Não há arquivo de licença (`LICENSE`) definido no repositório atualmente.
