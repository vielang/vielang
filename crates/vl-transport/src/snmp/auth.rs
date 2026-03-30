/// SNMP authentication — community string validation.
///
/// ThingsBoard Java supports a single configurable community string.
/// This module extends that with a small allowlist so multiple devices
/// can use different community strings (per-device credentials).
///
/// SNMPv3 USM (User Security Model) is not yet implemented.
/// For v3 support, the USM engine would go here.

/// Authenticate an incoming SNMP community string against the allowed set.
///
/// `allowed` is the primary community from `SnmpConfig.community`.
/// Additional communities can be added via the `extra_communities` slice.
pub fn authenticate_community(
    received:          &str,
    primary_community: &str,
    extra_communities: &[String],
) -> AuthResult {
    if received == primary_community {
        return AuthResult::Ok { community: received.to_string() };
    }
    for c in extra_communities {
        if received == c {
            return AuthResult::Ok { community: received.to_string() };
        }
    }
    AuthResult::CommunityMismatch
}

#[derive(Debug, PartialEq)]
pub enum AuthResult {
    Ok { community: String },
    CommunityMismatch,
}

impl AuthResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, AuthResult::Ok { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_community_accepted() {
        let r = authenticate_community("public", "public", &[]);
        assert!(r.is_ok());
    }

    #[test]
    fn test_extra_community_accepted() {
        let extras = vec!["private".to_string(), "monitor".to_string()];
        let r = authenticate_community("monitor", "public", &extras);
        assert!(r.is_ok());
    }

    #[test]
    fn test_wrong_community_rejected() {
        let r = authenticate_community("hacked", "public", &[]);
        assert_eq!(r, AuthResult::CommunityMismatch);
    }
}
