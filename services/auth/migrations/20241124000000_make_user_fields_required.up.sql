-- Make reg_number, year_joined, and phone_number NOT NULL
-- First, update any existing NULL values with defaults
UPDATE users 
SET reg_number = 'UNKNOWN' 
WHERE reg_number IS NULL;

UPDATE users 
SET year_joined = 2000 
WHERE year_joined IS NULL;

UPDATE users 
SET phone_number = '0000000000' 
WHERE phone_number IS NULL;

-- Now make the columns NOT NULL
ALTER TABLE users 
    ALTER COLUMN reg_number SET NOT NULL,
    ALTER COLUMN year_joined SET NOT NULL,
    ALTER COLUMN phone_number SET NOT NULL;
