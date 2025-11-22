## Estrutura dos Testes

```
tests/
├── common/                          # Código compartilhado entre testes
│   ├── fixtures.rs                  # Dados de teste (usuários válidos, inválidos, etc.)
│   ├── test_helpers.rs             # Helpers para setup de DB e app de teste (/users e full app)
│   └── db_fixtures.rs              # Funções de apoio para inserir cidades e vítimas diretamente no DB
└── integration/                     # Testes de integração
    ├── users_create_test.rs                    # CRUD básico de usuários
    ├── users_read_test.rs                      # Leitura de usuários
    ├── users_update_delete_test.rs             # Atualização e deleção de usuários
    ├── users_password_test.rs                  # Atualização de senha
    ├── auth_login_test.rs                      # Login (auth)
    ├── cities_integration_test.rs              # /api/v1/cities
    ├── users_multicity_test.rs                 # Regras de multi-cidade para usuários
    ├── victims_integration_test.rs             # Vítimas e endereços de vítima
    ├── protective_measures_integration_test.rs # Medidas protetivas
    └── attendances_integration_test.rs         # Atendimentos e endereços de atendimento

src/
├── adapters/
│   └── password_hasher_test.rs     # Testes unitários do hasher de senha
└── utils/
    └── validations_test.rs         # Testes unitários de validações
```

## Tipos de Testes

### Testes Unitários

Testam componentes isolados com ou sem mocks:

- **Password Hasher**: Testa hash, verificação, salts, caracteres especiais e unicode
- **Validações**: Testa validação de email e campos obrigatórios

**Executar apenas testes unitários:**
```bash
cargo test --lib
```

### Testes de Integração

Testam endpoints completos com banco de dados real, passando pelo middleware de autenticação (API key + JWT) quando aplicável.

#### /users (sem AuthMiddleware nos testes existentes)
- ✅ POST /users
  - Criar usuário válido
  - Email duplicado → 400
  - Registration duplicado → 400
  - Email inválido → 400
  - Campos vazios → 400
- ✅ GET /users
  - Lista todos os usuários
  - Lista vazia quando não há usuários
- ✅ GET /users/{id}
  - Usuário existente
  - 404 para inexistente
  - 404 para UUID inválido (rota não encontrada)
- ✅ PUT /users/{id}
  - Atualização com sucesso
  - Email duplicado com outro usuário → 400
  - Registration duplicado → 400
  - Usuário inexistente → 404
- ✅ DELETE /users/{id}
  - Soft delete de usuário existente
  - GET subsequente retorna 404
  - Usuário inexistente → 404
- ✅ PATCH /users/{id}/password
  - Atualiza senha com senha atual correta
  - Senha atual incorreta → 400
  - Usuário inexistente → 404
  - Campos vazios → 400

#### Regras de multi-cidade para usuários
- ✅ CITY_ADMIN/CITY_USER sem city_id em create/update → 400
- ✅ CITY_ADMIN com city_id válido → criado com sucesso
- ✅ Segundo CITY_ADMIN para mesma cidade (create ou update) → 400 (regra + índice parcial)

#### /api/v1/auth/login
- ✅ Login com credenciais válidas → 200 + token JWT
- ✅ Email inexistente → 401 (Invalid credentials)
- ✅ Senha incorreta → 401 (Invalid credentials)

#### /api/v1/cities
- ✅ POST /cities
  - Criação válida
  - state com tamanho != 2 → 400
  - Campos vazios → 400
- ✅ GET /cities
  - Lista vazia
  - Lista com múltiplas cidades
- ✅ GET /cities/{id}
  - Cidade existente
  - Cidade inexistente → 404
- ✅ PUT /cities/{id}
  - Atualização válida
  - state inválido → 400
  - Cidade inexistente → 404
- ✅ DELETE /cities/{id}
  - Soft delete → não aparece mais na listagem
  - Cidade inexistente → 404

#### /api/v1/victims & /api/v1/victim-addresses
- ✅ ROOT
  - Cria vítimas em cidades diferentes
  - Lista vítimas de todas as cidades
- ✅ CITY_ADMIN
  - Lista apenas vítimas da própria cidade
  - Não cria vítima em outra cidade → 403
- ✅ Soft delete de vítima
  - DELETE remove da listagem e GET/{id} → 404
- ✅ Endereços de vítima
  - CITY_ADMIN de cidade A não pode criar endereço para vítima de cidade B → 403

#### /api/v1/protective-measures
- ✅ Criação de medida para vítima na mesma cidade
- ✅ Regra "uma medida ativa por vítima"
  - Segunda medida ativa para mesma vítima → 400
- ✅ CITY_ADMIN de cidade A não cria medida para vítima em cidade B → 403
- ✅ Listagem por vítima `/victim/{victim_id}` retorna apenas medidas daquela vítima
- ✅ Criação para vítima inexistente → 404
- ✅ Soft delete de medida
  - DELETE → 200, GET/{id} → 404

#### /api/v1/attendances & /api/v1/attendance-addresses
- ✅ Criação de atendimento para vítima na mesma cidade
- ✅ CITY_ADMIN de cidade A não cria atendimento para vítima em cidade B → 403
- ✅ Listagem de atendimentos
  - ROOT vê todos
  - CITY_ADMIN vê apenas da própria cidade
- ✅ Soft delete de atendimento
  - DELETE → 200, GET/{id} → 404, não aparece na listagem
- ✅ CITY_ADMIN de cidade A não cria endereço de atendimento para atendimento em cidade B → 403
- ✅ CITY_ADMIN de cidade A é proibido de acessar atendimento (GET /{id}) em cidade B → 403

## Configuração para Testes de Integração

### Banco de Dados de Teste

Os testes de integração requerem um banco PostgreSQL. Configure a variável de ambiente:

```bash
# No arquivo .env ou export no terminal
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/nupevid_test"
```

**IMPORTANTE:** Use um banco de dados separado para testes! Os testes limpam a tabela `users` antes de cada execução.

### Criar Banco de Dados de Teste

```bash
# Criar o banco
createdb nupevid_test

# Ou via psql
psql -U postgres -c "CREATE DATABASE nupevid_test;"
```

As migrações são executadas automaticamente pelos helpers de teste.

## Executando os Testes

### Todos os Testes
```bash
cargo test -- --test-threads=1
```

### Apenas Testes Unitários
```bash
cargo test --lib
```

### Apenas Testes de Integração
```bash
cargo test --test integration_tests -- --test-threads=1
```

### Testes Específicos
```bash
# Por nome
cargo test test_create_user_success

# Por módulo
cargo test users_create

# Com output detalhado
cargo test -- --nocapture
```

### Executar Testes em Paralelo (não recomendado atualmente)
```bash
cargo test
```

**⚠️ Atenção**: Os testes de integração podem falhar quando executados em paralelo devido a race conditions no banco de dados compartilhado. Use `--test-threads=1` para garantir que todos os testes passem.

## Cobertura de Testes

### Resumo

- **Testes Unitários**: 11 testes
  - Password Hasher: 6 testes
  - Validações: 5 testes

- **Testes de Integração**: ~58 testes
  - Users CRUD + senha
  - Regras de multi-cidade de usuários
  - Login (/auth)
  - Cidades (/api/v1/cities)
  - Vítimas e endereços de vítima
  - Medidas protetivas
  - Atendimentos e endereços de atendimento

**Total aproximado: ~69 testes**

### Cenários Cobertos

#### Casos de Sucesso
- Criação, leitura, atualização e deleção de usuários
- Listagem de usuários (vazia e com dados)
- Atualização de senha

#### Casos de Erro
- Validações de campos obrigatórios
- Validações de email
- Duplicação de email e registration
- Recursos não encontrados (404)
- Senha incorreta
- UUIDs inválidos

## Boas Práticas

1. **Isolamento**: Cada teste de integração limpa o banco antes de executar (helpers `clean_users_table` / `clean_database`).
2. **Independência**: Testes não dependem uns dos outros.
3. **Dados de Teste**: Use fixtures do módulo `common::fixtures` e `common::db_fixtures`.
4. **Nomes Descritivos**: Nomes de teste indicam claramente o que testam.

## Fixtures Disponíveis

```rust
// Usuários válidos
fixtures::valid_create_user()        // João Silva
fixtures::valid_create_user_2()      // Maria Santos

// Dados de atualização
fixtures::valid_update_user()
fixtures::valid_update_password()

// Casos inválidos
fixtures::create_user_with_invalid_email()
fixtures::create_user_with_empty_fields()
fixtures::invalid_update_password()  // senha atual errada
```

## Troubleshooting

### Erro: "Failed to connect to database"
Verifique se o PostgreSQL está rodando e o DATABASE_URL está correto.

### Erro: "Failed to run migrations"
Certifique-se de que o banco existe e você tem permissões adequadas.

### Testes de integração falhando aleatoriamente
Execute com `--test-threads=1` para evitar race conditions.

### Password não verifica corretamente
Lembre-se que a ordem dos argumentos é `verify_password(hash, password)`.

## Dependências de Teste

```toml
[dev-dependencies]
actix-rt = "2.11.0"           # Runtime para testes assíncronos
actix-http = "3.11.2"         # HTTP para testes de integração
tokio = "1.48.0"              # Runtime assíncrono
mockall = "0.13.1"            # Mocking (para futuros testes)
pretty_assertions = "1.4.1"   # Assertions melhores
```

## Isolamento de Testes e Race Conditions

### Situação Atual

Os testes de integração compartilham o mesmo banco de dados e devem ser executados sequencialmente:

- **Execução Paralela**: ❌ 10 passed; 10 failed (race conditions)
- **Execução Sequencial** (`--test-threads=1`): ✅ 20 passed; 0 failed

**Causas das falhas em paralelo:**
1. Múltiplos testes acessando/modificando o mesmo banco simultaneamente
2. Estado compartilhado entre testes causando conflitos de dados
3. Testes esperando dados específicos que outros testes modificaram/deletaram
4. Violações de chave única quando testes tentam criar usuários com mesmo email/registration

### Opções de Melhoria Futura

#### Opção 1: Migrar para `sqlx::test` (Isolamento Total)

Cada teste recebe sua própria instância isolada de banco de dados automaticamente.

**Vantagens:**
- Verdadeiro isolamento de testes com bancos separados
- Sem conflitos de dados entre testes
- Pode executar testes em paralelo com segurança
- Limpeza automática após cada teste

**Desvantagens:**
- Execução de testes mais lenta (cada teste cria um novo banco)
- Requer configuração adequada de migrações
- Infraestrutura de teste mais complexa
- Overhead significativo para suíte pequena (20 testes)

**Exemplo de implementação:**
```rust
#[sqlx::test]
async fn test_create_user_success(pool: PgPool) -> sqlx::Result<()> {
    // Pool isolado criado automaticamente por sqlx::test
    let app = create_test_app(pool).await;
    
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&fixtures::valid_create_user())
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    
    Ok(())
}
```

#### Opção 2: Gerar Dados Únicos de Teste (Solução Pragmática)

Usar UUIDs para garantir que cada teste use dados únicos, evitando conflitos.

**Vantagens:**
- Implementação simples
- Testes podem rodar em paralelo
- Sem mudanças de infraestrutura
- Execução rápida de testes
- Adequado para suíte pequena/média

**Desvantagens:**
- Testes ainda compartilham o mesmo banco
- Necessita garantir limpeza ou dados não acumulam
- Não resolve todos os problemas de isolamento (ex: testes que consultam "todos os usuários")

**Exemplo de implementação:**
```rust
// tests/common/fixtures.rs
use uuid::Uuid;

pub fn unique_email() -> String {
    format!("test-{}@example.com", Uuid::new_v4().simple())
}

pub fn unique_registration() -> String {
    format!("{}", Uuid::new_v4().simple().to_string()[..10].to_uppercase())
}

pub fn valid_create_user() -> CreateUser {
    CreateUser {
        rank: "Professor".to_string(),
        registration: unique_registration(),
        full_name: format!("Test User {}", Uuid::new_v4().simple().to_string()[..8]),
        profile: "admin".to_string(),
        email: unique_email(),
        password: "senha123".to_string(),
    }
}
```

#### Opção 3: Manter Abordagem Atual (Status Quo)

Continuar executando testes sequencialmente com `--test-threads=1`.

**Vantagens:**
- Zero mudanças de código necessárias
- Testes já funcionam corretamente
- Simples e confiável

**Desvantagens:**
- Execução de testes mais lenta (apenas sequencial)
- Não escala bem com muitos testes
- Não segue as melhores práticas de teste (testes devem ser isolados)

### Recomendação

**Situação Atual (20 testes):** Manter **Opção 3** - usar `--test-threads=1` é aceitável

**Quando Reconsiderar:**

Migre para **Opção 2** (dados únicos) quando:
- ✅ Suíte crescer para 30-50 testes
- ✅ Tempo de execução sequencial se tornar problema (>15-20s)
- ✅ Equipe precisar de feedback mais rápido em CI/CD

Migre para **Opção 1** (sqlx::test) quando:
- ✅ Suíte crescer para 50+ testes de integração
- ✅ Problemas frequentes de isolamento mesmo com dados únicos
- ✅ Equipe precisar de garantias mais fortes de isolamento
- ✅ Overhead de setup/teardown se tornar insignificante comparado ao tempo total

### Casos Especiais

Alguns testes podem precisar de atenção especial ao implementar execução paralela:

1. **`test_get_all_users_empty`** - Espera banco vazio
   - Solução: Usar transação ou marcar como `#[serial_test::serial]`

2. **`test_get_all_users_success`** - Conta número exato de usuários
   - Solução: Filtrar por critério específico ou usar transações

## Próximos Passos

Melhorias futuras para a suíte de testes:

1. **Isolamento de testes** - Avaliar migração para Opção 2 ou Opção 1 conforme suíte crescer
2. Testes de middleware de autenticação (cobrir erros de API key ausente/errada)
3. Testes de performance/carga
4. Cobertura de código automatizada (tarpaulin)
5. Testes com mocks para serviços externos
6. Testes de concorrência
