CREATE TABLE IF NOT EXISTS laptops (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    brand          VARCHAR(100) NOT NULL,
    model          VARCHAR(100) NOT NULL,
    serial_number  VARCHAR(100) UNIQUE NOT NULL,
    status         VARCHAR(50)  NOT NULL DEFAULT 'available'
                   CHECK (status IN ('available', 'assigned', 'in_repair', 'retired')),
    assigned_to    UUID,   -- references users(id); handled at application layer
    purchase_date  DATE    NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_laptops_status        ON laptops(status);
CREATE INDEX IF NOT EXISTS idx_laptops_assigned_to   ON laptops(assigned_to);
CREATE INDEX IF NOT EXISTS idx_laptops_serial_number ON laptops(serial_number);
