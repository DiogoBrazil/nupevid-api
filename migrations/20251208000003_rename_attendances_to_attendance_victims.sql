-- Rename attendances table to attendance_victims
ALTER TABLE attendances RENAME TO attendance_victims;

-- Rename attendance_addresses table to attendance_victim_addresses
ALTER TABLE attendance_addresses RENAME TO attendance_victim_addresses;

-- Rename constraint on attendance_victim_addresses table
ALTER TABLE attendance_victim_addresses
    DROP CONSTRAINT fk_attendance_addresses_attendance;

ALTER TABLE attendance_victim_addresses
    ADD CONSTRAINT fk_attendance_victim_addresses_attendance_victim
    FOREIGN KEY (attendance_id) REFERENCES attendance_victims(id) ON DELETE RESTRICT;

-- Rename indexes
ALTER INDEX idx_attendances_victim_id RENAME TO idx_attendance_victims_victim_id;
ALTER INDEX idx_attendances_is_deleted RENAME TO idx_attendance_victims_is_deleted;
ALTER INDEX idx_attendances_date RENAME TO idx_attendance_victims_date;

ALTER INDEX idx_attendance_addresses_attendance_id RENAME TO idx_attendance_victim_addresses_attendance_id;
ALTER INDEX idx_attendance_addresses_is_deleted RENAME TO idx_attendance_victim_addresses_is_deleted;
