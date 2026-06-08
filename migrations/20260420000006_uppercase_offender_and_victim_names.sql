-- Alinhar dados existentes com a nova normalização (QA-3): nomes são armazenados em
-- UPPERCASE + trim. Acentos são preservados. Migração idempotente.
UPDATE offenders
SET full_name = UPPER(TRIM(full_name)), updated_at = NOW()
WHERE full_name IS NOT NULL AND full_name <> UPPER(TRIM(full_name));

UPDATE victims
SET full_name = UPPER(TRIM(full_name)), updated_at = NOW()
WHERE full_name IS NOT NULL AND full_name <> UPPER(TRIM(full_name));
