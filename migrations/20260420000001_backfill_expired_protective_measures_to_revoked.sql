-- Prepara remoção do status 'Expired' convertendo linhas existentes para 'Revoked'.
-- Decisão de negócio: medidas protetivas passam a ter prazo indeterminado, não existindo
-- mais o conceito de 'expiração'. Linhas históricas marcadas como Expired passam a ser
-- consideradas simplesmente Revoked (fim da vigência) para preservar não-ativação.
UPDATE protective_measures
SET status = 'Revoked', updated_at = NOW()
WHERE status = 'Expired';
