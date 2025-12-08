-- Add occurrence_report_number and offender_id to protective_measures table

-- Add occurrence_report_number column (optional)
ALTER TABLE protective_measures
    ADD COLUMN occurrence_report_number VARCHAR(100);

-- Add offender_id column (required) with foreign key
ALTER TABLE protective_measures
    ADD COLUMN offender_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- Remove the default value (it was only for migration purposes)
ALTER TABLE protective_measures
    ALTER COLUMN offender_id DROP DEFAULT;

-- Add foreign key constraint
ALTER TABLE protective_measures
    ADD CONSTRAINT fk_protective_measures_offender
    FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT;

-- Create index for better query performance
CREATE INDEX idx_protective_measures_offender_id ON protective_measures(offender_id);

-- Add comments for documentation
COMMENT ON COLUMN protective_measures.occurrence_report_number IS 'Número do boletim de ocorrência relacionado à medida protetiva';
COMMENT ON COLUMN protective_measures.offender_id IS 'ID do agressor relacionado à medida protetiva';
