-- Create victims table
CREATE TABLE victims (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(255) NOT NULL,
    document_id VARCHAR(50),
    birth_date DATE,
    phone VARCHAR(50),
    city_id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_victims_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT
);

CREATE INDEX idx_victims_city_id ON victims(city_id);
CREATE INDEX idx_victims_is_deleted ON victims(is_deleted);
CREATE INDEX idx_victims_full_name ON victims(full_name);

CREATE TRIGGER update_victims_updated_at
    BEFORE UPDATE ON victims
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create victim_addresses table
CREATE TABLE victim_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    victim_id UUID NOT NULL,
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
    CONSTRAINT fk_victim_addresses_victim FOREIGN KEY (victim_id) REFERENCES victims(id) ON DELETE RESTRICT
);

CREATE INDEX idx_victim_addresses_victim_id ON victim_addresses(victim_id);
CREATE INDEX idx_victim_addresses_is_deleted ON victim_addresses(is_deleted);

CREATE TRIGGER update_victim_addresses_updated_at
    BEFORE UPDATE ON victim_addresses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
