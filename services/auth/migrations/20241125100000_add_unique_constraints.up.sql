-- Add unique constraints to phone_number and reg_number
ALTER TABLE users 
    ADD CONSTRAINT users_phone_number_unique UNIQUE (phone_number),
    ADD CONSTRAINT users_reg_number_unique UNIQUE (reg_number);

-- Add check constraint for year_joined format (20XX)
ALTER TABLE users 
    ADD CONSTRAINT users_year_joined_check CHECK (year_joined >= 2000 AND year_joined <= 2099);

-- Add check constraint for reg_number format (20XXXXX - 7 digits starting with 20)
ALTER TABLE users 
    ADD CONSTRAINT users_reg_number_format_check CHECK (reg_number ~ '^20\d{5}$');

-- Add check constraint for phone_number format (must start with + and country code)
ALTER TABLE users 
    ADD CONSTRAINT users_phone_number_format_check CHECK (phone_number ~ '^\+\d{1,3}\d{9,15}$');
