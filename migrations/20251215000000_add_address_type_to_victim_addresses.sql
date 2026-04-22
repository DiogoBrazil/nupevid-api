-- Add address_type_enum and address_type column to victim_addresses

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

ALTER TABLE victim_addresses
    ADD COLUMN IF NOT EXISTS address_type address_type_enum;

UPDATE victim_addresses
SET address_type = 'Residential'
WHERE address_type IS NULL;

ALTER TABLE victim_addresses
    ALTER COLUMN address_type SET NOT NULL;
