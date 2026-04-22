-- Migration to change violence_aggravator_enum values from Portuguese to English

-- Step 1: Create new enum type with English values
CREATE TYPE violence_aggravator_enum_new AS ENUM ('AlcoholUse', 'DrugUse', 'PsychiatricIssues', 'Other');

-- Step 2: Add temporary column with new type
ALTER TABLE attendance_offenders
ADD COLUMN violence_aggravator_new violence_aggravator_enum_new;

-- Step 3: Migrate data from old to new column
UPDATE attendance_offenders
SET violence_aggravator_new = CASE violence_aggravator::text
    WHEN 'UsoBebidaAlcoolica' THEN 'AlcoholUse'::violence_aggravator_enum_new
    WHEN 'UsoDrogas' THEN 'DrugUse'::violence_aggravator_enum_new
    WHEN 'ProblemasPsiquiatricos' THEN 'PsychiatricIssues'::violence_aggravator_enum_new
    WHEN 'Outros' THEN 'Other'::violence_aggravator_enum_new
END;

-- Step 4: Drop old column
ALTER TABLE attendance_offenders DROP COLUMN violence_aggravator;

-- Step 5: Rename new column to original name
ALTER TABLE attendance_offenders RENAME COLUMN violence_aggravator_new TO violence_aggravator;

-- Step 6: Make column NOT NULL (it was NOT NULL before)
ALTER TABLE attendance_offenders ALTER COLUMN violence_aggravator SET NOT NULL;

-- Step 7: Drop old enum type
DROP TYPE violence_aggravator_enum;

-- Step 8: Rename new enum type to original name
ALTER TYPE violence_aggravator_enum_new RENAME TO violence_aggravator_enum;
