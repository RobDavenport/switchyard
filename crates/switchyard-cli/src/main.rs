use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use switchyard_core::{
    CompileError, MindId, OpDocument, OwnedProgramCatalog, ProgramCatalogDocument, ProgramId,
    SignalId, TaskId,
};

fn main() -> ExitCode {
    match run(env::args().collect()) {
        Ok(CommandResult::Print(output)) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Ok(CommandResult::Silent) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<CommandResult, String> {
    match args.as_slice() {
        [_, command, path] => match command.as_str() {
            "asset-bundle-check" => asset_bundle_check(Path::new(path)),
            "asset-bundle-summary" => asset_bundle_summary(Path::new(path)),
            "asset-bundle-normalize" => asset_bundle_normalize(Path::new(path), None),
            "catalog-check" => catalog_check(Path::new(path)),
            "catalog-summary" => catalog_summary(Path::new(path)),
            "snapshot-summary" => snapshot_summary(Path::new(path)),
            "catalog-normalize" => catalog_normalize(Path::new(path), None),
            "snapshot-normalize" => snapshot_normalize(Path::new(path), None),
            other => Err(format!("unknown command: {other}\n{}", usage())),
        },
        [_, command, path, output] => match command.as_str() {
            "asset-bundle-normalize" => {
                asset_bundle_normalize(Path::new(path), Some(Path::new(output)))
            }
            "catalog-normalize" => catalog_normalize(Path::new(path), Some(Path::new(output))),
            "snapshot-normalize" => snapshot_normalize(Path::new(path), Some(Path::new(output))),
            "snapshot-check" => snapshot_check(Path::new(path), Path::new(output)),
            other => Err(format!("unknown command: {other}\n{}", usage())),
        },
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: switchyard-cli <asset-bundle-check|asset-bundle-summary|catalog-check|catalog-summary|snapshot-summary> <path>\n       switchyard-cli <asset-bundle-normalize|catalog-normalize|snapshot-normalize> <input> [output]\n       switchyard-cli <snapshot-check> <catalog> <snapshot>".to_owned()
}

fn asset_bundle_check(path: &Path) -> Result<CommandResult, String> {
    let summary = build_asset_bundle_summary(path)?;
    let snapshot_ids = summary.snapshots.iter().map(|snapshot| snapshot.id.clone()).collect();
    let total_task_count = summary.snapshots.iter().map(|snapshot| snapshot.task_count).sum();
    let summary = AssetBundleCheckSummary {
        kind: "asset_bundle_check",
        ok: true,
        catalog_program_count: summary.catalog_program_count,
        snapshot_count: summary.snapshots.len(),
        snapshot_ids,
        total_task_count,
        snapshots: summary
            .snapshots
            .into_iter()
            .map(|snapshot| AssetBundleSnapshotSummary {
                id: snapshot.id,
                task_count: snapshot.task_count,
                root_task_count: snapshot.root_task_count,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("asset bundle check encode failed: {error}"))
}

fn asset_bundle_summary(path: &Path) -> Result<CommandResult, String> {
    let summary = build_asset_bundle_summary(path)?;
    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("asset bundle summary encode failed: {error}"))
}

fn asset_bundle_normalize(
    path: &Path,
    output_path: Option<&Path>,
) -> Result<CommandResult, String> {
    let mut manifest = read_asset_bundle_manifest(path)?;
    validate_asset_bundle_manifest(path, &manifest)?;
    manifest.snapshots.sort_by(|left, right| left.id.cmp(&right.id));
    let normalized = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("asset bundle normalize encode failed: {error}"))?;
    emit_or_write(normalized, output_path)
}

fn catalog_check(path: &Path) -> Result<CommandResult, String> {
    let text = read_text(path)?;
    let document: ProgramCatalogDocument =
        serde_json::from_str(&text).map_err(|error| format!("catalog decode failed: {error}"))?;
    let catalog = document
        .compile()
        .map_err(|error| format!("catalog compile failed: {}", compile_error_label(error)))?;

    let summary = CatalogCheckSummary {
        kind: "catalog_check",
        ok: true,
        program_count: catalog.programs().len(),
        program_ids: catalog.programs().iter().map(|program| program.id().0).collect(),
    };
    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("catalog check encode failed: {error}"))
}

fn catalog_summary(path: &Path) -> Result<CommandResult, String> {
    let text = read_text(path)?;
    let document: ProgramCatalogDocument =
        serde_json::from_str(&text).map_err(|error| format!("catalog decode failed: {error}"))?;
    document
        .compile()
        .map_err(|error| format!("catalog compile failed: {}", compile_error_label(error)))?;

    let summary = build_catalog_summary(&document);
    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("catalog summary encode failed: {error}"))
}

fn snapshot_summary(path: &Path) -> Result<CommandResult, String> {
    let text = read_text(path)?;
    let snapshot: DynamicSnapshot =
        serde_json::from_str(&text).map_err(|error| format!("snapshot decode failed: {error}"))?;
    let summary = build_snapshot_summary(&snapshot);
    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("snapshot summary encode failed: {error}"))
}

fn snapshot_check(catalog_path: &Path, snapshot_path: &Path) -> Result<CommandResult, String> {
    let catalog = read_catalog(catalog_path)?;
    let snapshot = read_snapshot(snapshot_path)?;
    let normalized = NormalizedSnapshot::from(snapshot);
    validate_snapshot_against_catalog(&catalog, &normalized)
        .map_err(|error| format!("snapshot validation failed: {error}"))?;

    let program_ids = catalog.programs().iter().map(|program| program.id().0).collect();
    let task_ids = normalized.tasks.iter().map(|task| task.id.0).collect();
    let root_task_ids = normalized
        .tasks
        .iter()
        .filter(|task| task.parent.is_none())
        .map(|task| task.id.0)
        .collect();
    let summary = SnapshotCheckSummary {
        kind: "snapshot_check",
        ok: true,
        catalog_program_count: catalog.programs().len(),
        task_count: normalized.tasks.len(),
        program_ids,
        task_ids,
        root_task_ids,
        pending_signals: normalized.pending_signals.iter().map(|signal| signal.0).collect(),
    };
    serde_json::to_string_pretty(&summary)
        .map(CommandResult::Print)
        .map_err(|error| format!("snapshot check encode failed: {error}"))
}

fn catalog_normalize(path: &Path, output_path: Option<&Path>) -> Result<CommandResult, String> {
    let mut document = read_catalog_document(path)?;
    document
        .compile()
        .map_err(|error| format!("catalog compile failed: {}", compile_error_label(error)))?;
    document.programs.sort_by_key(|program| program.id.0);
    let normalized = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("catalog normalize encode failed: {error}"))?;
    emit_or_write(normalized, output_path)
}

fn snapshot_normalize(path: &Path, output_path: Option<&Path>) -> Result<CommandResult, String> {
    let normalized = NormalizedSnapshot::from(read_snapshot(path)?);
    let normalized = serde_json::to_string_pretty(&normalized)
        .map_err(|error| format!("snapshot normalize encode failed: {error}"))?;
    emit_or_write(normalized, output_path)
}

fn read_catalog_document(path: &Path) -> Result<ProgramCatalogDocument, String> {
    let text = read_text(path)?;
    serde_json::from_str(&text).map_err(|error| format!("catalog decode failed: {error}"))
}

fn read_asset_bundle_manifest(path: &Path) -> Result<AssetBundleManifest, String> {
    let text = read_text(path)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("asset bundle manifest decode failed: {error}"))
}

fn read_catalog(path: &Path) -> Result<OwnedProgramCatalog, String> {
    read_catalog_document(path)?
        .compile()
        .map_err(|error| format!("catalog compile failed: {}", compile_error_label(error)))
}

fn read_snapshot(path: &Path) -> Result<DynamicSnapshot, String> {
    let text = read_text(path)?;
    serde_json::from_str(&text).map_err(|error| format!("snapshot decode failed: {error}"))
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn resolve_manifest_reference(manifest_path: &Path, reference: &str) -> PathBuf {
    let reference = Path::new(reference);
    if reference.is_absolute() {
        reference.to_path_buf()
    } else {
        manifest_path.parent().unwrap_or_else(|| Path::new(".")).join(reference)
    }
}

fn ensure_manifest_reference_exists(
    manifest_path: &Path,
    kind: &'static str,
    id: Option<&str>,
    reference_path: &Path,
) -> Result<(), String> {
    if reference_path.exists() {
        return Ok(());
    }

    match id {
        Some(id) => Err(format!(
            "asset bundle manifest {kind} path not found for {id}: {} (from {})",
            reference_path.display(),
            manifest_path.display()
        )),
        None => Err(format!(
            "asset bundle manifest {kind} path not found: {} (from {})",
            reference_path.display(),
            manifest_path.display()
        )),
    }
}

fn emit_or_write(text: String, output_path: Option<&Path>) -> Result<CommandResult, String> {
    if let Some(path) = output_path {
        fs::write(path, &text).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(CommandResult::Silent)
    } else {
        Ok(CommandResult::Print(text))
    }
}

fn build_asset_bundle_summary(path: &Path) -> Result<AssetBundleSummary, String> {
    let manifest = read_asset_bundle_manifest(path)?;
    validate_asset_bundle_manifest(path, &manifest)?;
    let catalog_path = resolve_manifest_reference(path, &manifest.catalog);
    let catalog = read_catalog(&catalog_path)?;
    let catalog_summary = build_catalog_summary(&read_catalog_document(&catalog_path)?);

    let mut snapshots = Vec::with_capacity(manifest.snapshots.len());
    for snapshot in &manifest.snapshots {
        let snapshot_path = resolve_manifest_reference(path, &snapshot.path);
        let normalized = NormalizedSnapshot::from(read_snapshot(&snapshot_path)?);
        let root_task_count = normalized.tasks.iter().filter(|task| task.parent.is_none()).count();
        let pending_signal_count = normalized.pending_signals.len();
        let mut program_ids = BTreeSet::<u16>::new();
        let mut mind_ids = BTreeSet::<u16>::new();

        for task in &normalized.tasks {
            program_ids.insert(task.program_id.0);
            mind_ids.insert(task.mind_id.0);
        }

        snapshots.push(AssetBundleDetailedSnapshotSummary {
            id: snapshot.id.clone(),
            path: snapshot.path.clone(),
            clock: normalized.clock,
            next_task_id: normalized.next_task_id,
            task_count: normalized.tasks.len(),
            root_task_count,
            pending_signal_count,
            program_ids: program_ids.into_iter().collect(),
            mind_ids: mind_ids.into_iter().collect(),
        });
    }

    snapshots.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(AssetBundleSummary {
        kind: "asset_bundle_summary",
        manifest_version: manifest.version,
        catalog_path: manifest.catalog,
        catalog_program_count: catalog.programs().len(),
        catalog_program_ids: catalog_summary.program_ids,
        snapshot_count: snapshots.len(),
        snapshot_ids: snapshots.iter().map(|snapshot| snapshot.id.clone()).collect(),
        total_task_count: snapshots.iter().map(|snapshot| snapshot.task_count).sum(),
        snapshots,
    })
}

fn validate_asset_bundle_manifest(
    path: &Path,
    manifest: &AssetBundleManifest,
) -> Result<(), String> {
    if manifest.version != 1 {
        return Err(format!(
            "asset bundle manifest version {} is unsupported; expected 1",
            manifest.version
        ));
    }

    let catalog_path = resolve_manifest_reference(path, &manifest.catalog);
    ensure_manifest_reference_exists(path, "catalog", None, &catalog_path)?;
    let catalog = read_catalog(&catalog_path)?;

    let mut snapshot_ids = BTreeSet::<&str>::new();
    for snapshot in &manifest.snapshots {
        if snapshot.id.trim().is_empty() {
            return Err("asset bundle manifest snapshot id must not be empty".to_owned());
        }
        if !snapshot_ids.insert(&snapshot.id) {
            return Err(format!("asset bundle manifest duplicate snapshot id {}", snapshot.id));
        }

        let snapshot_path = resolve_manifest_reference(path, &snapshot.path);
        ensure_manifest_reference_exists(path, "snapshot", Some(&snapshot.id), &snapshot_path)?;

        let normalized = NormalizedSnapshot::from(read_snapshot(&snapshot_path)?);
        validate_snapshot_against_catalog(&catalog, &normalized)
            .map_err(|error| format!("snapshot validation failed for {}: {error}", snapshot.id))?;
    }

    Ok(())
}

fn build_catalog_summary(document: &ProgramCatalogDocument) -> CatalogSummary {
    let mut op_histogram = BTreeMap::<&'static str, usize>::new();
    let mut referenced_program_ids = BTreeSet::<u16>::new();
    let mut signal_ids = BTreeSet::<u16>::new();
    let mut predicate_ids = BTreeSet::<u16>::new();
    let mut mind_ids = BTreeSet::<u16>::new();
    let mut host_call_ids = BTreeSet::<u16>::new();

    for program in &document.programs {
        for op in &program.ops {
            *op_histogram.entry(op_kind(op)).or_default() += 1;
            match op {
                OpDocument::Call { call, .. } => {
                    host_call_ids.insert(call.0);
                }
                OpDocument::ChangeMind { mind } => {
                    mind_ids.insert(mind.0);
                }
                OpDocument::RepeatUntilPredicate { predicate, program } => {
                    predicate_ids.insert(predicate.0);
                    referenced_program_ids.insert(program.0);
                }
                OpDocument::WaitSignalUntilTick { signal, .. }
                | OpDocument::WaitSignalOrTicks { signal, .. }
                | OpDocument::WaitSignal { signal } => {
                    signal_ids.insert(signal.0);
                }
                OpDocument::WaitPredicate { predicate }
                | OpDocument::BranchPredicate { predicate, .. } => {
                    predicate_ids.insert(predicate.0);
                    if let OpDocument::BranchPredicate { if_true, if_false, .. } = op {
                        referenced_program_ids.insert(if_true.0);
                        referenced_program_ids.insert(if_false.0);
                    }
                }
                OpDocument::Spawn { program }
                | OpDocument::RepeatCount { program, .. }
                | OpDocument::TimeoutUntilTick { program, .. }
                | OpDocument::TimeoutTicks { program, .. } => {
                    referenced_program_ids.insert(program.0);
                }
                OpDocument::RaceChildrenUntilTick { left, right, .. }
                | OpDocument::RaceChildrenOrTicks { left, right, .. }
                | OpDocument::SyncChildren { left, right }
                | OpDocument::RaceChildren { left, right }
                | OpDocument::Race2 { left, right } => {
                    referenced_program_ids.insert(left.0);
                    referenced_program_ids.insert(right.0);
                }
                _ => {}
            }
        }
    }

    CatalogSummary {
        kind: "catalog_summary",
        program_count: document.programs.len(),
        program_ids: document.programs.iter().map(|program| program.id.0).collect(),
        referenced_program_ids: referenced_program_ids.into_iter().collect(),
        op_histogram,
        signal_ids: signal_ids.into_iter().collect(),
        predicate_ids: predicate_ids.into_iter().collect(),
        mind_ids: mind_ids.into_iter().collect(),
        host_call_ids: host_call_ids.into_iter().collect(),
    }
}

fn build_snapshot_summary(snapshot: &DynamicSnapshot) -> SnapshotSummary {
    let mut task_count = 0usize;
    let mut program_ids = BTreeSet::<u16>::new();
    let mut mind_ids = BTreeSet::<u16>::new();
    let mut wait_kind_histogram = BTreeMap::<&'static str, usize>::new();
    let mut outcome_histogram = BTreeMap::<&'static str, usize>::new();

    for task in snapshot.tasks.iter().flatten() {
        task_count += 1;
        program_ids.insert(task.program_id.0);
        mind_ids.insert(task.mind_id.0);
        *wait_kind_histogram.entry(wait_kind(task.wait)).or_default() += 1;
        *outcome_histogram.entry(outcome_kind(task.outcome)).or_default() += 1;
    }

    SnapshotSummary {
        kind: "snapshot_summary",
        clock: snapshot.clock,
        next_task_id: snapshot.next_task_id,
        task_count,
        program_ids: program_ids.into_iter().collect(),
        mind_ids: mind_ids.into_iter().collect(),
        pending_signals: snapshot.pending_signals.iter().flatten().map(|signal| signal.0).collect(),
        wait_kind_histogram,
        outcome_histogram,
    }
}

fn compile_error_label(error: CompileError) -> String {
    match error {
        CompileError::EmptyCatalog => "catalog is empty".to_owned(),
        CompileError::EmptyProgram { program_id } => {
            format!("program {} has no ops", program_id.0)
        }
        CompileError::DuplicateProgramId(program_id) => {
            format!("duplicate program id {}", program_id.0)
        }
        CompileError::UnknownProgramReference { owner, target } => {
            format!("program {} references missing program {}", owner.0, target.0)
        }
    }
}

fn op_kind(op: &OpDocument) -> &'static str {
    match op {
        OpDocument::Action { .. } => "action",
        OpDocument::Call { .. } => "call",
        OpDocument::ChangeMind { .. } => "change_mind",
        OpDocument::RepeatUntilPredicate { .. } => "repeat_until_predicate",
        OpDocument::WaitUntilTick { .. } => "wait_until_tick",
        OpDocument::WaitSignalUntilTick { .. } => "wait_signal_until_tick",
        OpDocument::TimeoutUntilTick { .. } => "timeout_until_tick",
        OpDocument::RaceChildrenUntilTick { .. } => "race_children_until_tick",
        OpDocument::RaceChildrenOrTicks { .. } => "race_children_or_ticks",
        OpDocument::TimeoutTicks { .. } => "timeout_ticks",
        OpDocument::WaitSignalOrTicks { .. } => "wait_signal_or_ticks",
        OpDocument::WaitTicks { .. } => "wait_ticks",
        OpDocument::WaitSignal { .. } => "wait_signal",
        OpDocument::WaitPredicate { .. } => "wait_predicate",
        OpDocument::Spawn { .. } => "spawn",
        OpDocument::RepeatCount { .. } => "repeat_count",
        OpDocument::SyncChildren { .. } => "sync_children",
        OpDocument::BranchPredicate { .. } => "branch_predicate",
        OpDocument::JoinChildren => "join_children",
        OpDocument::JoinAnyChildren => "join_any_children",
        OpDocument::RaceChildren { .. } => "race_children",
        OpDocument::Race2 { .. } => "race2",
        OpDocument::Succeed => "succeed",
        OpDocument::Fail => "fail",
    }
}

fn wait_kind(wait: ContractWaitReason) -> &'static str {
    match wait {
        ContractWaitReason::Ready => "ready",
        ContractWaitReason::Ticks { .. } => "ticks",
        ContractWaitReason::Signal { .. } => "signal",
        ContractWaitReason::Predicate { .. } => "predicate",
        ContractWaitReason::SignalOrTicks { .. } => "signal_or_ticks",
        ContractWaitReason::RaceOrTicks { .. } => "race_or_ticks",
        ContractWaitReason::Timeout { .. } => "timeout",
        ContractWaitReason::RepeatUntilPredicate { .. } => "repeat_until_predicate",
        ContractWaitReason::ChildrenAll => "children_all",
        ContractWaitReason::ChildrenAny => "children_any",
        ContractWaitReason::Race { .. } => "race",
    }
}

fn outcome_kind(outcome: ContractOutcome) -> &'static str {
    match outcome {
        ContractOutcome::Running => "running",
        ContractOutcome::Succeeded => "succeeded",
        ContractOutcome::Failed => "failed",
        ContractOutcome::Cancelled => "cancelled",
    }
}

fn validate_snapshot_against_catalog(
    catalog: &OwnedProgramCatalog,
    snapshot: &NormalizedSnapshot,
) -> Result<(), String> {
    let valid_program_ids =
        catalog.programs().iter().map(|program| program.id().0).collect::<BTreeSet<_>>();
    let mut tasks_by_id = BTreeMap::<u32, &ContractTaskRecord>::new();

    for task in &snapshot.tasks {
        if !valid_program_ids.contains(&task.program_id.0) {
            return Err(format!(
                "task {} references unknown program {}",
                task.id.0, task.program_id.0
            ));
        }
        if tasks_by_id.insert(task.id.0, task).is_some() {
            return Err(format!("duplicate task id {}", task.id.0));
        }
    }

    for task in &snapshot.tasks {
        if let Some(parent) = task.parent {
            if !tasks_by_id.contains_key(&parent.0) {
                return Err(format!(
                    "task {} references missing parent task {}",
                    task.id.0, parent.0
                ));
            }
        }

        if !tasks_by_id.contains_key(&task.scope_root.0) {
            return Err(format!(
                "task {} references missing scope root task {}",
                task.id.0, task.scope_root.0
            ));
        }

        validate_wait_references(task, &tasks_by_id)?;
    }

    Ok(())
}

fn validate_wait_references(
    task: &ContractTaskRecord,
    tasks_by_id: &BTreeMap<u32, &ContractTaskRecord>,
) -> Result<(), String> {
    match task.wait {
        ContractWaitReason::Race { left, right }
        | ContractWaitReason::RaceOrTicks { left, right, .. } => {
            if !tasks_by_id.contains_key(&left.0) {
                return Err(format!("task {} wait references missing task {}", task.id.0, left.0));
            }
            if !tasks_by_id.contains_key(&right.0) {
                return Err(format!("task {} wait references missing task {}", task.id.0, right.0));
            }
        }
        ContractWaitReason::Timeout { child, .. } => {
            if !tasks_by_id.contains_key(&child.0) {
                return Err(format!("task {} wait references missing task {}", task.id.0, child.0));
            }
        }
        ContractWaitReason::Ready
        | ContractWaitReason::Ticks { .. }
        | ContractWaitReason::Signal { .. }
        | ContractWaitReason::Predicate { .. }
        | ContractWaitReason::SignalOrTicks { .. }
        | ContractWaitReason::RepeatUntilPredicate { .. }
        | ContractWaitReason::ChildrenAll
        | ContractWaitReason::ChildrenAny => {}
    }

    Ok(())
}

#[derive(Serialize)]
struct CatalogCheckSummary {
    kind: &'static str,
    ok: bool,
    program_count: usize,
    program_ids: Vec<u16>,
}

#[derive(Serialize)]
struct CatalogSummary {
    kind: &'static str,
    program_count: usize,
    program_ids: Vec<u16>,
    referenced_program_ids: Vec<u16>,
    op_histogram: BTreeMap<&'static str, usize>,
    signal_ids: Vec<u16>,
    predicate_ids: Vec<u16>,
    mind_ids: Vec<u16>,
    host_call_ids: Vec<u16>,
}

#[derive(Serialize)]
struct SnapshotSummary {
    kind: &'static str,
    clock: u64,
    next_task_id: u32,
    task_count: usize,
    program_ids: Vec<u16>,
    mind_ids: Vec<u16>,
    pending_signals: Vec<u16>,
    wait_kind_histogram: BTreeMap<&'static str, usize>,
    outcome_histogram: BTreeMap<&'static str, usize>,
}

#[derive(Serialize)]
struct SnapshotCheckSummary {
    kind: &'static str,
    ok: bool,
    catalog_program_count: usize,
    task_count: usize,
    program_ids: Vec<u16>,
    task_ids: Vec<u32>,
    root_task_ids: Vec<u32>,
    pending_signals: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
struct AssetBundleManifest {
    version: u32,
    catalog: String,
    snapshots: Vec<AssetBundleSnapshotManifestEntry>,
}

#[derive(Serialize, Deserialize)]
struct AssetBundleSnapshotManifestEntry {
    id: String,
    path: String,
}

#[derive(Serialize)]
struct AssetBundleCheckSummary {
    kind: &'static str,
    ok: bool,
    catalog_program_count: usize,
    snapshot_count: usize,
    snapshot_ids: Vec<String>,
    total_task_count: usize,
    snapshots: Vec<AssetBundleSnapshotSummary>,
}

#[derive(Serialize)]
struct AssetBundleSnapshotSummary {
    id: String,
    task_count: usize,
    root_task_count: usize,
}

#[derive(Serialize)]
struct AssetBundleSummary {
    kind: &'static str,
    manifest_version: u32,
    catalog_path: String,
    catalog_program_count: usize,
    catalog_program_ids: Vec<u16>,
    snapshot_count: usize,
    snapshot_ids: Vec<String>,
    total_task_count: usize,
    snapshots: Vec<AssetBundleDetailedSnapshotSummary>,
}

#[derive(Serialize)]
struct AssetBundleDetailedSnapshotSummary {
    id: String,
    path: String,
    clock: u64,
    next_task_id: u32,
    task_count: usize,
    root_task_count: usize,
    pending_signal_count: usize,
    program_ids: Vec<u16>,
    mind_ids: Vec<u16>,
}

#[derive(Serialize, serde::Deserialize)]
struct DynamicSnapshot {
    clock: u64,
    next_task_id: u32,
    tasks: Vec<Option<ContractTaskRecord>>,
    pending_signals: Vec<Option<SignalId>>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContractOutcome {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
struct ContractTaskRecord {
    id: TaskId,
    program_id: ProgramId,
    mind_id: MindId,
    ip: usize,
    parent: Option<TaskId>,
    scope_root: TaskId,
    outcome: ContractOutcome,
    wait: ContractWaitReason,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ContractWaitReason {
    Ready,
    Ticks { until_tick: u64 },
    Signal { signal: SignalId },
    Predicate { predicate: switchyard_core::PredicateId },
    SignalOrTicks { signal: SignalId, until_tick: u64 },
    RaceOrTicks { left: TaskId, right: TaskId, until_tick: u64 },
    Timeout { child: TaskId, until_tick: u64 },
    RepeatUntilPredicate { predicate: switchyard_core::PredicateId, resume_at_tick: u64 },
    ChildrenAll,
    ChildrenAny,
    Race { left: TaskId, right: TaskId },
}

enum CommandResult {
    Print(String),
    Silent,
}

#[derive(Serialize)]
struct NormalizedSnapshot {
    clock: u64,
    next_task_id: u32,
    tasks: Vec<ContractTaskRecord>,
    pending_signals: Vec<SignalId>,
}

impl From<DynamicSnapshot> for NormalizedSnapshot {
    fn from(snapshot: DynamicSnapshot) -> Self {
        let mut tasks: Vec<ContractTaskRecord> = snapshot.tasks.into_iter().flatten().collect();
        tasks.sort_by_key(|task| task.id.0);
        let pending_signals = snapshot.pending_signals.into_iter().flatten().collect();
        Self { clock: snapshot.clock, next_task_id: snapshot.next_task_id, tasks, pending_signals }
    }
}
