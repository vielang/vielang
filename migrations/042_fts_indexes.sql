-- Phase P5 — Full-Text Search: tsvector columns, triggers, GIN indexes
-- Tables: device, asset, customer, tb_user, entity_view, edge, dashboard

-- ── 1. Add search_text columns ────────────────────────────────────────────────

ALTER TABLE device      ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE asset       ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE customer    ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE tb_user     ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE entity_view ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE edge        ADD COLUMN IF NOT EXISTS search_text TSVECTOR;
ALTER TABLE dashboard   ADD COLUMN IF NOT EXISTS search_text TSVECTOR;

-- ── 2. Trigger functions (table-specific — each has different searchable cols) ─

CREATE OR REPLACE FUNCTION update_device_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.name,  '') || ' ' ||
        coalesce(NEW.label, '') || ' ' ||
        coalesce(NEW.type,  '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_asset_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.name,  '') || ' ' ||
        coalesce(NEW.label, '') || ' ' ||
        coalesce(NEW.type,  '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_customer_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.title, '') || ' ' ||
        coalesce(NEW.email, '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_user_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.first_name, '') || ' ' ||
        coalesce(NEW.last_name,  '') || ' ' ||
        coalesce(NEW.email,      '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_entity_view_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.name, '') || ' ' ||
        coalesce(NEW.type, '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_edge_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.name,  '') || ' ' ||
        coalesce(NEW.label, '') || ' ' ||
        coalesce(NEW.type,  '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_dashboard_search_text() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_text := to_tsvector('english',
        coalesce(NEW.title, '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ── 3. Attach triggers ────────────────────────────────────────────────────────

DROP TRIGGER IF EXISTS trig_device_fts      ON device;
DROP TRIGGER IF EXISTS trig_asset_fts       ON asset;
DROP TRIGGER IF EXISTS trig_customer_fts    ON customer;
DROP TRIGGER IF EXISTS trig_user_fts        ON tb_user;
DROP TRIGGER IF EXISTS trig_entity_view_fts ON entity_view;
DROP TRIGGER IF EXISTS trig_edge_fts        ON edge;
DROP TRIGGER IF EXISTS trig_dashboard_fts   ON dashboard;

CREATE TRIGGER trig_device_fts
    BEFORE INSERT OR UPDATE ON device
    FOR EACH ROW EXECUTE FUNCTION update_device_search_text();

CREATE TRIGGER trig_asset_fts
    BEFORE INSERT OR UPDATE ON asset
    FOR EACH ROW EXECUTE FUNCTION update_asset_search_text();

CREATE TRIGGER trig_customer_fts
    BEFORE INSERT OR UPDATE ON customer
    FOR EACH ROW EXECUTE FUNCTION update_customer_search_text();

CREATE TRIGGER trig_user_fts
    BEFORE INSERT OR UPDATE ON tb_user
    FOR EACH ROW EXECUTE FUNCTION update_user_search_text();

CREATE TRIGGER trig_entity_view_fts
    BEFORE INSERT OR UPDATE ON entity_view
    FOR EACH ROW EXECUTE FUNCTION update_entity_view_search_text();

CREATE TRIGGER trig_edge_fts
    BEFORE INSERT OR UPDATE ON edge
    FOR EACH ROW EXECUTE FUNCTION update_edge_search_text();

CREATE TRIGGER trig_dashboard_fts
    BEFORE INSERT OR UPDATE ON dashboard
    FOR EACH ROW EXECUTE FUNCTION update_dashboard_search_text();

-- ── 4. GIN indexes ────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_device_fts      ON device      USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_asset_fts       ON asset       USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_customer_fts    ON customer    USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_user_fts        ON tb_user     USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_entity_view_fts ON entity_view USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_edge_fts        ON edge        USING GIN(search_text);
CREATE INDEX IF NOT EXISTS idx_dashboard_fts   ON dashboard   USING GIN(search_text);

-- GIN index cho JSONB device_data (attributes/profile data queries)
CREATE INDEX IF NOT EXISTS idx_device_data_gin ON device USING GIN(device_data jsonb_path_ops);

-- ── 5. Backfill existing rows ─────────────────────────────────────────────────

UPDATE device SET search_text = to_tsvector('english',
    coalesce(name,  '') || ' ' ||
    coalesce(label, '') || ' ' ||
    coalesce(type,  ''));

UPDATE asset SET search_text = to_tsvector('english',
    coalesce(name,  '') || ' ' ||
    coalesce(label, '') || ' ' ||
    coalesce(type,  ''));

UPDATE customer SET search_text = to_tsvector('english',
    coalesce(title, '') || ' ' ||
    coalesce(email, ''));

UPDATE tb_user SET search_text = to_tsvector('english',
    coalesce(first_name, '') || ' ' ||
    coalesce(last_name,  '') || ' ' ||
    coalesce(email,      ''));

UPDATE entity_view SET search_text = to_tsvector('english',
    coalesce(name, '') || ' ' ||
    coalesce(type, ''));

UPDATE edge SET search_text = to_tsvector('english',
    coalesce(name,  '') || ' ' ||
    coalesce(label, '') || ' ' ||
    coalesce(type,  ''));

UPDATE dashboard SET search_text = to_tsvector('english',
    coalesce(title, ''));
