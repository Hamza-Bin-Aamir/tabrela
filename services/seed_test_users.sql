-- ============================================================================
-- DEV ONLY: Seed 20 Test Users for Allocation Testing
-- ============================================================================
-- Run this script ONLY in development environment:
--   psql $DATABASE_URL -f services/seed_test_users.sql
--
-- These users have properly formatted data matching your schema constraints:
-- - reg_number: 20XXXXX format
-- - phone_number: +923XXXXXXXXX format (Pakistani mobile)
-- - email_verified: true (so they can be used immediately)
-- - Password: "password123" (bcrypt hash)
-- ============================================================================

INSERT INTO users (id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at)
VALUES 
    -- Debaters (potential speakers) - Class of 2023
    ('a0000001-0000-0000-0000-000000000001', 'ahmad_khan', 'ahmad.khan@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt1', '2023001', 2023, '+923001234001', true, NOW()),
    ('a0000001-0000-0000-0000-000000000002', 'fatima_ali', 'fatima.ali@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt2', '2023002', 2023, '+923001234002', true, NOW()),
    ('a0000001-0000-0000-0000-000000000003', 'hassan_raza', 'hassan.raza@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt3', '2023003', 2023, '+923001234003', true, NOW()),
    ('a0000001-0000-0000-0000-000000000004', 'ayesha_malik', 'ayesha.malik@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt4', '2023004', 2023, '+923001234004', true, NOW()),
    ('a0000001-0000-0000-0000-000000000005', 'usman_sheikh', 'usman.sheikh@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt5', '2023005', 2023, '+923001234005', true, NOW()),
    ('a0000001-0000-0000-0000-000000000006', 'zainab_ahmed', 'zainab.ahmed@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt6', '2023006', 2023, '+923001234006', true, NOW()),
    ('a0000001-0000-0000-0000-000000000007', 'bilal_hussain', 'bilal.hussain@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt7', '2023007', 2023, '+923001234007', true, NOW()),
    ('a0000001-0000-0000-0000-000000000008', 'maria_khan', 'maria.khan@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt8', '2023008', 2023, '+923001234008', true, NOW()),
    
    -- Junior debaters - Class of 2024
    ('a0000001-0000-0000-0000-000000000009', 'ali_nawaz', 'ali.nawaz@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt9', '2024001', 2024, '+923001234009', true, NOW()),
    ('a0000001-0000-0000-0000-000000000010', 'sara_iqbal', 'sara.iqbal@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt10', '2024002', 2024, '+923001234010', true, NOW()),
    ('a0000001-0000-0000-0000-000000000011', 'omar_farooq', 'omar.farooq@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt11', '2024003', 2024, '+923001234011', true, NOW()),
    ('a0000001-0000-0000-0000-000000000012', 'hira_shah', 'hira.shah@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt12', '2024004', 2024, '+923001234012', true, NOW()),
    
    -- Senior members (potential adjudicators) - Class of 2021/2022
    ('a0000001-0000-0000-0000-000000000013', 'tariq_mahmood', 'tariq.mahmood@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt13', '2021001', 2021, '+923001234013', true, NOW()),
    ('a0000001-0000-0000-0000-000000000014', 'nadia_aslam', 'nadia.aslam@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt14', '2021002', 2021, '+923001234014', true, NOW()),
    ('a0000001-0000-0000-0000-000000000015', 'imran_qureshi', 'imran.qureshi@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt15', '2022001', 2022, '+923001234015', true, NOW()),
    ('a0000001-0000-0000-0000-000000000016', 'amina_yousaf', 'amina.yousaf@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt16', '2022002', 2022, '+923001234016', true, NOW()),
    
    -- Mixed/Resource members
    ('a0000001-0000-0000-0000-000000000017', 'kamran_abbasi', 'kamran.abbasi@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt17', '2022003', 2022, '+923001234017', true, NOW()),
    ('a0000001-0000-0000-0000-000000000018', 'sana_riaz', 'sana.riaz@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt18', '2023009', 2023, '+923001234018', true, NOW()),
    ('a0000001-0000-0000-0000-000000000019', 'faisal_dar', 'faisal.dar@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt19', '2023010', 2023, '+923001234019', true, NOW()),
    ('a0000001-0000-0000-0000-000000000020', 'rabia_hassan', 'rabia.hassan@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4eSl6bSPxJZ.zJSe', 'randomsalt20', '2023011', 2023, '+923001234020', true, NOW())
ON CONFLICT (id) DO NOTHING;

-- Initialize merit for the test users (triggered automatically, but just in case)
INSERT INTO user_merit (user_id, total_merit, created_at, updated_at)
SELECT id, 0, NOW(), NOW()
FROM users 
WHERE id LIKE 'a0000001-0000-0000-0000-00000000000%'
ON CONFLICT (user_id) DO NOTHING;

-- Output confirmation
SELECT 'Created ' || COUNT(*) || ' test users' AS status FROM users WHERE id LIKE 'a0000001-0000-0000-0000-00000000000%';
