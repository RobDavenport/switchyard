#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures" / "contracts"

def fail(message: str) -> None:
    print(f"[contract-validation] {message}", file=sys.stderr)
    raise SystemExit(1)

def load(name: str):
    with (FIXTURES / name).open("r", encoding="utf-8") as handle:
        return json.load(handle)

def is_positive_int(value) -> bool:
    return isinstance(value, int) and value >= 1

def validate_program_fixture(payload, expect_valid: bool) -> None:
    ok = True
    if not isinstance(payload, dict) or "programs" not in payload or not isinstance(payload["programs"], list):
        ok = False
    else:
        for program in payload["programs"]:
            if not isinstance(program, dict):
                ok = False
                continue
            if not is_positive_int(program.get("id")):
                ok = False
            ops = program.get("ops")
            if not isinstance(ops, list) or len(ops) == 0:
                ok = False
                continue
            for op in ops:
                if not isinstance(op, dict):
                    ok = False
                    continue
                kind = op.get("op")
                if kind not in {
                    "action",
                    "call",
                    "change_mind",
                    "repeat_until_predicate",
                    "wait_until_tick",
                    "wait_signal_until_tick",
                    "timeout_until_tick",
                    "race_children_until_tick",
                    "race_children_or_ticks",
                    "timeout_ticks",
                    "wait_ticks",
                    "wait_signal",
                    "wait_signal_or_ticks",
                    "wait_predicate",
                    "spawn",
                    "sync_children",
                    "branch_predicate",
                    "repeat_count",
                    "join_children",
                    "join_any_children",
                    "race_children",
                    "race2",
                    "succeed",
                    "fail",
                }:
                    ok = False
                    continue
                if kind == "action" and not is_positive_int(op.get("action")):
                    ok = False
                if kind == "call" and not (
                    is_positive_int(op.get("call"))
                    and isinstance(op.get("arg0"), int)
                    and isinstance(op.get("arg1"), int)
                    and isinstance(op.get("arg2"), int)
                    and isinstance(op.get("arg3"), int)
                ):
                    ok = False
                if kind == "change_mind" and not is_positive_int(op.get("mind")):
                    ok = False
                if kind == "repeat_until_predicate" and not (
                    is_positive_int(op.get("predicate")) and is_positive_int(op.get("program"))
                ):
                    ok = False
                if kind == "wait_until_tick" and not (
                    isinstance(op.get("until_tick"), int) and op.get("until_tick") >= 0
                ):
                    ok = False
                if kind == "wait_signal_until_tick" and not (
                    is_positive_int(op.get("signal"))
                    and isinstance(op.get("until_tick"), int)
                    and op.get("until_tick") >= 0
                ):
                    ok = False
                if kind == "timeout_until_tick" and not (
                    isinstance(op.get("until_tick"), int)
                    and op.get("until_tick") >= 0
                    and is_positive_int(op.get("program"))
                ):
                    ok = False
                if kind == "race_children_until_tick" and not (
                    is_positive_int(op.get("left"))
                    and is_positive_int(op.get("right"))
                    and isinstance(op.get("until_tick"), int)
                    and op.get("until_tick") >= 0
                ):
                    ok = False
                if kind == "race_children_or_ticks" and not (
                    is_positive_int(op.get("left"))
                    and is_positive_int(op.get("right"))
                    and isinstance(op.get("ticks"), int)
                    and op.get("ticks") >= 0
                ):
                    ok = False
                if kind == "timeout_ticks" and not (
                    isinstance(op.get("ticks"), int) and op.get("ticks") >= 0 and is_positive_int(op.get("program"))
                ):
                    ok = False
                if kind == "wait_ticks" and not isinstance(op.get("ticks"), int):
                    ok = False
                if kind == "wait_signal" and not is_positive_int(op.get("signal")):
                    ok = False
                if kind == "wait_signal_or_ticks" and not (
                    is_positive_int(op.get("signal")) and isinstance(op.get("ticks"), int)
                ):
                    ok = False
                if kind == "wait_predicate" and not is_positive_int(op.get("predicate")):
                    ok = False
                if kind == "spawn" and not is_positive_int(op.get("program")):
                    ok = False
                if kind == "sync_children" and not (
                    is_positive_int(op.get("left")) and is_positive_int(op.get("right"))
                ):
                    ok = False
                if kind == "repeat_count" and not (
                    isinstance(op.get("count"), int) and op.get("count") >= 0 and is_positive_int(op.get("program"))
                ):
                    ok = False
                if kind == "branch_predicate" and not (
                    is_positive_int(op.get("predicate"))
                    and is_positive_int(op.get("if_true"))
                    and is_positive_int(op.get("if_false"))
                ):
                    ok = False
                if kind == "race_children" and not (
                    is_positive_int(op.get("left")) and is_positive_int(op.get("right"))
                ):
                    ok = False
                if kind == "race2" and not (is_positive_int(op.get("left")) and is_positive_int(op.get("right"))):
                    ok = False
    if expect_valid and not ok:
        fail("program.valid.json did not satisfy the validator")
    if not expect_valid and ok:
        fail("program.invalid.json unexpectedly passed validation")

def validate_snapshot_fixture(payload, expect_valid: bool) -> None:
    ok = True
    if not isinstance(payload, dict):
        ok = False
    else:
        if not isinstance(payload.get("clock"), int) or payload["clock"] < 0:
            ok = False
        if not isinstance(payload.get("next_task_id"), int) or payload["next_task_id"] < 0:
            ok = False
        if not isinstance(payload.get("pending_signals"), list):
            ok = False
        if not isinstance(payload.get("tasks"), list):
            ok = False
        else:
            for task in payload["tasks"]:
                if task is None:
                    continue
                if not isinstance(task, dict):
                    ok = False
                    continue
                if not is_positive_int(task.get("id")):
                    ok = False
                if not is_positive_int(task.get("program_id")):
                    ok = False
                if not is_positive_int(task.get("mind_id")):
                    ok = False
                if not isinstance(task.get("ip"), int) or task["ip"] < 0:
                    ok = False
                if not is_positive_int(task.get("scope_root")):
                    ok = False
                if task.get("outcome") not in {"running", "succeeded", "failed", "cancelled"}:
                    ok = False
                wait = task.get("wait")
                if not isinstance(wait, dict):
                    ok = False
                    continue
                kind = wait.get("kind")
                if kind not in {
                    "ready",
                    "ticks",
                    "race_or_ticks",
                    "timeout",
                    "signal_or_ticks",
                    "signal",
                    "predicate",
                    "repeat_until_predicate",
                    "children_all",
                    "children_any",
                    "race",
                }:
                    ok = False
                if kind == "ticks" and not isinstance(wait.get("until_tick"), int):
                    ok = False
                if kind == "race_or_ticks" and not (
                    is_positive_int(wait.get("left"))
                    and is_positive_int(wait.get("right"))
                    and isinstance(wait.get("until_tick"), int)
                ):
                    ok = False
                if kind == "timeout" and not (
                    is_positive_int(wait.get("child"))
                    and isinstance(wait.get("until_tick"), int)
                ):
                    ok = False
                if kind == "signal" and not is_positive_int(wait.get("signal")):
                    ok = False
                if kind == "signal_or_ticks" and not (
                    is_positive_int(wait.get("signal"))
                    and isinstance(wait.get("until_tick"), int)
                ):
                    ok = False
                if kind == "predicate" and not is_positive_int(wait.get("predicate")):
                    ok = False
                if kind == "repeat_until_predicate" and not (
                    is_positive_int(wait.get("predicate"))
                    and isinstance(wait.get("resume_at_tick"), int)
                    and wait.get("resume_at_tick") >= 0
                ):
                    ok = False
                if kind == "race" and not (is_positive_int(wait.get("left")) and is_positive_int(wait.get("right"))):
                    ok = False
    if expect_valid and not ok:
        fail("snapshot.valid.json did not satisfy the validator")
    if not expect_valid and ok:
        fail("snapshot.invalid.json unexpectedly passed validation")

def validate_asset_bundle_fixture(payload, expect_valid: bool) -> None:
    ok = True
    if not isinstance(payload, dict):
        ok = False
    else:
        if not isinstance(payload.get("version"), int) or payload["version"] < 1:
            ok = False
        if not isinstance(payload.get("catalog"), str) or payload["catalog"].strip() == "":
            ok = False
        elif expect_valid and not (FIXTURES / payload["catalog"]).exists():
            ok = False
        snapshots = payload.get("snapshots")
        if not isinstance(snapshots, list):
            ok = False
        else:
            snapshot_ids = set()
            for snapshot in snapshots:
                if not isinstance(snapshot, dict):
                    ok = False
                    continue
                snapshot_id = snapshot.get("id")
                snapshot_path = snapshot.get("path")
                if not isinstance(snapshot_id, str) or snapshot_id.strip() == "":
                    ok = False
                elif snapshot_id in snapshot_ids:
                    ok = False
                else:
                    snapshot_ids.add(snapshot_id)
                if not isinstance(snapshot_path, str) or snapshot_path.strip() == "":
                    ok = False
                elif expect_valid and not (FIXTURES / snapshot_path).exists():
                    ok = False
    if expect_valid and not ok:
        fail("asset-bundle.valid.json did not satisfy the validator")
    if not expect_valid and ok:
        fail("asset-bundle.invalid.json unexpectedly passed validation")

def main() -> None:
    validate_program_fixture(load("program.valid.json"), expect_valid=True)
    validate_program_fixture(load("program.invalid.json"), expect_valid=False)
    validate_snapshot_fixture(load("snapshot.valid.json"), expect_valid=True)
    validate_snapshot_fixture(load("snapshot.invalid.json"), expect_valid=False)
    validate_asset_bundle_fixture(load("asset-bundle.valid.json"), expect_valid=True)
    validate_asset_bundle_fixture(load("asset-bundle.invalid.json"), expect_valid=False)
    print("[contract-validation] all switchyard fixtures validated")

if __name__ == "__main__":
    main()
