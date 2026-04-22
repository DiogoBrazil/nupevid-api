-- Postgres não permite DROP VALUE em ENUM; solução canônica é recriar o type.
-- Pré-requisito: nenhuma linha com status = 'Expired' (garantido por M1).
-- O índice antigo por vítima usa a coluna status e precisa ser removido antes
-- da recriação do enum; ele será substituído pelo índice por (victim, offender) em M5.
-- ATENÇÃO: migração destrutiva. Rollback exige recriar o type antigo e reconverter.

DROP INDEX IF EXISTS idx_protective_measures_one_active_per_victim;

ALTER TYPE protective_measure_status_enum RENAME TO protective_measure_status_enum_old;

CREATE TYPE protective_measure_status_enum AS ENUM ('Valid', 'Revoked');

ALTER TABLE protective_measures
    ALTER COLUMN status TYPE protective_measure_status_enum
    USING status::text::protective_measure_status_enum;

DROP TYPE protective_measure_status_enum_old;
