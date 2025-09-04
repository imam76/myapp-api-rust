-- Down migration: contacts

-- Dropping the table will also remove its indexes, constraints, triggers, and RLS policies.
DROP TABLE IF EXISTS contacts;
