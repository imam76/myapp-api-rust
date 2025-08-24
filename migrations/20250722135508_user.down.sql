-- Down migration: users

-- Drop trigger and indexes
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP INDEX IF EXISTS idx_users_username;
DROP INDEX IF EXISTS idx_users_email;

-- Drop table
DROP TABLE IF EXISTS users;

-- Drop shared helper function and extension (safe to drop when rolling back all migrations)
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP EXTENSION IF EXISTS pgcrypto;