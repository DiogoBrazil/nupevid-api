-- Create protective_measures table
CREATE TABLE protective_measures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    process_number VARCHAR(100) NOT NULL,
    issued_at DATE NOT NULL,
    judicial_authority VARCHAR(255) NOT NULL,
    court_district VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    victim_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_protective_measures_victim FOREIGN KEY (victim_id) REFERENCES victims(id) ON DELETE RESTRICT
);

CREATE INDEX idx_protective_measures_victim_id ON protective_measures(victim_id);
CREATE INDEX idx_protective_measures_is_active ON protective_measures(is_active);
CREATE INDEX idx_protective_measures_is_deleted ON protective_measures(is_deleted);

-- Partial unique index: only one active measure per victim at a time
CREATE UNIQUE INDEX idx_protective_measures_one_active_per_victim
ON protective_measures(victim_id)
WHERE is_active = true AND is_deleted = false;

CREATE TRIGGER update_protective_measures_updated_at
    BEFORE UPDATE ON protective_measures
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
