-- P8: Advanced Geofencing — PostGIS-backed polygons and circles.
-- Wraps all PostGIS DDL in a DO block so the migration is a no-op when
-- PostGIS is not installed (e.g. postgres:alpine without the extension).
-- Set geofence.enabled = true in config only when PostGIS is available.

DO $$
BEGIN
    -- Install extension (fails silently if not available)
    CREATE EXTENSION IF NOT EXISTS postgis;

    -- Geofence definitions
    EXECUTE $sql$
        CREATE TABLE IF NOT EXISTS tb_geofence (
            id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id    UUID        NOT NULL,
            name         TEXT        NOT NULL,
            type         TEXT        NOT NULL CHECK (type IN ('POLYGON', 'CIRCLE')),
            polygon      GEOGRAPHY(POLYGON, 4326),
            center       GEOGRAPHY(POINT, 4326),
            radius_m     DOUBLE PRECISION,
            created_time BIGINT      NOT NULL,
            extra_info   JSONB
        )
    $sql$;

    EXECUTE $sql$
        CREATE INDEX IF NOT EXISTS idx_geofence_polygon ON tb_geofence USING GIST(polygon)
            WHERE polygon IS NOT NULL
    $sql$;
    EXECUTE $sql$
        CREATE INDEX IF NOT EXISTS idx_geofence_circle  ON tb_geofence USING GIST(center)
            WHERE center IS NOT NULL
    $sql$;
    EXECUTE $sql$
        CREATE INDEX IF NOT EXISTS idx_geofence_tenant  ON tb_geofence(tenant_id)
    $sql$;

    -- Device last-known location
    EXECUTE $sql$
        CREATE TABLE IF NOT EXISTS device_location (
            device_id    UUID    PRIMARY KEY REFERENCES device(id) ON DELETE CASCADE,
            location     GEOGRAPHY(POINT, 4326) NOT NULL,
            updated_time BIGINT  NOT NULL
        )
    $sql$;
    EXECUTE $sql$
        CREATE INDEX IF NOT EXISTS idx_device_location ON device_location USING GIST(location)
    $sql$;

EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'PostGIS not available — geofence tables skipped. Install PostGIS and re-run migrations to enable geofencing.';
END;
$$;
