-- Migration 013: Seed default tenant profile
-- Matches ThingsBoard Java default data on first install

INSERT INTO tenant_profile (
    id, created_time, name, description,
    is_default, isolated_vl_rule_engine, profile_data, version
)
VALUES (
    '13814000-1dd2-11b2-8080-808080808080',
    0,
    'Default',
    'Default tenant profile',
    TRUE,
    FALSE,
    '{
        "configuration": {
            "type": "DEFAULT",
            "maxDevices": 0,
            "maxAssets": 0,
            "maxCustomers": 0,
            "maxUsers": 0,
            "maxDashboards": 0,
            "maxRuleChains": 0,
            "maxResourcesInBytes": 0,
            "maxOtaPackagesInBytes": 0,
            "maxTransportMessages": 0,
            "maxTransportDataPoints": 0,
            "maxREExecutions": 0,
            "maxJSExecutions": 0,
            "maxDPStorageDays": 0,
            "maxRuleNodeExecutionsPerMessage": 0,
            "maxEmails": 0,
            "maxSms": 0,
            "maxCreatedAlarms": 0,
            "defaultStorageTtlDays": 0,
            "alarmsTtlDays": 0,
            "rpcTtlDays": 0,
            "warnThreshold": 0
        }
    }'::jsonb,
    1
)
ON CONFLICT (id) DO NOTHING;
