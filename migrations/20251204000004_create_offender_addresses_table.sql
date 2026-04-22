-- Create offender_addresses table

CREATE TABLE offender_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    offender_id UUID NOT NULL,
    street VARCHAR(255),
    number VARCHAR(50),
    district VARCHAR(100),
    city_id UUID,
    zip_code VARCHAR(20),
    complement VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);

-- Foreign key will be added after offenders table is created
-- CREATE INDEX after foreign key is created
CREATE INDEX idx_offender_addresses_is_deleted ON offender_addresses(is_deleted);

CREATE TRIGGER update_offender_addresses_updated_at
    BEFORE UPDATE ON offender_addresses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
