-- Create offenders table
CREATE TABLE offenders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(255) NOT NULL,
    cpf VARCHAR(50),
    birth_date DATE,
    city_id UUID NOT NULL,
    victim_id UUID NOT NULL,
    imprisoned BOOLEAN NOT NULL DEFAULT FALSE,
    occupation VARCHAR(255),
    workplace VARCHAR(255),
    is_public_security_agent BOOLEAN NOT NULL DEFAULT FALSE,
    security_force security_force_enum,
    relationship_to_victim relationship_to_victim_enum NOT NULL,
    uses_alcohol BOOLEAN NOT NULL DEFAULT FALSE,
    uses_drugs BOOLEAN NOT NULL DEFAULT FALSE,
    has_psychiatric_issues BOOLEAN NOT NULL DEFAULT FALSE,
    psychiatric_issues_type VARCHAR(255),
    was_drunk_during_assault BOOLEAN NOT NULL DEFAULT FALSE,
    observation TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_offenders_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT,
    CONSTRAINT fk_offenders_victim FOREIGN KEY (victim_id) REFERENCES victims(id) ON DELETE RESTRICT
);

CREATE INDEX idx_offenders_city_id ON offenders(city_id);
CREATE INDEX idx_offenders_victim_id ON offenders(victim_id);
CREATE INDEX idx_offenders_is_deleted ON offenders(is_deleted);
CREATE INDEX idx_offenders_full_name ON offenders(full_name);
CREATE INDEX idx_offenders_relationship ON offenders(relationship_to_victim);
CREATE INDEX idx_offenders_imprisoned ON offenders(imprisoned);

CREATE TRIGGER update_offenders_updated_at
    BEFORE UPDATE ON offenders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Now add foreign keys to related tables
ALTER TABLE offender_phones
    ADD CONSTRAINT fk_offender_phones_offender
    FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT;

CREATE INDEX idx_offender_phones_offender_id ON offender_phones(offender_id);

ALTER TABLE offender_addresses
    ADD CONSTRAINT fk_offender_addresses_offender
    FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT;

CREATE INDEX idx_offender_addresses_offender_id ON offender_addresses(offender_id);

ALTER TABLE offender_addresses
    ADD CONSTRAINT fk_offender_addresses_city
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT;

CREATE INDEX idx_offender_addresses_city_id ON offender_addresses(city_id);

ALTER TABLE offender_work_addresses
    ADD CONSTRAINT fk_offender_work_addresses_offender
    FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT;

CREATE INDEX idx_offender_work_addresses_offender_id ON offender_work_addresses(offender_id);

ALTER TABLE offender_work_addresses
    ADD CONSTRAINT fk_offender_work_addresses_city
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT;

CREATE INDEX idx_offender_work_addresses_city_id ON offender_work_addresses(city_id);
