-- Change latitude and longitude columns from NUMERIC to DOUBLE PRECISION
-- This fixes the type mismatch with Rust's f64 type

ALTER TABLE attendances
    ALTER COLUMN latitude TYPE DOUBLE PRECISION,
    ALTER COLUMN longitude TYPE DOUBLE PRECISION;
