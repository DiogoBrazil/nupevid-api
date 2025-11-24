-- Add valid_until column to protective_measures table
-- This field indicates when the protective measure expires

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'protective_measures' AND column_name = 'valid_until'
    ) THEN
        ALTER TABLE protective_measures ADD COLUMN valid_until DATE;
    END IF;
END $$;
