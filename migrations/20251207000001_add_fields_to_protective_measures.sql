-- Add distance_meters and sei_process_number to protective_measures table

DO $$
BEGIN
    -- Add distance_meters column
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'protective_measures' AND column_name = 'distance_meters'
    ) THEN
        ALTER TABLE protective_measures ADD COLUMN distance_meters INTEGER;
    END IF;

    -- Add sei_process_number column
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'protective_measures' AND column_name = 'sei_process_number'
    ) THEN
        ALTER TABLE protective_measures ADD COLUMN sei_process_number VARCHAR(100);
    END IF;
END $$;

-- Add comment for documentation
COMMENT ON COLUMN protective_measures.distance_meters IS 'Distância em metros que o agressor deve manter da vítima';
COMMENT ON COLUMN protective_measures.sei_process_number IS 'Número do processo no sistema SEI';
