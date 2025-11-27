-- Add unique constraints to phone_number and reg_number (only if they don't exist)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_phone_number_unique') THEN
        ALTER TABLE users ADD CONSTRAINT users_phone_number_unique UNIQUE (phone_number);
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_reg_number_unique') THEN
        ALTER TABLE users ADD CONSTRAINT users_reg_number_unique UNIQUE (reg_number);
    END IF;
END $$;

-- Add check constraint for year_joined format (20XX)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_year_joined_check') THEN
        ALTER TABLE users ADD CONSTRAINT users_year_joined_check CHECK (year_joined >= 2000 AND year_joined <= 2099);
    END IF;
END $$;

-- Add check constraint for reg_number format (20XXXXX - 7 digits starting with 20)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_reg_number_format_check') THEN
        ALTER TABLE users ADD CONSTRAINT users_reg_number_format_check CHECK (reg_number ~ '^20\d{5}$');
    END IF;
END $$;

-- Add check constraint for phone_number format (must start with + and country code)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'users_phone_number_format_check') THEN
        ALTER TABLE users ADD CONSTRAINT users_phone_number_format_check CHECK (phone_number ~ '^\+\d{1,3}\d{9,15}$');
    END IF;
END $$;
