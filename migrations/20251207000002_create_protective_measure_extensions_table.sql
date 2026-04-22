-- Create protective_measure_extensions table
-- This table stores the history of all extensions/renewals of protective measures

CREATE TABLE IF NOT EXISTS protective_measure_extensions (
    id UUID PRIMARY KEY,
    protective_measure_id UUID NOT NULL,
    extension_number INTEGER NOT NULL,
    extension_date DATE NOT NULL,
    new_valid_until DATE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,

    CONSTRAINT fk_extensions_protective_measure
        FOREIGN KEY (protective_measure_id)
        REFERENCES protective_measures(id)
        ON DELETE CASCADE,

    -- Ensure no duplicate extension numbers per measure
    CONSTRAINT unique_extension_number_per_measure
        UNIQUE (protective_measure_id, extension_number)
);

-- Indexes for better query performance
CREATE INDEX idx_extensions_protective_measure_id
    ON protective_measure_extensions(protective_measure_id);

CREATE INDEX idx_extensions_date
    ON protective_measure_extensions(extension_date);

CREATE INDEX idx_extensions_is_deleted
    ON protective_measure_extensions(is_deleted);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER update_protective_measure_extensions_updated_at
    BEFORE UPDATE ON protective_measure_extensions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE protective_measure_extensions IS 'Histórico de prorrogações das medidas protetivas';
COMMENT ON COLUMN protective_measure_extensions.extension_number IS 'Número da prorrogação (1ª, 2ª, 3ª, etc.)';
COMMENT ON COLUMN protective_measure_extensions.extension_date IS 'Data em que a prorrogação foi concedida';
COMMENT ON COLUMN protective_measure_extensions.new_valid_until IS 'Nova data de validade após a prorrogação';
COMMENT ON COLUMN protective_measure_extensions.notes IS 'Observações sobre a prorrogação';
