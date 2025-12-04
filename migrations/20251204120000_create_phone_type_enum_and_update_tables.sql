-- Create phone_type_enum
CREATE TYPE phone_type_enum AS ENUM (
    'Mobile',
    'Residential',
    'Work'
);

-- Update victim_phones table to use the enum
ALTER TABLE victim_phones
    ALTER COLUMN phone_type DROP DEFAULT,
    ALTER COLUMN phone_type TYPE phone_type_enum USING
        CASE
            WHEN phone_type IS NULL THEN NULL
            WHEN phone_type = 'Mobile' THEN 'Mobile'::phone_type_enum
            WHEN phone_type = 'Residential' THEN 'Residential'::phone_type_enum
            WHEN phone_type = 'Work' THEN 'Work'::phone_type_enum
            ELSE 'Mobile'::phone_type_enum
        END;

-- Update offender_phones table to use the enum
ALTER TABLE offender_phones
    ALTER COLUMN phone_type DROP DEFAULT,
    ALTER COLUMN phone_type TYPE phone_type_enum USING
        CASE
            WHEN phone_type IS NULL THEN NULL
            WHEN phone_type = 'Mobile' THEN 'Mobile'::phone_type_enum
            WHEN phone_type = 'Residential' THEN 'Residential'::phone_type_enum
            WHEN phone_type = 'Work' THEN 'Work'::phone_type_enum
            ELSE 'Mobile'::phone_type_enum
        END;
