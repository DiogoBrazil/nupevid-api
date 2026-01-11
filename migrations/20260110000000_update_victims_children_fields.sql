-- Update has_children to boolean and add is_pregnant

ALTER TABLE victims ADD COLUMN is_pregnant BOOLEAN;

UPDATE victims
SET is_pregnant = CASE WHEN has_children = 'Pregnant' THEN true ELSE false END;

ALTER TABLE victims
    ALTER COLUMN has_children DROP DEFAULT,
    ALTER COLUMN has_children TYPE BOOLEAN
        USING (children_count IS NOT NULL);

ALTER TABLE victims
    ALTER COLUMN has_children SET DEFAULT false,
    ALTER COLUMN has_children SET NOT NULL;

DROP TYPE has_children_enum;
