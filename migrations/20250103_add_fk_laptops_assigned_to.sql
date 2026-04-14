-- Add foreign key constraint on laptops.assigned_to referencing users(id).
-- ON DELETE SET NULL ensures that when a user is deleted, any laptops
-- previously assigned to them will have assigned_to set to NULL.
-- The application layer still explicitly sets status = 'available' for
-- a clean transition, but this constraint enforces referential integrity.
ALTER TABLE laptops
ADD CONSTRAINT fk_laptops_assigned_to
FOREIGN KEY (assigned_to) REFERENCES users(id)
ON DELETE SET NULL;