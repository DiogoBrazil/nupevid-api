-- Mark orphan attendances (without a protective measure) as soft-deleted before removing them.
-- NOT NULL is structural and rejects NULL values regardless of is_deleted flag, so the rows
-- must be physically removed. The soft-delete UPDATE is kept as an audit marker.
UPDATE attendance_victims
SET is_deleted = TRUE, updated_at = NOW()
WHERE protective_measure_id IS NULL AND is_deleted = FALSE;

UPDATE attendance_offenders
SET is_deleted = TRUE, updated_at = NOW()
WHERE protective_measure_id IS NULL AND is_deleted = FALSE;

DELETE FROM attendance_victims WHERE protective_measure_id IS NULL;
DELETE FROM attendance_offenders WHERE protective_measure_id IS NULL;

ALTER TABLE attendance_victims
    ALTER COLUMN protective_measure_id SET NOT NULL;

ALTER TABLE attendance_offenders
    ALTER COLUMN protective_measure_id SET NOT NULL;
