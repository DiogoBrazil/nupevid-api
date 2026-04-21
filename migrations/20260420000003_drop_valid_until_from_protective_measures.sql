-- Medidas protetivas passam a ter prazo indeterminado.
-- ATENÇÃO: migração destrutiva e irreversível. Recomendado snapshot do banco antes.
-- Executar APENAS depois de garantir que nenhum código referencia a coluna (Fase 0 completa).
ALTER TABLE protective_measures
    DROP COLUMN valid_until;
