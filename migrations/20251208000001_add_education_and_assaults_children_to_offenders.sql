-- Create education level enum
CREATE TYPE education_level_enum AS ENUM (
    'Elementary',
    'High School',
    'College',
    'Postgraduate'
);

-- Add education_level column to offenders table
ALTER TABLE offenders
    ADD COLUMN education_level education_level_enum NOT NULL DEFAULT 'Elementary';

-- Add assaults_children column to offenders table
ALTER TABLE offenders
    ADD COLUMN assaults_children BOOLEAN NOT NULL DEFAULT FALSE;

-- Remove default values after adding (to make them required for new inserts)
ALTER TABLE offenders
    ALTER COLUMN education_level DROP DEFAULT,
    ALTER COLUMN assaults_children DROP DEFAULT;
