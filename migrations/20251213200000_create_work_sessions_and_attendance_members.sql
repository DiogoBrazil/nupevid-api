-- Migration: Create Work Sessions and Attendance Members Tables
-- Description: Sistema de rastreamento de equipes para atendimentos
-- Date: 2025-12-13

-- 1. Criar enum para função do membro na equipe
CREATE TYPE team_member_function AS ENUM (
    'Commander',
    'Driver',
    'Patroller'
);

-- 2. Criar tabela de sessões de trabalho
CREATE TABLE work_sessions (
    id UUID PRIMARY KEY,
    created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Constraint: apenas 1 sessão ativa por usuário criador
    CONSTRAINT unique_active_session_per_user
        EXCLUDE (created_by_user_id WITH =)
        WHERE (is_active = true)
);

-- Índices para work_sessions
CREATE INDEX idx_work_sessions_user_id ON work_sessions(created_by_user_id);
CREATE INDEX idx_work_sessions_is_active ON work_sessions(is_active);
CREATE INDEX idx_work_sessions_started_at ON work_sessions(started_at);

-- Trigger para updated_at
CREATE TRIGGER update_work_sessions_updated_at
    BEFORE UPDATE ON work_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 3. Criar tabela de membros da sessão de trabalho
CREATE TABLE work_session_members (
    id UUID PRIMARY KEY,
    work_session_id UUID NOT NULL REFERENCES work_sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    function team_member_function,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Um usuário não pode estar duas vezes na mesma sessão
    UNIQUE(work_session_id, user_id)
);

-- Índices para work_session_members
CREATE INDEX idx_work_session_members_session ON work_session_members(work_session_id);
CREATE INDEX idx_work_session_members_user ON work_session_members(user_id);
CREATE INDEX idx_work_session_members_function ON work_session_members(function);

-- 4. Criar tabela de membros em atendimento a vítimas
CREATE TABLE attendance_victim_members (
    id UUID PRIMARY KEY,
    attendance_victim_id UUID NOT NULL REFERENCES attendance_victims(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    work_session_id UUID REFERENCES work_sessions(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Um usuário não pode estar duas vezes no mesmo atendimento
    UNIQUE(attendance_victim_id, user_id)
);

-- Índices para attendance_victim_members
CREATE INDEX idx_attendance_victim_members_attendance ON attendance_victim_members(attendance_victim_id);
CREATE INDEX idx_attendance_victim_members_user ON attendance_victim_members(user_id);
CREATE INDEX idx_attendance_victim_members_session ON attendance_victim_members(work_session_id);

-- 5. Criar tabela de membros em atendimento a agressores
CREATE TABLE attendance_offender_members (
    id UUID PRIMARY KEY,
    attendance_offender_id UUID NOT NULL REFERENCES attendance_offenders(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    work_session_id UUID REFERENCES work_sessions(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Um usuário não pode estar duas vezes no mesmo atendimento
    UNIQUE(attendance_offender_id, user_id)
);

-- Índices para attendance_offender_members
CREATE INDEX idx_attendance_offender_members_attendance ON attendance_offender_members(attendance_offender_id);
CREATE INDEX idx_attendance_offender_members_user ON attendance_offender_members(user_id);
CREATE INDEX idx_attendance_offender_members_session ON attendance_offender_members(work_session_id);

-- Nota: Políticas de acesso são validadas no código Rust
-- Regras implementadas:
--   - Todos os usuários podem criar, atualizar e encerrar suas próprias sessões
--   - Apenas CITY_ADMIN e ROOT podem visualizar sessões de outros usuários
--   - Todos podem gerenciar membros de atendimentos da sua cidade
--   - Validações de cidade são aplicadas em todas as operações
