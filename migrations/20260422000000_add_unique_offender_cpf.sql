-- Ensure active offenders cannot share the same CPF.
CREATE UNIQUE INDEX IF NOT EXISTS idx_offenders_cpf_unique
ON offenders(cpf)
WHERE cpf IS NOT NULL AND is_deleted = false;
