-- Update psychiatric_issues_type to text[] and derive has_psychiatric_issues

ALTER TABLE offenders
    ALTER COLUMN psychiatric_issues_type TYPE TEXT[]
    USING (
        CASE
            WHEN psychiatric_issues_type IS NULL OR psychiatric_issues_type = '' THEN NULL
            ELSE ARRAY[psychiatric_issues_type]
        END
    );

UPDATE offenders
SET has_psychiatric_issues = CASE
    WHEN psychiatric_issues_type IS NOT NULL
        AND array_length(psychiatric_issues_type, 1) > 0
        THEN true
    ELSE false
END;

ALTER TABLE offenders
    ALTER COLUMN has_psychiatric_issues SET DEFAULT false,
    ALTER COLUMN has_psychiatric_issues SET NOT NULL;
