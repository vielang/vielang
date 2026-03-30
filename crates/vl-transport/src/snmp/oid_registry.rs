/// Static registry of common SNMP OID → friendly telemetry key name mappings.
///
/// Covers MIB-II (RFC 1213) and a selection of common enterprise OIDs.
/// When an OID is not in the registry, the raw dotted-decimal string is used
/// as the telemetry key (preserving all data, just less readable).
use std::collections::HashMap;

fn registry() -> &'static HashMap<&'static str, &'static str> {
    REGISTRY.get_or_init(init_registry)
}

/// Resolve an OID string to a human-readable telemetry key.
/// Returns empty string if not found in the registry.
pub fn resolve_oid(oid: &str) -> &'static str {
    registry().get(oid).copied().unwrap_or("")
}

/// Return a human-readable name or the OID itself.
pub fn oid_to_key(oid: &str) -> String {
    let name = resolve_oid(oid);
    if name.is_empty() {
        oid.to_string()
    } else {
        name.to_string()
    }
}

// ── Static registry ───────────────────────────────────────────────────────────

use std::sync::OnceLock;

static REGISTRY: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn init_registry() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // ── MIB-II system (1.3.6.1.2.1.1) ────────────────────────────────────────
    m.insert("1.3.6.1.2.1.1.1.0", "sysDescr");
    m.insert("1.3.6.1.2.1.1.2.0", "sysObjectID");
    m.insert("1.3.6.1.2.1.1.3.0", "sysUpTime");
    m.insert("1.3.6.1.2.1.1.4.0", "sysContact");
    m.insert("1.3.6.1.2.1.1.5.0", "sysName");
    m.insert("1.3.6.1.2.1.1.6.0", "sysLocation");
    m.insert("1.3.6.1.2.1.1.7.0", "sysServices");

    // ── MIB-II interfaces (1.3.6.1.2.1.2) ────────────────────────────────────
    m.insert("1.3.6.1.2.1.2.1.0",   "ifNumber");
    m.insert("1.3.6.1.2.1.2.2.1.1", "ifIndex");
    m.insert("1.3.6.1.2.1.2.2.1.2", "ifDescr");
    m.insert("1.3.6.1.2.1.2.2.1.3", "ifType");
    m.insert("1.3.6.1.2.1.2.2.1.4", "ifMtu");
    m.insert("1.3.6.1.2.1.2.2.1.5", "ifSpeed");
    m.insert("1.3.6.1.2.1.2.2.1.6", "ifPhysAddress");
    m.insert("1.3.6.1.2.1.2.2.1.7", "ifAdminStatus");
    m.insert("1.3.6.1.2.1.2.2.1.8", "ifOperStatus");
    m.insert("1.3.6.1.2.1.2.2.1.10", "ifInOctets");
    m.insert("1.3.6.1.2.1.2.2.1.11", "ifInUcastPkts");
    m.insert("1.3.6.1.2.1.2.2.1.13", "ifInDiscards");
    m.insert("1.3.6.1.2.1.2.2.1.14", "ifInErrors");
    m.insert("1.3.6.1.2.1.2.2.1.16", "ifOutOctets");
    m.insert("1.3.6.1.2.1.2.2.1.17", "ifOutUcastPkts");
    m.insert("1.3.6.1.2.1.2.2.1.19", "ifOutDiscards");
    m.insert("1.3.6.1.2.1.2.2.1.20", "ifOutErrors");

    // ── MIB-II IP (1.3.6.1.2.1.4) ────────────────────────────────────────────
    m.insert("1.3.6.1.2.1.4.1.0",  "ipForwarding");
    m.insert("1.3.6.1.2.1.4.2.0",  "ipDefaultTTL");
    m.insert("1.3.6.1.2.1.4.3.0",  "ipInReceives");
    m.insert("1.3.6.1.2.1.4.5.0",  "ipInHdrErrors");
    m.insert("1.3.6.1.2.1.4.6.0",  "ipInAddrErrors");
    m.insert("1.3.6.1.2.1.4.10.0", "ipInDelivers");
    m.insert("1.3.6.1.2.1.4.11.0", "ipOutRequests");
    m.insert("1.3.6.1.2.1.4.14.0", "ipOutDiscards");
    m.insert("1.3.6.1.2.1.4.17.0", "ipReasmTimeout");
    m.insert("1.3.6.1.2.1.4.23.0", "ipFragOKs");
    m.insert("1.3.6.1.2.1.4.24.0", "ipFragFails");

    // ── MIB-II TCP (1.3.6.1.2.1.6) ───────────────────────────────────────────
    m.insert("1.3.6.1.2.1.6.1.0",  "tcpRtoAlgorithm");
    m.insert("1.3.6.1.2.1.6.5.0",  "tcpActiveOpens");
    m.insert("1.3.6.1.2.1.6.6.0",  "tcpPassiveOpens");
    m.insert("1.3.6.1.2.1.6.7.0",  "tcpAttemptFails");
    m.insert("1.3.6.1.2.1.6.8.0",  "tcpEstabResets");
    m.insert("1.3.6.1.2.1.6.9.0",  "tcpCurrEstab");
    m.insert("1.3.6.1.2.1.6.10.0", "tcpInSegs");
    m.insert("1.3.6.1.2.1.6.11.0", "tcpOutSegs");
    m.insert("1.3.6.1.2.1.6.12.0", "tcpRetransSegs");
    m.insert("1.3.6.1.2.1.6.14.0", "tcpInErrs");
    m.insert("1.3.6.1.2.1.6.15.0", "tcpOutRsts");

    // ── MIB-II UDP (1.3.6.1.2.1.7) ───────────────────────────────────────────
    m.insert("1.3.6.1.2.1.7.1.0", "udpInDatagrams");
    m.insert("1.3.6.1.2.1.7.2.0", "udpNoPorts");
    m.insert("1.3.6.1.2.1.7.3.0", "udpInErrors");
    m.insert("1.3.6.1.2.1.7.4.0", "udpOutDatagrams");

    // ── SNMP MIB (1.3.6.1.2.1.11) ────────────────────────────────────────────
    m.insert("1.3.6.1.2.1.11.1.0",  "snmpInPkts");
    m.insert("1.3.6.1.2.1.11.2.0",  "snmpOutPkts");
    m.insert("1.3.6.1.2.1.11.30.0", "snmpEnableAuthenTraps");

    // ── HR MIB — Host Resources (1.3.6.1.2.1.25) ─────────────────────────────
    m.insert("1.3.6.1.2.1.25.1.1.0",  "hrSystemUptime");
    m.insert("1.3.6.1.2.1.25.1.6.0",  "hrSystemNumUsers");
    m.insert("1.3.6.1.2.1.25.1.7.0",  "hrSystemProcesses");
    m.insert("1.3.6.1.2.1.25.2.2.0",  "hrMemorySize");
    m.insert("1.3.6.1.2.1.25.3.3.1.2","hrProcessorLoad");

    // ── UCD-SNMP MIB — net-snmp extensions (1.3.6.1.4.1.2021) ───────────────
    m.insert("1.3.6.1.4.1.2021.10.1.3.1", "laLoad1");
    m.insert("1.3.6.1.4.1.2021.10.1.3.2", "laLoad5");
    m.insert("1.3.6.1.4.1.2021.10.1.3.3", "laLoad15");
    m.insert("1.3.6.1.4.1.2021.11.9.0",   "ssCpuUser");
    m.insert("1.3.6.1.4.1.2021.11.10.0",  "ssCpuSystem");
    m.insert("1.3.6.1.4.1.2021.11.11.0",  "ssCpuIdle");
    m.insert("1.3.6.1.4.1.2021.4.5.0",    "memTotalReal");
    m.insert("1.3.6.1.4.1.2021.4.6.0",    "memAvailReal");
    m.insert("1.3.6.1.4.1.2021.4.11.0",   "memTotalFree");
    m.insert("1.3.6.1.4.1.2021.4.13.0",   "memShared");
    m.insert("1.3.6.1.4.1.2021.4.14.0",   "memBuffer");
    m.insert("1.3.6.1.4.1.2021.4.15.0",   "memCached");
    m.insert("1.3.6.1.4.1.2021.9.1.6.1",  "dskUsed");
    m.insert("1.3.6.1.4.1.2021.9.1.7.1",  "dskAvail");
    m.insert("1.3.6.1.4.1.2021.9.1.9.1",  "dskPercent");

    // ── SNMPv2-MIB trap OIDs ─────────────────────────────────────────────────
    m.insert("1.3.6.1.6.3.1.1.4.1.0", "snmpTrapOID");
    m.insert("1.3.6.1.6.3.1.1.5.1",   "coldStart");
    m.insert("1.3.6.1.6.3.1.1.5.2",   "warmStart");
    m.insert("1.3.6.1.6.3.1.1.5.3",   "linkDown");
    m.insert("1.3.6.1.6.3.1.1.5.4",   "linkUp");
    m.insert("1.3.6.1.6.3.1.1.5.5",   "authenticationFailure");

    m
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_oid() {
        assert_eq!(oid_to_key("1.3.6.1.2.1.1.5.0"), "sysName");
        assert_eq!(oid_to_key("1.3.6.1.2.1.1.3.0"), "sysUpTime");
    }

    #[test]
    fn test_unknown_oid_returns_raw() {
        assert_eq!(oid_to_key("1.3.6.1.9.9.9.9.9"), "1.3.6.1.9.9.9.9.9");
    }

    #[test]
    fn test_ucd_snmp_oids() {
        assert_eq!(oid_to_key("1.3.6.1.4.1.2021.11.9.0"), "ssCpuUser");
        assert_eq!(oid_to_key("1.3.6.1.4.1.2021.4.6.0"), "memAvailReal");
    }

    #[test]
    fn test_trap_oids() {
        assert_eq!(oid_to_key("1.3.6.1.6.3.1.1.5.3"), "linkDown");
        assert_eq!(oid_to_key("1.3.6.1.6.3.1.1.5.4"), "linkUp");
    }
}
