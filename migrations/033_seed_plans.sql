-- Phase 70: Seed default subscription plans
-- Prices in USD cents. Annual = 10 months price (2 months free).
-- -1 means unlimited for numeric limits.

INSERT INTO subscription_plan (
    id, created_time, name, display_name, description,
    price_monthly, price_annually,
    stripe_price_id_monthly, stripe_price_id_annually,
    max_devices, max_users, max_assets, max_dashboards, max_rule_chains,
    max_edges, max_transport_msgs_month, max_js_execs_month,
    max_emails_month, max_sms_month, max_alarms, max_api_keys,
    feature_white_label, feature_edge_computing, feature_advanced_rbac,
    feature_audit_log, feature_sso, feature_api_export,
    sort_order, is_active
) VALUES
(
    gen_random_uuid(),
    EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    'free', 'Free', 'Get started with VieLang IoT. No credit card required.',
    0, 0,
    NULL, NULL,
    10, 3, 10, 5, 2,
    0, 50000, 5000,
    50, 0, 100, 1,
    false, false, false, false, false, false,
    0, true
),
(
    gen_random_uuid(),
    EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    'starter', 'Starter', 'For small teams and growing IoT deployments.',
    2900, 29000,
    NULL, NULL,
    100, 10, 50, 20, 5,
    0, 1000000, 100000,
    500, 0, 1000, 3,
    false, false, false, true, false, false,
    1, true
),
(
    gen_random_uuid(),
    EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    'pro', 'Pro', 'For professional teams with advanced IoT needs.',
    9900, 99000,
    NULL, NULL,
    1000, 50, 500, 100, 20,
    5, 10000000, 1000000,
    5000, 500, 10000, 10,
    true, true, true, true, true, true,
    2, true
),
(
    gen_random_uuid(),
    EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    'enterprise', 'Enterprise', 'Unlimited scale for large enterprises. Contact sales for pricing.',
    0, 0,
    NULL, NULL,
    -1, -1, -1, -1, -1,
    -1, -1, -1,
    -1, -1, -1, -1,
    true, true, true, true, true, true,
    3, true
)
ON CONFLICT (name) DO NOTHING;
