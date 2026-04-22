-- Create victim_phones table for multiple phone numbers per victim

CREATE TABLE victim_phones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    victim_id UUID NOT NULL,
    phone VARCHAR(50) NOT NULL,
    phone_type VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_victim_phones_victim FOREIGN KEY (victim_id)
        REFERENCES victims(id) ON DELETE RESTRICT
);

CREATE INDEX idx_victim_phones_victim_id ON victim_phones(victim_id);
CREATE INDEX idx_victim_phones_is_deleted ON victim_phones(is_deleted);

CREATE TRIGGER update_victim_phones_updated_at
    BEFORE UPDATE ON victim_phones
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Migrate existing phone data from victims table to victim_phones
INSERT INTO victim_phones (victim_id, phone, phone_type)
SELECT id, phone, 'primary'
FROM victims
WHERE phone IS NOT NULL AND phone != '' AND is_deleted = FALSE;

-- Drop the old phone column from victims table
ALTER TABLE victims DROP COLUMN phone;
