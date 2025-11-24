-- Add permission_policies JSONB column to users table
-- This column stores policy-based permissions for CITY_ADMIN and CITY_USER profiles
-- Format: {"policy_name": ["city_id1", "city_id2", ...]}
-- ROOT users have implicit total access and don't need policies

ALTER TABLE users
ADD COLUMN permission_policies JSONB DEFAULT '{}' NOT NULL;

-- Add index for JSONB queries
CREATE INDEX idx_users_permission_policies ON users USING GIN (permission_policies);
