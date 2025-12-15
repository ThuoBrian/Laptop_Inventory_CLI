CREATE TABLE IF NOT EXISTS laptops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    brand VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    serial_number VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('available', 'assigned', 'in_repair', 'retired')),
    assigned_to UUID,
    purchase_date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_laptops_status ON LAPTOPS(status);
CREATE INDEX idx_laptops_assigned_to ON LAPTOPS(assigned_to);
CREATE INDEX idx_laptops_purchase_date ON LAPTOPS(serial_number);
