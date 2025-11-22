-- Create attendances table
CREATE TABLE attendances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    victim_id UUID NOT NULL,
    was_victim_present BOOLEAN NOT NULL,
    attendance_date DATE NOT NULL,
    attendance_time TIME NOT NULL,
    description TEXT,
    latitude NUMERIC(10, 7),
    longitude NUMERIC(10, 7),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_attendances_victim FOREIGN KEY (victim_id) REFERENCES victims(id) ON DELETE RESTRICT
);

CREATE INDEX idx_attendances_victim_id ON attendances(victim_id);
CREATE INDEX idx_attendances_is_deleted ON attendances(is_deleted);
CREATE INDEX idx_attendances_date ON attendances(attendance_date);

CREATE TRIGGER update_attendances_updated_at
    BEFORE UPDATE ON attendances
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create attendance_addresses table
CREATE TABLE attendance_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    attendance_id UUID NOT NULL,
    street VARCHAR(255),
    number VARCHAR(50),
    district VARCHAR(100),
    city_name VARCHAR(100),
    state VARCHAR(2),
    zip_code VARCHAR(20),
    complement VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_attendance_addresses_attendance FOREIGN KEY (attendance_id) REFERENCES attendances(id) ON DELETE RESTRICT
);

CREATE INDEX idx_attendance_addresses_attendance_id ON attendance_addresses(attendance_id);
CREATE INDEX idx_attendance_addresses_is_deleted ON attendance_addresses(is_deleted);

CREATE TRIGGER update_attendance_addresses_updated_at
    BEFORE UPDATE ON attendance_addresses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
