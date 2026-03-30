-- Phase 60.1: Domain ↔ OAuth2 client assignment link table
CREATE TABLE IF NOT EXISTS domain_oauth2_client (
    domain_id        UUID NOT NULL REFERENCES domain(id) ON DELETE CASCADE,
    oauth2_client_id UUID NOT NULL,
    PRIMARY KEY (domain_id, oauth2_client_id)
);

CREATE INDEX IF NOT EXISTS idx_domain_oauth2_domain ON domain_oauth2_client(domain_id);
