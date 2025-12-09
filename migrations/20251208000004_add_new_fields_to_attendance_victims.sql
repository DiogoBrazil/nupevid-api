-- Create ENUM types for the new fields
CREATE TYPE risk_level AS ENUM ('High', 'Medium', 'Low');
CREATE TYPE offender_freedom_status AS ENUM ('Imprisoned', 'Free', 'Monitored');
CREATE TYPE offender_firearm_access AS ENUM ('Yes', 'No', 'Unknown');

-- Add new fields to attendance_victims table
ALTER TABLE attendance_victims
    ADD COLUMN offender_id UUID,
    ADD COLUMN protective_measure_id UUID,
    ADD COLUMN is_remote BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN risk_level risk_level,
    ADD COLUMN offender_freedom_status offender_freedom_status,
    ADD COLUMN offender_has_firearm_access offender_firearm_access,
    ADD COLUMN needs_legal_assistance BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN needs_psychological_support BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN was_instructed_about_protective_measure_procedures BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN offender_violated_protective_measure BOOLEAN NOT NULL DEFAULT FALSE;

-- Add foreign key constraints
ALTER TABLE attendance_victims
    ADD CONSTRAINT fk_attendance_victims_offender
    FOREIGN KEY (offender_id) REFERENCES offenders(id) ON DELETE RESTRICT;

ALTER TABLE attendance_victims
    ADD CONSTRAINT fk_attendance_victims_protective_measure
    FOREIGN KEY (protective_measure_id) REFERENCES protective_measures(id) ON DELETE RESTRICT;

-- Create indexes for the new foreign keys
CREATE INDEX idx_attendance_victims_offender_id ON attendance_victims(offender_id);
CREATE INDEX idx_attendance_victims_protective_measure_id ON attendance_victims(protective_measure_id);
