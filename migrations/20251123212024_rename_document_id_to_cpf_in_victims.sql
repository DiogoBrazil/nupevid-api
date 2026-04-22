-- Rename document_id to cpf in victims table
-- Change type to VARCHAR(11) and add UNIQUE constraint

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'victims' AND column_name = 'document_id'
    ) THEN
        ALTER TABLE victims RENAME COLUMN document_id TO cpf;
    END IF;
END $$;

ALTER TABLE victims
    ALTER COLUMN cpf TYPE VARCHAR(11);

CREATE UNIQUE INDEX IF NOT EXISTS idx_victims_cpf_unique ON victims(cpf) WHERE cpf IS NOT NULL AND is_deleted = false;
