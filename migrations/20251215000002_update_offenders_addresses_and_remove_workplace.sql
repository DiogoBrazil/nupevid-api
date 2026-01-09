-- Add address_type to offender_addresses, remove offender_work_addresses, and drop workplace

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'address_type_enum'
    ) THEN
        CREATE TYPE address_type_enum AS ENUM (
            'Residential',
            'Work',
            'Correspondence',
            'Commercial',
            'Institutional',
            'Temporary',
            'Other'
        );
    END IF;
END $$;

ALTER TABLE offender_addresses
    ADD COLUMN IF NOT EXISTS address_type address_type_enum;

UPDATE offender_addresses
SET address_type = 'Residential'
WHERE address_type IS NULL;

ALTER TABLE offender_addresses
    ALTER COLUMN address_type SET NOT NULL;

DROP TABLE IF EXISTS offender_work_addresses;

ALTER TABLE offenders
    DROP COLUMN IF EXISTS workplace;
