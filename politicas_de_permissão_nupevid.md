Plano de Implementação - Sistema de Políticas de Permissão

  1. Hierarquia de Perfis e Capacidades

  ┌─────────────────────────────────────────────────────────────────────────┐
  │                              ROOT                                        │
  │  • Acesso total implícito (sem policies)                                │
  │  • Pode adicionar QUALQUER permissão para QUALQUER usuário              │
  │  • Único que pode criar cidades                                         │
  │  • Único que pode adicionar permissões extras para CITY_ADMIN           │
  └─────────────────────────────────────────────────────────────────────────┘
                                      │
                      ┌───────────────┴───────────────┐
                      ▼                               ▼
  ┌─────────────────────────────────┐  ┌─────────────────────────────────┐
  │          CITY_ADMIN             │  │          CITY_USER              │
  │                                 │  │                                 │
  │ PADRÃO (city_id):               │  │ PADRÃO (city_id):               │
  │  • CRUD completo na sua cidade  │  │  • Apenas LEITURA na sua cidade │
  │                                 │  │                                 │
  │ EXTRAS (via ROOT):              │  │ EXTRAS (via ROOT ou CITY_ADMIN):│
  │  • Permissões em outras cidades │  │  • Permissões adicionais        │
  │                                 │  │                                 │
  │ PODE ADICIONAR PERMISSÕES:      │  │ NÃO PODE adicionar permissões   │
  │  • Para CITY_USER da sua cidade │  │                                 │
  │  • Apenas permissões que possui │  │                                 │
  └─────────────────────────────────┘  └─────────────────────────────────┘

  ---
  2. Regras de Atribuição de Permissões

  | Quem atribui | Para quem               | Quais permissões                         |
  |--------------|-------------------------|------------------------------------------|
  | ROOT         | CITY_ADMIN              | Qualquer permissão em qualquer cidade    |
  | ROOT         | CITY_USER               | Qualquer permissão em qualquer cidade    |
  | CITY_ADMIN   | CITY_USER da sua cidade | Apenas permissões que ele próprio possui |
  | CITY_USER    | -                       | ❌ Não pode atribuir permissões           |

  ---
  3. Exemplo Prático

  Cenário:
  - Cidade A: Porto Velho (city_id: aaa-111)
  - Cidade B: Ariquemes (city_id: bbb-222)
  - Cidade C: Vilhena (city_id: ccc-333)

  CITY_ADMIN "João" (city_id: aaa-111):
    permission_policies: {
      "can_read_victim": ["aaa-111", "bbb-222"],    // Sua cidade + Ariquemes (dado por ROOT)
      "can_create_victim": ["aaa-111"],              // Apenas sua cidade
      "can_update_victim": ["aaa-111"],
      "can_delete_victim": ["aaa-111"]
    }

  CITY_USER "Maria" (city_id: aaa-111):
    permission_policies: {
      "can_read_victim": ["aaa-111"]                 // Padrão: apenas leitura na sua cidade
    }

  → João (CITY_ADMIN) quer dar permissão para Maria ler vítimas de Ariquemes:
    ✅ PERMITIDO - João tem "can_read_victim" em "bbb-222"

  → João quer dar permissão para Maria ler vítimas de Vilhena:
    ❌ NEGADO - João NÃO tem permissão em "ccc-333"

  → João quer dar permissão para Maria CRIAR vítimas em Ariquemes:
    ❌ NEGADO - João NÃO tem "can_create_victim" em "bbb-222"

  ---
  4. Estrutura no Banco de Dados

  -- Adicionar coluna na tabela users
  ALTER TABLE users ADD COLUMN permission_policies JSONB DEFAULT '{}';

  -- Exemplo de dados:
  -- CITY_ADMIN com permissões padrão + extras
  {
    "can_read_city": ["aaa-111"],
    "can_update_city": ["aaa-111"],
    "can_delete_city": ["aaa-111"],
    "can_create_user": ["aaa-111"],
    "can_read_user": ["aaa-111"],
    "can_update_user": ["aaa-111"],
    "can_delete_user": ["aaa-111"],
    "can_create_victim": ["aaa-111"],
    "can_read_victim": ["aaa-111", "bbb-222"],  // Extra dado por ROOT
    "can_update_victim": ["aaa-111"],
    "can_delete_victim": ["aaa-111"],
    "can_create_attendance": ["aaa-111"],
    "can_read_attendance": ["aaa-111"],
    "can_update_attendance": ["aaa-111"],
    "can_delete_attendance": ["aaa-111"],
    "can_create_protective_measure": ["aaa-111"],
    "can_read_protective_measure": ["aaa-111"],
    "can_update_protective_measure": ["aaa-111"],
    "can_delete_protective_measure": ["aaa-111"]
  }

  ---
  5. Policies Padrão por Perfil

  CITY_ADMIN (ao ser criado):
  can_read_city, can_update_city, can_delete_city: [city_id]
  can_create_user, can_read_user, can_update_user, can_delete_user: [city_id]
  can_create_victim, can_read_victim, can_update_victim, can_delete_victim: [city_id]
  can_create_attendance, can_read_attendance, can_update_attendance, can_delete_attendance: [city_id]
  can_create_protective_measure, can_read_protective_measure, can_update_protective_measure, can_delete_protective_measure: [city_id]

  CITY_USER (ao ser criado):
  can_read_city: [city_id]
  can_read_user: [city_id]
  can_read_victim: [city_id]
  can_read_attendance: [city_id]
  can_read_protective_measure: [city_id]

  ---
  6. Etapas de Implementação

  1. Migração: Adicionar coluna permission_policies JSONB na tabela users
  2. Entidades: Atualizar CreateUser, UpdateUser, User
  3. Serviço de Usuários:
    - Preencher policies padrão na criação
    - Validar quem pode atribuir quais policies
  4. Middleware/Helper de Autorização: Função para verificar policy
  5. Atualizar todos os services: Usar verificação de policy em vez de perfil
  6. Testes: Atualizar e criar novos testes

  ---
  7. Confirmação

  Entendi corretamente que:

  1. ✅ ROOT = acesso total, sem policies
  2. ✅ CITY_ADMIN = CRUD padrão na sua cidade, pode receber extras de ROOT
  3. ✅ CITY_USER = Leitura padrão na sua cidade
  4. ✅ CITY_ADMIN pode dar permissões para CITY_USER da sua cidade, mas apenas as que ele possui
  5. ✅ CITY_USER não pode dar permissões para ninguém
  6. ✅ Criar cidade = apenas ROOT