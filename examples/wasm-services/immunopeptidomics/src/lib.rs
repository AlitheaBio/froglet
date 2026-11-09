//! Narrow synthetic immunopeptidomics example using provider-local SQLite.
//!
//! A recorded genotype selects a sample stratum; it does not establish peptide
//! restriction. Absence from a measured assay is not evidence of biological
//! absence. Only counts and requester-supplied query metadata are returned.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const SCHEMA: &str = "froglet.immunopeptidomics.counts.v1";
const COLUMNS: [&str; 6] = [
    "sample_id",
    "assay_status",
    "has_requested_allele",
    "typing_status",
    "typing_status_rows",
    "has_exact_peptide",
];

// One row per local sample, with existence checks rather than observation joins.
// Duplicate detections or typing observations cannot multiply the denominator.
// Every comparison is explicitly binary: this example does no normalization.
const SQL: &str = "SELECT s.sample_id AS sample_id, s.assay_status AS assay_status, \
EXISTS(SELECT 1 FROM sample_hla_typing h WHERE h.sample_id = s.sample_id COLLATE BINARY AND h.allele_name = ?2 COLLATE BINARY) AS has_requested_allele, \
(SELECT CASE WHEN COUNT(*) = 0 THEN 'missing' ELSE MIN(t.status) END FROM sample_hla_typing_status t WHERE t.sample_id = s.sample_id COLLATE BINARY AND t.locus = ?5 COLLATE BINARY) AS typing_status, \
(SELECT COUNT(*) FROM sample_hla_typing_status t WHERE t.sample_id = s.sample_id COLLATE BINARY AND t.locus = ?5 COLLATE BINARY) AS typing_status_rows, \
EXISTS(SELECT 1 FROM sample_peptide_detections p WHERE p.sample_id = s.sample_id COLLATE BINARY AND p.sequence = ?1 COLLATE BINARY) AS has_exact_peptide \
FROM samples s WHERE s.sample_class = ?3 COLLATE BINARY AND s.organ = ?4 COLLATE BINARY ORDER BY s.sample_id COLLATE BINARY";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Query {
    candidate_peptide: String,
    hla_allele: String,
    sample_class: String,
    organ: String,
}

impl Query {
    fn locus(&self) -> Option<&str> {
        let (locus, allele) = self.hla_allele.split_once('*')?;
        if !matches!(locus, "HLA-A" | "HLA-B" | "HLA-C") {
            return None;
        }
        let fields = allele.split(':').collect::<Vec<_>>();
        if fields.len() != 2
            || fields.iter().any(|field| {
                !(2..=3).contains(&field.len()) || !field.bytes().all(|b| b.is_ascii_digit())
            })
        {
            return None;
        }
        Some(locus)
    }

    fn valid(&self) -> bool {
        !self.candidate_peptide.is_empty()
            && self.candidate_peptide.len() <= 128
            && self
                .candidate_peptide
                .bytes()
                .all(|b| b"ACDEFGHIKLMNPQRSTVWY".contains(&b))
            && self.locus().is_some()
            && [&self.sample_class, &self.organ]
                .iter()
                .all(|value| !value.trim().is_empty() && value.len() <= 128)
    }
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct Counts {
    cohort_total: u64,
    hla_unknown: u64,
    hla_nonmatching: u64,
    eligible: u64,
    measured: u64,
    detected: u64,
    not_detected: u64,
    not_measured: u64,
    failed: u64,
    missing: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostRows {
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
}

fn flag(value: &Value) -> Result<bool, &'static str> {
    match value.as_u64() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err("invalid_local_data"),
    }
}

fn aggregate(response: &[u8]) -> Result<Counts, &'static str> {
    let data: HostRows = serde_json::from_slice(response).map_err(|_| "invalid_local_data")?;
    if data.columns != COLUMNS {
        return Err("invalid_local_data");
    }
    let mut counts = Counts::default();
    let mut ids = BTreeSet::new();
    for row in &data.rows {
        if row.len() != COLUMNS.len() {
            return Err("invalid_local_data");
        }
        let id = row[0].as_str().ok_or("invalid_local_data")?;
        if id.is_empty() || !ids.insert(id) {
            return Err("invalid_local_data");
        }
        let assay = row[1].as_str().ok_or("invalid_local_data")?;
        if !matches!(assay, "measured" | "not_measured" | "failed") {
            return Err("invalid_local_data");
        }
        let has_allele = flag(&row[2])?;
        let typing_status = row[3].as_str().ok_or("invalid_local_data")?;
        if !matches!(typing_status, "complete" | "incomplete" | "missing") {
            return Err("invalid_local_data");
        }
        let typing_rows = row[4].as_u64().ok_or("invalid_local_data")?;
        if typing_rows > 1 || (typing_rows == 0 && typing_status != "missing") {
            return Err("invalid_local_data");
        }
        let detected = flag(&row[5])?;
        counts.cohort_total += 1;
        if !has_allele {
            if typing_status == "complete" {
                counts.hla_nonmatching += 1;
            } else {
                counts.hla_unknown += 1;
            }
            continue;
        }
        counts.eligible += 1;
        match assay {
            "measured" => {
                counts.measured += 1;
                if detected {
                    counts.detected += 1;
                } else {
                    counts.not_detected += 1;
                }
            }
            "not_measured" => counts.not_measured += 1,
            "failed" => counts.failed += 1,
            _ => unreachable!(),
        }
    }
    counts.missing = counts.not_measured + counts.failed;
    Ok(counts)
}

fn respond_with_host<F>(request: &[u8], host: F) -> String
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, &'static str>,
{
    // Derived structs accept positional arrays; require an object first.
    if !matches!(
        serde_json::from_slice::<Value>(request),
        Ok(Value::Object(_))
    ) {
        return json!({"error": "invalid_request"}).to_string();
    }
    let query = match serde_json::from_slice::<Query>(request) {
        Ok(query) if query.valid() => query,
        _ => return json!({"error": "invalid_request"}).to_string(),
    };
    let envelope = json!({
        "op": "db.query",
        "request": {
            "handle": "cohort",
            "sql": SQL,
            "params": [query.candidate_peptide, query.hla_allele, query.sample_class, query.organ, query.locus().expect("validated locus")],
        },
    });
    let request_bytes = serde_json::to_vec(&envelope).expect("host request is JSON data");
    let result = host(&request_bytes).and_then(|bytes| aggregate(&bytes));
    match result {
        Ok(counts) => json!({
            "schema_version": SCHEMA,
            "query": query,
            "counts": counts,
            "interpretation": {
                "hla_selection": "recorded_genotype_not_peptide_restriction",
                "not_detected": "not_detected_in_measured_assay_not_biological_absence",
                "denominator": "measured_eligible_samples",
            },
        })
        .to_string(),
        Err(code) => json!({"error": code}).to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "froglet_host")]
extern "C" {
    fn call_json(pointer: i32, len: i32) -> i64;
}

#[cfg(target_arch = "wasm32")]
fn invoke_host(request: &[u8]) -> Result<Vec<u8>, &'static str> {
    let len = i32::try_from(request.len()).map_err(|_| "host_query_failed")?;
    let packed = unsafe { call_json(request.as_ptr() as i32, len) } as u64;
    let pointer = (packed >> 32) as u32 as *const u8;
    let len = (packed & 0xffff_ffff) as u32 as usize;
    if pointer.is_null() || len == 0 {
        return Err("host_query_failed");
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, len) }.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
fn invoke_host(_request: &[u8]) -> Result<Vec<u8>, &'static str> {
    Err("host_query_failed")
}

/// Evaluate the request; native execution deliberately cannot access a database.
pub fn respond(request: &[u8]) -> String {
    respond_with_host(request, invoke_host)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    let mut buffer = Vec::<u8>::with_capacity(len as u32 as usize);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer as i32
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn run(pointer: i32, len: i32) -> i64 {
    let request =
        unsafe { std::slice::from_raw_parts(pointer as u32 as *const u8, len as u32 as usize) };
    let bytes = respond(request).into_bytes();
    let packed = ((bytes.as_ptr() as u32 as i64) << 32) | bytes.len() as i64;
    std::mem::forget(bytes);
    packed
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUEST: &[u8] = br#"{"candidate_peptide":"ACDEFGHIK","hla_allele":"HLA-A*02:01","sample_class":"healthy_tissue","organ":"synthetic_organ_a"}"#;

    fn response(rows: Vec<Value>) -> Vec<u8> {
        serde_json::to_vec(&json!({"columns": COLUMNS, "rows": rows})).unwrap()
    }

    #[test]
    fn denominator_and_missingness_have_distinct_meanings() {
        let rows = vec![
            json!(["d", "measured", 1, "complete", 1, 1]),
            json!(["n", "measured", 1, "complete", 1, 0]),
            json!(["u", "not_measured", 1, "complete", 1, 1]),
            json!(["f", "failed", 1, "complete", 1, 1]),
            json!(["x", "measured", 0, "complete", 1, 1]),
            json!(["m", "measured", 0, "missing", 0, 1]),
            json!(["i", "measured", 0, "incomplete", 1, 1]),
        ];
        assert_eq!(
            aggregate(&response(rows)).unwrap(),
            Counts {
                cohort_total: 7,
                hla_unknown: 2,
                hla_nonmatching: 1,
                eligible: 4,
                measured: 2,
                detected: 1,
                not_detected: 1,
                not_measured: 1,
                failed: 1,
                missing: 2,
            }
        );
    }

    #[test]
    fn recorded_allele_is_sufficient_despite_incomplete_or_missing_typing() {
        let counts = aggregate(&response(vec![
            json!(["a", "measured", 1, "incomplete", 1, 1]),
            json!(["b", "measured", 1, "missing", 0, 0]),
        ]))
        .unwrap();
        assert_eq!(counts.eligible, 2);
        assert_eq!(counts.measured, 2);
        assert_eq!(counts.detected, 1);
        assert_eq!(counts.hla_unknown, 0);
    }

    #[test]
    fn empty_cohort_is_valid_and_zero_denominator_is_explicit() {
        assert_eq!(aggregate(&response(vec![])).unwrap(), Counts::default());
        let counts =
            aggregate(&response(vec![json!(["a", "failed", 1, "complete", 1, 0])])).unwrap();
        assert_eq!(counts.measured, 0);
        assert_eq!(counts.missing, 1);
    }

    #[test]
    fn malformed_local_facts_fail_closed_without_echoing_records() {
        let valid = json!(["private-sample", "measured", 1, "complete", 1, 1]);
        for rows in [
            vec![valid.clone(), valid.clone()],
            vec![json!(["", "measured", 1, "complete", 1, 1])],
            vec![json!(["private-sample", "unknown", 1, "complete", 1, 1])],
            vec![json!([
                "private-sample",
                "measured",
                true,
                "complete",
                1,
                1
            ])],
            vec![json!(["private-sample", "measured", 1, "unknown", 1, 1])],
            vec![json!(["private-sample", "measured", 1, "complete", 2, 1])],
            vec![json!(["private-sample", "measured", 1, "complete", 0, 1])],
            vec![json!(["private-sample", "measured", 1, "complete", 1, 2])],
        ] {
            let text = respond_with_host(REQUEST, |_| Ok(response(rows)));
            assert_eq!(text, r#"{"error":"invalid_local_data"}"#);
            assert!(!text.contains("private-sample"));
        }
        for bytes in [
            br#"{"columns":[],"rows":[]}"#.to_vec(),
            br#"{"columns":[],"rows":[],"truncated":true}"#.to_vec(),
            b"not JSON".to_vec(),
        ] {
            assert!(aggregate(&bytes).is_err());
        }
    }

    #[test]
    fn request_is_bound_to_fixed_sql_and_output_contains_only_counts_and_metadata() {
        let output = respond_with_host(REQUEST, |request| {
            let value: Value = serde_json::from_slice(request).unwrap();
            assert_eq!(value["op"], "db.query");
            assert_eq!(value["request"]["handle"], "cohort");
            assert_eq!(value["request"]["sql"], SQL);
            assert_eq!(
                value["request"]["params"],
                json!([
                    "ACDEFGHIK",
                    "HLA-A*02:01",
                    "healthy_tissue",
                    "synthetic_organ_a",
                    "HLA-A"
                ])
            );
            Ok(response(vec![json!([
                "private-sample",
                "measured",
                1,
                "complete",
                1,
                1
            ])]))
        });
        let value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["counts"]["detected"], 1);
        assert_eq!(
            value["interpretation"]["denominator"],
            "measured_eligible_samples"
        );
        assert!(!output.contains("private-sample"));
        assert_eq!(value.as_object().unwrap().len(), 4);
    }

    #[test]
    fn malformed_requests_never_call_host() {
        for input in [
            b"null".as_slice(), b"[]".as_slice(), b"{}".as_slice(),
            br#"{"candidate_peptide":"ACDEFGHIK","hla_allele":"HLA-A*02:01","sample_class":"healthy_tissue","organ":"a","rows":[]}"#,
            br#"{"candidate_peptide":"ACDEFGHIK","candidate_peptide":"AAAA","hla_allele":"HLA-A*02:01","sample_class":"healthy_tissue","organ":"a"}"#,
            br#"{"candidate_peptide":"acdefghik","hla_allele":"HLA-A*02:01","sample_class":"healthy_tissue","organ":"a"}"#,
            br#"{"candidate_peptide":"ACDEFGHIK","hla_allele":"A0201","sample_class":"healthy_tissue","organ":"a"}"#,
            br#"{"candidate_peptide":"ACDEFGHIK","hla_allele":"HLA-A*02","sample_class":"healthy_tissue","organ":"a"}"#,
        ] {
            assert_eq!(respond_with_host(input, |_| panic!("must not access host")), r#"{"error":"invalid_request"}"#);
        }
    }

    #[test]
    fn host_failures_do_not_become_zero_counts() {
        assert_eq!(
            respond_with_host(REQUEST, |_| Err("host_query_failed")),
            r#"{"error":"host_query_failed"}"#
        );
        assert_eq!(respond(REQUEST), r#"{"error":"host_query_failed"}"#);
    }
}
