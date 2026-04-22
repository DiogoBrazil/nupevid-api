-- Create offender_phones table for multiple phone numbers per offender

CREATE TABLE offender_phones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    offender_id UUID NOT NULL,
    phone VARCHAR(50) NOT NULL,
    phone_type VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);

-- Foreign key will be added after offenders table is created
-- CREATE INDEX after foreign key is created
CREATE INDEX idx_offender_phones_is_deleted ON offender_phones(is_deleted);

CREATE TRIGGER update_offender_phones_updated_at
    BEFORE UPDATE ON offender_phones
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
