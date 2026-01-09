-- Update enums, victims, offenders, and protective measures to match new payload requirements

-- Extend education_level_enum values
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'education_level_enum' AND e.enumlabel = 'Illiterate'
    ) THEN
        ALTER TYPE education_level_enum ADD VALUE 'Illiterate';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'education_level_enum' AND e.enumlabel = 'Semi-illiterate'
    ) THEN
        ALTER TYPE education_level_enum ADD VALUE 'Semi-illiterate';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'education_level_enum' AND e.enumlabel = 'Master'
    ) THEN
        ALTER TYPE education_level_enum ADD VALUE 'Master';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_enum e
        JOIN pg_type t ON t.oid = e.enumtypid
        WHERE t.typname = 'education_level_enum' AND e.enumlabel = 'Doctorate'
    ) THEN
        ALTER TYPE education_level_enum ADD VALUE 'Doctorate';
    END IF;
END $$;

-- Update victims table
ALTER TABLE victims
    ALTER COLUMN education_level TYPE education_level_enum
    USING CASE
        WHEN education_level IS NULL OR education_level = '' THEN NULL
        WHEN education_level IN (
            'Elementary',
            'High School',
            'College',
            'Postgraduate',
            'Illiterate',
            'Semi-illiterate',
            'Master',
            'Doctorate'
        ) THEN education_level::education_level_enum
        ELSE NULL
    END;

ALTER TABLE victims
    ALTER COLUMN special_needs_type TYPE TEXT[]
    USING CASE
        WHEN special_needs_type IS NULL THEN NULL
        ELSE ARRAY[special_needs_type]
    END;

ALTER TABLE victims
    ALTER COLUMN psychiatric_issues_type TYPE TEXT[]
    USING CASE
        WHEN psychiatric_issues_type IS NULL THEN NULL
        ELSE ARRAY[psychiatric_issues_type]
    END;

DROP INDEX IF EXISTS idx_victims_violence_type;

ALTER TABLE victims
    DROP COLUMN workplace,
    DROP COLUMN violence_type;

-- Update offenders table
ALTER TABLE offenders DROP CONSTRAINT IF EXISTS fk_offenders_victim;
DROP INDEX IF EXISTS idx_offenders_victim_id;
DROP INDEX IF EXISTS idx_offenders_relationship;

ALTER TABLE offenders
    DROP COLUMN victim_id,
    DROP COLUMN relationship_to_victim,
    DROP COLUMN was_drunk_during_assault,
    DROP COLUMN assaults_children;

-- Add protective_measure_status_enum
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'protective_measure_status_enum'
    ) THEN
        CREATE TYPE protective_measure_status_enum AS ENUM ('Valid', 'Revoked', 'Expired');
    END IF;
END $$;

-- Update protective_measures table
ALTER TABLE protective_measures
    ADD COLUMN status protective_measure_status_enum NOT NULL DEFAULT 'Valid',
    ADD COLUMN relationship_to_victim relationship_to_victim_enum,
    ADD COLUMN was_drunk_during_assault BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN assaults_children BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN violence_types violence_type_enum[] NOT NULL DEFAULT ARRAY['Physical']::violence_type_enum[];

UPDATE protective_measures
SET status = CASE
    WHEN is_active THEN 'Valid'::protective_measure_status_enum
    ELSE 'Revoked'::protective_measure_status_enum
END;

UPDATE protective_measures
SET relationship_to_victim = 'Spouse'
WHERE relationship_to_victim IS NULL;

ALTER TABLE protective_measures
    ALTER COLUMN relationship_to_victim SET NOT NULL,
    ALTER COLUMN status DROP DEFAULT,
    ALTER COLUMN was_drunk_during_assault DROP DEFAULT,
    ALTER COLUMN assaults_children DROP DEFAULT,
    ALTER COLUMN violence_types DROP DEFAULT;

DROP INDEX IF EXISTS idx_protective_measures_one_active_per_victim;
DROP INDEX IF EXISTS idx_protective_measures_is_active;

ALTER TABLE protective_measures
    DROP COLUMN is_active;

CREATE UNIQUE INDEX idx_protective_measures_one_active_per_victim
    ON protective_measures(victim_id)
    WHERE status = 'Valid' AND is_deleted = false;
