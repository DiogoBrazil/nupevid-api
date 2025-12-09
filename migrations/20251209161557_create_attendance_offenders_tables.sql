-- Create ENUM type for violence aggravators
CREATE TYPE violence_aggravator_enum AS ENUM ('UsoBebidaAlcoolica', 'UsoDrogas', 'ProblemasPsiquiatricos', 'Outros');

-- Create attendance_offenders table
CREATE TABLE attendance_offenders (
    id UUID PRIMARY KEY,
    offender_id UUID NOT NULL,
    victim_id UUID NOT NULL,
    protective_measure_id UUID,
    was_offender_present BOOLEAN NOT NULL,
    attendance_date DATE NOT NULL,
    attendance_time TIME NOT NULL,
    is_remote BOOLEAN NOT NULL DEFAULT FALSE,
    assaults_children BOOLEAN NOT NULL DEFAULT FALSE,
    violence_aggravator violence_aggravator_enum NOT NULL,
    violence_aggravator_other TEXT,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_attendance_offenders_offender FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT,
    CONSTRAINT fk_attendance_offenders_victim FOREIGN KEY (victim_id) REFERENCES victims(id) ON DELETE RESTRICT,
    CONSTRAINT fk_attendance_offenders_protective_measure FOREIGN KEY (protective_measure_id) REFERENCES protective_measures(id) ON DELETE RESTRICT
);

CREATE INDEX idx_attendance_offenders_offender_id ON attendance_offenders(offender_id);
CREATE INDEX idx_attendance_offenders_victim_id ON attendance_offenders(victim_id);
CREATE INDEX idx_attendance_offenders_protective_measure_id ON attendance_offenders(protective_measure_id);
CREATE INDEX idx_attendance_offenders_is_deleted ON attendance_offenders(is_deleted);
CREATE INDEX idx_attendance_offenders_date ON attendance_offenders(attendance_date);

CREATE TRIGGER update_attendance_offenders_updated_at
    BEFORE UPDATE ON attendance_offenders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create attendance_offender_addresses table
CREATE TABLE attendance_offender_addresses (
    id UUID PRIMARY KEY,
    attendance_id UUID NOT NULL,
    street VARCHAR(255),
    number VARCHAR(50),
    district VARCHAR(100),
    city_id UUID,
    zip_code VARCHAR(20),
    complement VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT fk_attendance_offender_addresses_attendance FOREIGN KEY (attendance_id) REFERENCES attendance_offenders(id) ON DELETE RESTRICT,
    CONSTRAINT fk_attendance_offender_addresses_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT
);

CREATE INDEX idx_attendance_offender_addresses_attendance_id ON attendance_offender_addresses(attendance_id);
CREATE INDEX idx_attendance_offender_addresses_is_deleted ON attendance_offender_addresses(is_deleted);

CREATE TRIGGER update_attendance_offender_addresses_updated_at
    BEFORE UPDATE ON attendance_offender_addresses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
