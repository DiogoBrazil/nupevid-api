ALTER TABLE users
ALTER COLUMN registration TYPE VARCHAR(50) USING registration::VARCHAR;

DROP INDEX IF EXISTS idx_users_registration;
CREATE INDEX idx_users_registration ON users(registration);
