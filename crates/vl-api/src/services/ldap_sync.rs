use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use vl_auth::ldap::{LdapAuthProvider, LdapConfig};
use vl_core::entities::{Authority, User};
use vl_dao::LdapConfigDao;
use vl_dao::postgres::user::UserDao;

pub struct LdapSyncService {
    ldap_config_dao: Arc<LdapConfigDao>,
    user_dao:        Arc<UserDao>,
}

impl LdapSyncService {
    pub fn new(ldap_config_dao: Arc<LdapConfigDao>, user_dao: Arc<UserDao>) -> Self {
        Self { ldap_config_dao, user_dao }
    }

    pub async fn run_sync_loop(self: Arc<Self>, interval_secs: u64) {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        ticker.tick().await; // skip first immediate tick
        loop {
            ticker.tick().await;
            match self.ldap_config_dao.find_all_tenants_with_ldap().await {
                Err(e) => error!("LDAP sync: failed to load tenant configs: {e}"),
                Ok(tenant_ids) => {
                    for tenant_id in tenant_ids {
                        if let Err(e) = self.sync_tenant(tenant_id).await {
                            error!(tenant=%tenant_id, "LDAP sync failed: {e}");
                        }
                    }
                }
            }
        }
    }

    async fn sync_tenant(&self, tenant_id: Uuid) -> Result<(), String> {
        let config_row = self.ldap_config_dao.find_by_tenant(tenant_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("No LDAP config for tenant {tenant_id}"))?;

        let ldap_config = LdapConfig {
            server_url:        config_row.server_url.clone(),
            use_tls:           config_row.use_tls,
            base_dn:           config_row.base_dn.clone(),
            search_filter:     config_row.search_filter.clone(),
            bind_dn:           config_row.bind_dn.clone(),
            bind_password:     config_row.bind_password.clone(),
            username_attr:     config_row.username_attr.clone(),
            first_name_attr:   config_row.first_name_attr.clone(),
            last_name_attr:    config_row.last_name_attr.clone(),
            email_attr:        config_row.email_attr.clone(),
            default_authority: config_row.default_authority.clone(),
            group_search_base: config_row.group_search_base.clone(),
            group_filter:      config_row.group_filter.clone(),
        };

        let provider = LdapAuthProvider::new(ldap_config);
        let members = provider.search_group_members().await
            .map_err(|e| e.to_string())?;

        let mut synced = 0usize;
        for member in &members {
            if member.email.is_empty() {
                continue;
            }
            match self.user_dao.find_by_email(&member.email).await {
                Err(e) => error!(email=%member.email, "LDAP sync: lookup failed: {e}"),
                Ok(Some(_)) => {} // already exists — no update needed for basic sync
                Ok(None) => {
                    // Provision new user
                    let now = chrono::Utc::now().timestamp_millis();
                    let user = User {
                        id:              Uuid::new_v4(),
                        created_time:    now,
                        tenant_id,
                        customer_id:     None,
                        email:           member.email.clone(),
                        authority:       Authority::CustomerUser,
                        first_name:      member.first_name.clone(),
                        last_name:       member.last_name.clone(),
                        phone:           None,
                        additional_info: Some(serde_json::json!({"ldapDn": member.dn})),
                        version:         1,
                    };
                    if let Err(e) = self.user_dao.save(&user).await {
                        error!(email=%member.email, "LDAP sync: provision failed: {e}");
                    } else {
                        synced += 1;
                    }
                }
            }
        }
        info!(tenant=%tenant_id, total=members.len(), provisioned=synced, "LDAP sync complete");
        Ok(())
    }
}
