-- Auto-update updated_at on every row change for laptops and users tables.
-- This ensures updated_at is always current regardless of whether the
-- application-layer query includes SET updated_at = NOW().

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_laptops_updated_at
BEFORE UPDATE ON laptops
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();