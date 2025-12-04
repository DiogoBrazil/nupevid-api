-- Add new fields to victims table

-- Optional fields
ALTER TABLE victims ADD COLUMN education_level VARCHAR(100);
ALTER TABLE victims ADD COLUMN occupation VARCHAR(100);
ALTER TABLE victims ADD COLUMN workplace VARCHAR(200);

-- Required fields with defaults
ALTER TABLE victims ADD COLUMN violence_type violence_type_enum NOT NULL DEFAULT 'Physical';
ALTER TABLE victims ADD COLUMN has_children has_children_enum NOT NULL DEFAULT 'No';
ALTER TABLE victims ADD COLUMN children_count INTEGER;
ALTER TABLE victims ADD COLUMN has_special_needs BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE victims ADD COLUMN special_needs_type VARCHAR(200);
ALTER TABLE victims ADD COLUMN uses_alcohol BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE victims ADD COLUMN uses_drugs BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE victims ADD COLUMN has_psychiatric_issues BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE victims ADD COLUMN psychiatric_issues_type VARCHAR(200);

-- Add indexes for frequently queried fields
CREATE INDEX idx_victims_violence_type ON victims(violence_type);
CREATE INDEX idx_victims_has_children ON victims(has_children);
CREATE INDEX idx_victims_has_special_needs ON victims(has_special_needs);
