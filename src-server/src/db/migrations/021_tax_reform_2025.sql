-- Tax Reform Acts 2025 (effective 2026)
-- Author: Quadri Atharu
-- Updates: CIT brackets, VAT threshold, Education Tax, CGT progressive, PAYE bands, minimum tax threshold

-- 1. Update tax_configs seed data: old CIT thresholds replaced
-- CIT small: <=50M (0%), medium: 50M-250M (15%), large: >250M (25%)
-- Education Tax: 1% (was 2%)
-- CGT: progressive 10%/15%/20% (was flat 10%)
-- VAT registration threshold: 50M (was 25M)
-- Minimum tax threshold: 50M (was 25M)
-- PAYE bands: 0%/15%/20%/25%/30%/35%

-- Add new tax type enum value if not exists
ALTER TABLE tax_configs ALTER COLUMN tax_type TYPE VARCHAR(20);

-- Insert updated default tax configurations for new companies
-- These are templates; actual company rates are in tax_configs per company_id

-- 2. Update existing license feature descriptions for NRS rename (if table exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'license_features') THEN
        UPDATE license_features SET description = 'PAYE tax computation and filing (NRS)' WHERE code = 'tax_paye';
        UPDATE license_features SET description = 'All Tax Types — VAT, WHT, PAYE, CIT, CGT, Stamp Duty, Education Tax (NRS)' WHERE code = 'tax_all';
    END IF;
END $$;

-- 3. Add CGT progressive rate columns
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cgt_tier_1_rate NUMERIC(5,2) DEFAULT 10.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cgt_tier_1_threshold NUMERIC(18,2) DEFAULT 50000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cgt_tier_2_rate NUMERIC(5,2) DEFAULT 15.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cgt_tier_2_threshold NUMERIC(18,2) DEFAULT 250000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cgt_tier_3_rate NUMERIC(5,2) DEFAULT 20.00;

-- 4. Add PAYE bracket columns for Tax Reform 2025
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_1_max NUMERIC(18,2) DEFAULT 800000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_1_rate NUMERIC(5,2) DEFAULT 0.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_2_max NUMERIC(18,2) DEFAULT 3200000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_2_rate NUMERIC(5,2) DEFAULT 15.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_3_max NUMERIC(18,2) DEFAULT 7200000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_3_rate NUMERIC(5,2) DEFAULT 20.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_4_max NUMERIC(18,2) DEFAULT 14000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_4_rate NUMERIC(5,2) DEFAULT 25.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_5_max NUMERIC(18,2) DEFAULT 25000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_5_rate NUMERIC(5,2) DEFAULT 30.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS paye_band_6_rate NUMERIC(5,2) DEFAULT 35.00;

-- 5. Add WHT individual recipient rate column
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS wht_individual_rate NUMERIC(5,2) DEFAULT 5.00;

-- 6. Add VAT registration threshold column
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS vat_registration_threshold NUMERIC(18,2) DEFAULT 50000000.00;

-- 7. Add CIT bracket thresholds
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cit_small_threshold NUMERIC(18,2) DEFAULT 50000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cit_medium_threshold NUMERIC(18,2) DEFAULT 250000000.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cit_small_rate NUMERIC(5,2) DEFAULT 0.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cit_medium_rate NUMERIC(5,2) DEFAULT 15.00;
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS cit_large_rate NUMERIC(5,2) DEFAULT 25.00;

-- 8. Add minimum tax threshold
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS minimum_tax_threshold NUMERIC(18,2) DEFAULT 50000000.00;

-- 9. Update education tax rate default
ALTER TABLE tax_configs ADD COLUMN IF NOT EXISTS edu_tax_rate NUMERIC(5,2) DEFAULT 1.00;
