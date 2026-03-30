-- Phase 54: OAuth2 client registration template
CREATE TABLE IF NOT EXISTS oauth2_client_registration_template (
    id                              UUID    PRIMARY KEY,
    created_time                    BIGINT  NOT NULL,
    additional_info                 JSONB,
    provider_id                     VARCHAR(255),
    name                            VARCHAR(255),
    authorization_uri               VARCHAR(255),
    token_uri                       VARCHAR(255),
    scope                           VARCHAR(255),
    user_info_uri                   VARCHAR(255),
    user_name_attribute_name        VARCHAR(255),
    jwk_set_uri                     VARCHAR(255),
    client_authentication_method    VARCHAR(255),
    type                            VARCHAR(31),
    comment                         VARCHAR(255),
    login_button_icon               VARCHAR(255),
    login_button_label              VARCHAR(255),
    help_link                       VARCHAR(255),
    platforms                       VARCHAR(255)
);
