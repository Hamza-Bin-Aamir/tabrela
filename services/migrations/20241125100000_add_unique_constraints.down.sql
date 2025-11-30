-- Remove constraints
ALTER TABLE users 
    DROP CONSTRAINT IF EXISTS users_phone_number_unique,
    DROP CONSTRAINT IF EXISTS users_reg_number_unique,
    DROP CONSTRAINT IF EXISTS users_year_joined_check,
    DROP CONSTRAINT IF EXISTS users_reg_number_format_check,
    DROP CONSTRAINT IF EXISTS users_phone_number_format_check;
