-- Migration 014: Add assigned_customers column to dashboard
-- Matches ThingsBoard Java schema: assigned_customers varchar(1000000)
-- Stores JSON array of ShortCustomerInfo objects

ALTER TABLE dashboard
    ADD COLUMN IF NOT EXISTS assigned_customers VARCHAR(1000000);
