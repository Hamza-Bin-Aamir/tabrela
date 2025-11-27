-- Rollback: Make reg_number, year_joined, and phone_number nullable again
ALTER TABLE users 
    ALTER COLUMN reg_number DROP NOT NULL,
    ALTER COLUMN year_joined DROP NOT NULL,
    ALTER COLUMN phone_number DROP NOT NULL;
