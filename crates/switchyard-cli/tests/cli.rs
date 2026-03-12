use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_switchyard-cli")
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn fixture(path: &str) -> PathBuf {
    root().join(path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(cli_bin()).args(args).output().expect("run switchyard-cli")
}

fn write_temp_json(name: &str, contents: &str) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after epoch").as_nanos();
    let path = env::temp_dir().join(format!("switchyard-cli-{name}-{stamp}.json"));
    fs::write(&path, contents).expect("write temp json");
    path
}

fn write_temp_value(name: &str, value: &Value) -> PathBuf {
    write_temp_json(name, &serde_json::to_string_pretty(value).expect("encode temp json"))
}

fn write_asset_bundle_manifest(catalog: &Path, snapshots: &[(&str, &Path)]) -> PathBuf {
    let snapshots: Vec<Value> = snapshots
        .iter()
        .map(|(id, path)| {
            serde_json::json!({
                "id": id,
                "path": path.to_str().expect("utf-8 path"),
            })
        })
        .collect();
    write_temp_value(
        "asset-bundle-manifest",
        &serde_json::json!({
            "version": 1,
            "catalog": catalog.to_str().expect("utf-8 path"),
            "snapshots": snapshots,
        }),
    )
}

#[test]
fn catalog_check_accepts_valid_catalog() {
    let path = fixture("fixtures/contracts/program.valid.json");
    let output = run_cli(&["catalog-check", path.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "catalog_check");
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["program_count"], 3);
    assert_eq!(summary["program_ids"], serde_json::json!([1, 2, 3]));
}

#[test]
fn catalog_check_reports_compile_error_for_missing_reference() {
    let path = write_temp_json(
        "missing-program",
        r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "spawn", "program": 99 },
        { "op": "succeed" }
      ]
    }
  ]
}"#,
    );

    let output = run_cli(&["catalog-check", path.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("catalog compile failed"));
    assert!(stderr.contains("program 1 references missing program 99"));

    let _ = fs::remove_file(path);
}

#[test]
fn catalog_summary_reports_programs_and_op_histogram() {
    let path = fixture("fixtures/contracts/program.valid.json");
    let output = run_cli(&["catalog-summary", path.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "catalog_summary");
    assert_eq!(summary["program_count"], 3);
    assert_eq!(summary["program_ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(summary["referenced_program_ids"], serde_json::json!([2, 3]));
    assert_eq!(summary["op_histogram"]["repeat_count"], 1);
    assert_eq!(summary["op_histogram"]["call"], 1);
    assert_eq!(summary["op_histogram"]["change_mind"], 1);
    assert_eq!(summary["signal_ids"], serde_json::json!([7]));
    assert_eq!(summary["predicate_ids"], serde_json::json!([5]));
    assert_eq!(summary["mind_ids"], serde_json::json!([2]));
    assert_eq!(summary["host_call_ids"], serde_json::json!([1]));
}

#[test]
fn snapshot_summary_reports_clock_tasks_and_wait_kinds() {
    let path = fixture("fixtures/contracts/snapshot.valid.json");
    let output = run_cli(&["snapshot-summary", path.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "snapshot_summary");
    assert_eq!(summary["clock"], 3);
    assert_eq!(summary["next_task_id"], 3);
    assert_eq!(summary["task_count"], 3);
    assert_eq!(summary["program_ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(summary["mind_ids"], serde_json::json!([1]));
    assert_eq!(summary["pending_signals"], serde_json::json!([]));
    assert_eq!(summary["wait_kind_histogram"]["race_or_ticks"], 1);
    assert_eq!(summary["wait_kind_histogram"]["signal"], 2);
    assert_eq!(summary["outcome_histogram"]["running"], 3);
}

#[test]
fn snapshot_summary_accepts_showcase_exported_runtime_snapshot() {
    let path = write_temp_json(
        "showcase-exported-snapshot",
        r#"{
  "clock": 1,
  "next_task_id": 1,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 1
      }
    }
  ],
  "pending_signals": []
}"#,
    );

    let output = run_cli(&["snapshot-summary", path.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "snapshot_summary");
    assert_eq!(summary["clock"], 1);
    assert_eq!(summary["task_count"], 1);
    assert_eq!(summary["program_ids"], serde_json::json!([1]));
    assert_eq!(summary["wait_kind_histogram"]["signal"], 1);

    let _ = fs::remove_file(path);
}

#[test]
fn snapshot_check_accepts_valid_catalog_and_snapshot_pair() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot = fixture("fixtures/contracts/snapshot.valid.json");

    let output = run_cli(&[
        "snapshot-check",
        catalog.to_str().expect("utf-8 path"),
        snapshot.to_str().expect("utf-8 path"),
    ]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "snapshot_check");
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["task_count"], 3);
    assert_eq!(summary["program_ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(summary["task_ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(summary["root_task_ids"], serde_json::json!([1]));
    assert_eq!(summary["pending_signals"], serde_json::json!([]));
}

#[test]
fn snapshot_check_rejects_snapshot_with_unknown_program_id() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let mut snapshot: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fixtures/contracts/snapshot.valid.json"))
            .expect("read snapshot fixture"),
    )
    .expect("parse snapshot fixture");
    snapshot["tasks"][0]["program_id"] = serde_json::json!(99);
    let snapshot_path = write_temp_value("snapshot-check-unknown-program", &snapshot);

    let output = run_cli(&[
        "snapshot-check",
        catalog.to_str().expect("utf-8 path"),
        snapshot_path.to_str().expect("utf-8 path"),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("snapshot validation failed"));
    assert!(stderr.contains("task 1 references unknown program 99"));

    let _ = fs::remove_file(snapshot_path);
}

#[test]
fn snapshot_check_rejects_snapshot_with_missing_parent_task() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let mut snapshot: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fixtures/contracts/snapshot.valid.json"))
            .expect("read snapshot fixture"),
    )
    .expect("parse snapshot fixture");
    snapshot["tasks"][1]["parent"] = serde_json::json!(99);
    let snapshot_path = write_temp_value("snapshot-check-missing-parent", &snapshot);

    let output = run_cli(&[
        "snapshot-check",
        catalog.to_str().expect("utf-8 path"),
        snapshot_path.to_str().expect("utf-8 path"),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("snapshot validation failed"));
    assert!(stderr.contains("task 2 references missing parent task 99"));

    let _ = fs::remove_file(snapshot_path);
}

#[test]
fn snapshot_check_rejects_snapshot_with_missing_wait_reference() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let mut snapshot: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fixtures/contracts/snapshot.valid.json"))
            .expect("read snapshot fixture"),
    )
    .expect("parse snapshot fixture");
    snapshot["tasks"][0]["wait"]["right"] = serde_json::json!(99);
    let snapshot_path = write_temp_value("snapshot-check-missing-wait-ref", &snapshot);

    let output = run_cli(&[
        "snapshot-check",
        catalog.to_str().expect("utf-8 path"),
        snapshot_path.to_str().expect("utf-8 path"),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("snapshot validation failed"));
    assert!(stderr.contains("task 1 wait references missing task 99"));

    let _ = fs::remove_file(snapshot_path);
}

#[test]
fn snapshot_check_accepts_showcase_exported_catalog_and_runtime_snapshot() {
    let catalog = write_temp_json(
        "showcase-exported-catalog",
        r#"{
  "programs": [
    {
      "id": 1,
      "ops": [
        { "op": "wait_signal", "signal": 1 },
        { "op": "succeed" }
      ]
    }
  ]
}"#,
    );
    let snapshot = write_temp_json(
        "showcase-exported-runtime-snapshot",
        r#"{
  "clock": 1,
  "next_task_id": 2,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 1
      }
    }
  ],
  "pending_signals": []
}"#,
    );

    let output = run_cli(&[
        "snapshot-check",
        catalog.to_str().expect("utf-8 path"),
        snapshot.to_str().expect("utf-8 path"),
    ]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "snapshot_check");
    assert_eq!(summary["task_count"], 1);
    assert_eq!(summary["program_ids"], serde_json::json!([1]));
    assert_eq!(summary["root_task_ids"], serde_json::json!([1]));

    let _ = fs::remove_file(catalog);
    let _ = fs::remove_file(snapshot);
}

#[test]
fn catalog_normalize_rewrites_to_canonical_json() {
    let input = write_temp_json(
        "catalog-normalize",
        r#"{"programs":[{"id":2,"ops":[{"op":"succeed"}]},{"id":1,"ops":[{"op":"action","action":9},{"op":"succeed"}]}]}"#,
    );

    let output = run_cli(&["catalog-normalize", input.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let normalized: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(normalized["programs"][0]["id"], 1);
    assert_eq!(normalized["programs"][1]["id"], 2);
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert!(stdout.contains("\n  \"programs\": [\n"));

    let _ = fs::remove_file(input);
}

#[test]
fn catalog_normalize_preserves_compile_validity() {
    let input = write_temp_json(
        "catalog-normalize-validity",
        r#"{
  "programs": [
    { "id": 3, "ops": [{ "op": "succeed" }] },
    { "id": 1, "ops": [{ "op": "spawn", "program": 3 }, { "op": "succeed" }] }
  ]
}"#,
    );
    let output_path = write_temp_json("catalog-normalize-output", "{}");

    let output = run_cli(&[
        "catalog-normalize",
        input.to_str().expect("utf-8 path"),
        output_path.to_str().expect("utf-8 path"),
    ]);
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());

    let check = run_cli(&["catalog-check", output_path.to_str().expect("utf-8 path")]);
    assert!(check.status.success(), "stderr={}", String::from_utf8_lossy(&check.stderr));

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output_path);
}

#[test]
fn snapshot_normalize_rewrites_to_canonical_json() {
    let input = write_temp_json(
        "snapshot-normalize",
        r#"{
  "clock": 9,
  "next_task_id": 12,
  "tasks": [
    null,
    {
      "id": 2,
      "program_id": 2,
      "mind_id": 2,
      "ip": 1,
      "parent": 1,
      "scope_root": 1,
      "outcome": "running",
      "wait": { "kind": "signal", "signal": 8 }
    },
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 3,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": { "kind": "race_or_ticks", "left": 2, "right": 3, "until_tick": 11 }
    }
  ],
  "pending_signals": [null, 7, null, 8]
}"#,
    );

    let output = run_cli(&["snapshot-normalize", input.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let normalized: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(normalized["tasks"][0]["id"], 1);
    assert_eq!(normalized["tasks"][1]["id"], 2);
    assert_eq!(normalized["pending_signals"], serde_json::json!([7, 8]));

    let _ = fs::remove_file(input);
}

#[test]
fn snapshot_normalize_preserves_wait_kind_structure() {
    let input = write_temp_json(
        "snapshot-normalize-structure",
        r#"{
  "clock": 4,
  "next_task_id": 2,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 0,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "repeat_until_predicate",
        "predicate": 5,
        "resume_at_tick": 6
      }
    }
  ],
  "pending_signals": [null]
}"#,
    );
    let output_path = write_temp_json("snapshot-normalize-output", "{}");

    let output = run_cli(&[
        "snapshot-normalize",
        input.to_str().expect("utf-8 path"),
        output_path.to_str().expect("utf-8 path"),
    ]);
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));

    let normalized: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read normalized output"))
            .expect("parse normalized snapshot");
    assert_eq!(normalized["tasks"][0]["wait"]["kind"], "repeat_until_predicate");
    assert_eq!(normalized["tasks"][0]["wait"]["predicate"], 5);
    assert_eq!(normalized["tasks"][0]["wait"]["resume_at_tick"], 6);
    assert_eq!(normalized["pending_signals"], serde_json::json!([]));

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output_path);
}

#[test]
fn asset_bundle_check_accepts_valid_manifest_with_multiple_snapshots() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot_a = fixture("fixtures/contracts/snapshot.valid.json");
    let snapshot_b = write_temp_json(
        "asset-bundle-valid-snapshot",
        r#"{
  "clock": 1,
  "next_task_id": 2,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 1
      }
    }
  ],
  "pending_signals": []
}"#,
    );
    let manifest =
        write_asset_bundle_manifest(&catalog, &[("beta", &snapshot_a), ("alpha", &snapshot_b)]);

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "asset_bundle_check");
    assert_eq!(summary["ok"], true);
    assert_eq!(summary["catalog_program_count"], 3);
    assert_eq!(summary["snapshot_count"], 2);
    assert_eq!(summary["snapshot_ids"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(summary["total_task_count"], 4);
    assert_eq!(summary["snapshots"][0]["id"], "alpha");
    assert_eq!(summary["snapshots"][0]["task_count"], 1);
    assert_eq!(summary["snapshots"][1]["id"], "beta");
    assert_eq!(summary["snapshots"][1]["task_count"], 3);

    let _ = fs::remove_file(snapshot_b);
    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_check_rejects_manifest_with_missing_referenced_file() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after epoch").as_nanos();
    let missing_snapshot =
        env::temp_dir().join(format!("switchyard-cli-missing-snapshot-{stamp}.json"));
    let manifest = write_asset_bundle_manifest(&catalog, &[("missing", &missing_snapshot)]);

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("asset bundle manifest snapshot path not found"));
    assert!(stderr.contains("missing"));

    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_check_rejects_incompatible_snapshot() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let mut snapshot: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fixtures/contracts/snapshot.valid.json"))
            .expect("read snapshot fixture"),
    )
    .expect("parse snapshot fixture");
    snapshot["tasks"][0]["program_id"] = serde_json::json!(99);
    let snapshot_path = write_temp_value("asset-bundle-incompatible-snapshot", &snapshot);
    let manifest = write_asset_bundle_manifest(&catalog, &[("broken", &snapshot_path)]);

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("snapshot validation failed"));
    assert!(stderr.contains("broken"));
    assert!(stderr.contains("task 1 references unknown program 99"));

    let _ = fs::remove_file(snapshot_path);
    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_check_rejects_duplicate_snapshot_ids() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot = fixture("fixtures/contracts/snapshot.valid.json");
    let manifest = write_asset_bundle_manifest(&catalog, &[("dup", &snapshot), ("dup", &snapshot)]);

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("duplicate snapshot id dup"));

    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_check_reports_snapshots_in_deterministic_id_order() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot_a = write_temp_json(
        "asset-bundle-order-a",
        r#"{
  "clock": 2,
  "next_task_id": 2,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 1,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 1
      }
    }
  ],
  "pending_signals": []
}"#,
    );
    let snapshot_b = fixture("fixtures/contracts/snapshot.valid.json");
    let manifest =
        write_asset_bundle_manifest(&catalog, &[("zeta", &snapshot_b), ("alpha", &snapshot_a)]);

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["snapshot_ids"], serde_json::json!(["alpha", "zeta"]));
    assert_eq!(summary["snapshots"][0]["id"], "alpha");
    assert_eq!(summary["snapshots"][1]["id"], "zeta");
    assert_eq!(summary["snapshot_count"], 2);
    assert_eq!(summary["total_task_count"], 4);

    let _ = fs::remove_file(snapshot_a);
    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_check_rejects_unsupported_manifest_version() {
    let manifest = write_temp_value(
        "asset-bundle-version",
        &serde_json::json!({
            "version": 2,
            "catalog": fixture("fixtures/contracts/program.valid.json").to_str().expect("utf-8 path"),
            "snapshots": [],
        }),
    );

    let output = run_cli(&["asset-bundle-check", manifest.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("asset bundle manifest version 2 is unsupported"));

    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_summary_reports_catalog_and_snapshot_metadata() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot_a = fixture("fixtures/contracts/snapshot.valid.json");
    let snapshot_b = write_temp_json(
        "asset-bundle-summary-snapshot",
        r#"{
  "clock": 4,
  "next_task_id": 3,
  "tasks": [
    {
      "id": 1,
      "program_id": 2,
      "mind_id": 3,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 7
      }
    }
  ],
  "pending_signals": [7]
}"#,
    );
    let manifest =
        write_asset_bundle_manifest(&catalog, &[("beta", &snapshot_a), ("alpha", &snapshot_b)]);

    let output = run_cli(&["asset-bundle-summary", manifest.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["kind"], "asset_bundle_summary");
    assert_eq!(summary["manifest_version"], 1);
    assert_eq!(summary["catalog_program_count"], 3);
    assert_eq!(summary["catalog_program_ids"], serde_json::json!([1, 2, 3]));
    assert_eq!(summary["snapshot_count"], 2);
    assert_eq!(summary["snapshot_ids"], serde_json::json!(["alpha", "beta"]));
    assert_eq!(summary["snapshots"][0]["id"], "alpha");
    assert_eq!(summary["snapshots"][0]["clock"], 4);
    assert_eq!(summary["snapshots"][0]["pending_signal_count"], 1);
    assert_eq!(summary["snapshots"][0]["program_ids"], serde_json::json!([2]));
    assert_eq!(summary["snapshots"][0]["mind_ids"], serde_json::json!([3]));
    assert_eq!(summary["snapshots"][1]["id"], "beta");
    assert_eq!(summary["snapshots"][1]["task_count"], 3);

    let _ = fs::remove_file(snapshot_b);
    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_summary_reports_snapshots_in_deterministic_id_order() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let snapshot_a = write_temp_json(
        "asset-bundle-summary-order-a",
        r#"{
  "clock": 5,
  "next_task_id": 2,
  "tasks": [
    {
      "id": 1,
      "program_id": 1,
      "mind_id": 2,
      "ip": 1,
      "parent": null,
      "scope_root": 1,
      "outcome": "running",
      "wait": {
        "kind": "signal",
        "signal": 1
      }
    }
  ],
  "pending_signals": []
}"#,
    );
    let snapshot_b = fixture("fixtures/contracts/snapshot.valid.json");
    let manifest =
        write_asset_bundle_manifest(&catalog, &[("zeta", &snapshot_b), ("alpha", &snapshot_a)]);

    let output = run_cli(&["asset-bundle-summary", manifest.to_str().expect("utf-8 path")]);

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).expect("parse stdout json");
    assert_eq!(summary["snapshot_ids"], serde_json::json!(["alpha", "zeta"]));
    assert_eq!(summary["snapshots"][0]["id"], "alpha");
    assert_eq!(summary["snapshots"][1]["id"], "zeta");

    let _ = fs::remove_file(snapshot_a);
    let _ = fs::remove_file(manifest);
}

#[test]
fn asset_bundle_summary_rejects_manifest_with_missing_referenced_file() {
    let catalog = fixture("fixtures/contracts/program.valid.json");
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock after epoch").as_nanos();
    let missing_snapshot =
        env::temp_dir().join(format!("switchyard-cli-missing-summary-snapshot-{stamp}.json"));
    let manifest = write_asset_bundle_manifest(&catalog, &[("missing", &missing_snapshot)]);

    let output = run_cli(&["asset-bundle-summary", manifest.to_str().expect("utf-8 path")]);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("asset bundle manifest snapshot path not found"));
    assert!(stderr.contains("missing"));

    let _ = fs::remove_file(manifest);
}
