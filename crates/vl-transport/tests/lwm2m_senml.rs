use vl_transport::lwm2m::senml::{parse_senml, senml_to_kv};
use serde_json::json;

// ── parse_senml tests ─────────────────────────────────────────────────────────

#[test]
fn parse_empty_array() {
    let result = parse_senml(b"[]");
    assert!(result.is_some());
    assert!(result.unwrap().is_empty());
}

#[test]
fn parse_invalid_json_returns_none() {
    assert!(parse_senml(b"not json").is_none());
    assert!(parse_senml(b"").is_none());
    assert!(parse_senml(b"{\"n\":\"x\"}").is_none()); // object, not array
}

#[test]
fn parse_single_numeric_value() {
    let data = br#"[{"n":"temperature","v":25.5}]"#;
    let records = parse_senml(data).expect("should parse");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].n.as_deref(), Some("temperature"));
    assert_eq!(records[0].v, Some(25.5));
    assert!(records[0].vs.is_none());
    assert!(records[0].vb.is_none());
}

#[test]
fn parse_multiple_records_different_types() {
    let data = br#"[
        {"n":"temp","v":22.1},
        {"n":"label","vs":"kitchen"},
        {"n":"active","vb":true},
        {"n":"raw","vd":"AQID"},
        {"n":"voltage","v":3.3,"t":1700000000.0}
    ]"#;
    let records = parse_senml(data).expect("should parse");
    assert_eq!(records.len(), 5);

    assert_eq!(records[0].v, Some(22.1));
    assert_eq!(records[1].vs.as_deref(), Some("kitchen"));
    assert_eq!(records[2].vb, Some(true));
    assert_eq!(records[3].vd.as_deref(), Some("AQID"));
    assert_eq!(records[4].t, Some(1700000000.0));
}

#[test]
fn parse_with_base_name() {
    let data = br#"[
        {"bn":"/3303/0/","n":"5700","v":25.5},
        {"n":"5701","vs":"Cel"}
    ]"#;
    let records = parse_senml(data).expect("should parse");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].bn.as_deref(), Some("/3303/0/"));
    assert_eq!(records[0].n.as_deref(), Some("5700"));
    assert_eq!(records[1].bn, None);
    assert_eq!(records[1].n.as_deref(), Some("5701"));
}

// ── senml_to_kv tests ─────────────────────────────────────────────────────────

#[test]
fn senml_to_kv_empty_records() {
    let kv = senml_to_kv(&[]);
    assert!(kv.as_object().unwrap().is_empty());
}

#[test]
fn senml_to_kv_single_numeric() {
    let records = parse_senml(br#"[{"n":"temperature","v":25.5}]"#).unwrap();
    let kv = senml_to_kv(&records);
    assert_eq!(kv["temperature"], json!(25.5));
}

#[test]
fn senml_to_kv_multiple_types() {
    let data = br#"[
        {"n":"temperature","v":22.1},
        {"n":"label","vs":"kitchen"},
        {"n":"active","vb":true}
    ]"#;
    let records = parse_senml(data).unwrap();
    let kv = senml_to_kv(&records);
    let obj = kv.as_object().unwrap();
    assert_eq!(obj.len(), 3);
    assert_eq!(kv["temperature"], json!(22.1));
    assert_eq!(kv["label"], json!("kitchen"));
    assert_eq!(kv["active"], json!(true));
}

#[test]
fn senml_to_kv_base_name_accumulates() {
    let data = br#"[
        {"bn":"/3303/0/","n":"5700","v":25.5},
        {"n":"5701","vs":"Cel"}
    ]"#;
    let records = parse_senml(data).unwrap();
    let kv = senml_to_kv(&records);
    // base name "/3303/0/" should persist to second record
    assert!(kv.get("/3303/0/5700").is_some(), "key /3303/0/5700 not found");
    assert!(kv.get("/3303/0/5701").is_some(), "key /3303/0/5701 not found");
    assert_eq!(kv["/3303/0/5700"], json!(25.5));
    assert_eq!(kv["/3303/0/5701"], json!("Cel"));
}

#[test]
fn senml_to_kv_skips_records_without_name() {
    let data = br#"[
        {"v":1.0},
        {"n":"named","v":2.0}
    ]"#;
    let records = parse_senml(data).unwrap();
    let kv = senml_to_kv(&records);
    let obj = kv.as_object().unwrap();
    // Only the named record should appear
    assert_eq!(obj.len(), 1);
    assert_eq!(kv["named"], json!(2.0));
}
