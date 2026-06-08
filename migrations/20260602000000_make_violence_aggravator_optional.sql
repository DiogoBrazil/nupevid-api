-- Make violence_aggravator optional on attendance_offenders.
-- The aggravator may now be left unspecified when registering an offender attendance.
ALTER TABLE attendance_offenders
    ALTER COLUMN violence_aggravator DROP NOT NULL;
