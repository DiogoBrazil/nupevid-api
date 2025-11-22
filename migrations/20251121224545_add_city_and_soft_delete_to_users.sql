-- Add city_id column (nullable for ROOT users)
ALTER TABLE users
ADD COLUMN city_id UUID NULL,
ADD CONSTRAINT fk_users_city FOREIGN KEY (city_id) REFERENCES cities(id) ON DELETE RESTRICT;

-- Add soft delete column
ALTER TABLE users
ADD COLUMN is_deleted BOOLEAN DEFAULT FALSE NOT NULL;

-- Add indexes
CREATE INDEX idx_users_city_id ON users(city_id);
CREATE INDEX idx_users_is_deleted ON users(is_deleted);

-- Add partial unique index: only one active CITY_ADMIN per city
CREATE UNIQUE INDEX idx_users_one_city_admin_per_city
ON users(city_id)
WHERE profile = 'CITY_ADMIN' AND is_deleted = false;
