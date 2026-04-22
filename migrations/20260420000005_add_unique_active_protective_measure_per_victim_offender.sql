-- Blindar a invariante "uma medida ativa por par (victim, offender)" no nível de banco.
-- Falha se existirem duplicatas; rodar pré-check antes:
--   SELECT victim_id, offender_id, COUNT(*)
--   FROM protective_measures
--   WHERE status = 'Valid' AND is_deleted = FALSE
--   GROUP BY victim_id, offender_id HAVING COUNT(*) > 1;
-- (Se esperamos que essa invariante já valha, o resultado deve ser vazio.)
DROP INDEX IF EXISTS idx_protective_measures_one_active_per_victim;

CREATE UNIQUE INDEX idx_protective_measures_unique_active_per_victim_offender
    ON protective_measures (victim_id, offender_id)
    WHERE status = 'Valid' AND is_deleted = FALSE;
