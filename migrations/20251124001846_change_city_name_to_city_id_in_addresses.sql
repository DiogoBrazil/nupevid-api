-- Change city_name to city_id in victim_addresses and attendance_addresses tables
-- Also remove state column since it's now derived from the city

-- victim_addresses
ALTER TABLE victim_addresses DROP COLUMN IF EXISTS city_name;
ALTER TABLE victim_addresses DROP COLUMN IF EXISTS state;
ALTER TABLE victim_addresses ADD COLUMN IF NOT EXISTS city_id UUID;
ALTER TABLE victim_addresses
    ADD CONSTRAINT fk_victim_addresses_city
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT;
CREATE INDEX IF NOT EXISTS idx_victim_addresses_city_id ON victim_addresses(city_id);

-- attendance_addresses
ALTER TABLE attendance_addresses DROP COLUMN IF EXISTS city_name;
ALTER TABLE attendance_addresses DROP COLUMN IF EXISTS state;
ALTER TABLE attendance_addresses ADD COLUMN IF NOT EXISTS city_id UUID;
ALTER TABLE attendance_addresses
    ADD CONSTRAINT fk_attendance_addresses_city
    FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT;
CREATE INDEX IF NOT EXISTS idx_attendance_addresses_city_id ON attendance_addresses(city_id);
