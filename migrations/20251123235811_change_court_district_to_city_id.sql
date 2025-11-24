-- Change court_district from String to court_district_id (UUID FK to cities)
-- This represents the city where the protective measure was issued

ALTER TABLE protective_measures
    DROP COLUMN IF EXISTS court_district;

ALTER TABLE protective_measures
    ADD COLUMN court_district_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- Remove the default after adding the column
ALTER TABLE protective_measures
    ALTER COLUMN court_district_id DROP DEFAULT;

-- Add foreign key constraint
ALTER TABLE protective_measures
    ADD CONSTRAINT fk_protective_measures_court_district
    FOREIGN KEY (court_district_id) REFERENCES cities(id) ON DELETE RESTRICT;

-- Add index for the new column
CREATE INDEX idx_protective_measures_court_district_id ON protective_measures(court_district_id);
