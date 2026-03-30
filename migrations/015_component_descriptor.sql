-- Component Descriptor table
-- Mirrors ThingsBoard Java schema-entities.sql component_descriptor table.
-- In ThingsBoard Java, descriptors are discovered from classpath and persisted here.

CREATE TABLE IF NOT EXISTS component_descriptor (
    id uuid NOT NULL CONSTRAINT component_descriptor_pkey PRIMARY KEY,
    created_time bigint NOT NULL,
    actions varchar(255),
    clazz varchar UNIQUE,
    configuration_descriptor varchar,
    configuration_version int DEFAULT 0,
    name varchar(255),
    scope varchar(255),
    type varchar(255),
    clustering_mode varchar(255),
    has_queue_name boolean DEFAULT false
);
