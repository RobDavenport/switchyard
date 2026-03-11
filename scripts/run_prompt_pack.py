#!/usr/bin/env python3
"""Generate the next prompt-pack loop from the current taskboard."""

from __future__ import annotations

import argparse
import pathlib
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class TaskboardItem:
    item_id: str
    status: str
    depends_on: tuple[str, ...]


PROMPT_MAP = {
    "switchyard.bootstrap": [
        "codex/prompts/01-REPO-AND-TOOLING.md",
        "codex/prompts/06-CI-LINT-AND-RELEASE.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.contracts": [
        "codex/prompts/02-CONTRACTS-AND-SCHEMAS.md",
        "codex/prompts/05-TESTS-AND-VALIDATION.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.scheduler-core": [
        "codex/prompts/03-CORE-DOMAIN.md",
        "codex/prompts/05-TESTS-AND-VALIDATION.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.persistence-and-inspection": [
        "codex/prompts/03-CORE-DOMAIN.md",
        "codex/prompts/05-TESTS-AND-VALIDATION.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.authoring-surface": [
        "codex/prompts/04-APIS-OR-PLUGIN-LAYER.md",
        "codex/prompts/05-TESTS-AND-VALIDATION.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.trace-and-debug-hooks": [
        "codex/prompts/04-APIS-OR-PLUGIN-LAYER.md",
        "codex/prompts/05-TESTS-AND-VALIDATION.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
    "switchyard.docs-and-release-hygiene": [
        "codex/prompts/01-REPO-AND-TOOLING.md",
        "codex/prompts/06-CI-LINT-AND-RELEASE.md",
        "codex/prompts/07-DOCS-FINAL-AUDIT.md",
    ],
}

BASELINE_COMMANDS = (
    "cargo fmt --all -- --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo test --workspace --all-features",
    "cargo check --workspace --lib --no-default-features",
    f'"{sys.executable}" scripts/validate_contract_fixtures.py',
)


def parse_taskboard(text: str) -> list[TaskboardItem]:
    items: list[TaskboardItem] = []
    current_id: str | None = None
    current_status: str | None = None
    current_depends: tuple[str, ...] = ()

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line == "items:" or line.startswith("version:"):
            continue
        if line.startswith("- id:"):
            if current_id is not None and current_status is not None:
                items.append(TaskboardItem(current_id, current_status, current_depends))
            current_id = line.split(":", 1)[1].strip()
            current_status = None
            current_depends = ()
            continue
        if line.startswith("status:"):
            current_status = line.split(":", 1)[1].strip()
            continue
        if line.startswith("depends_on:"):
            raw = line.split(":", 1)[1].strip()
            if raw == "[]":
                current_depends = ()
            else:
                current_depends = tuple(
                    part.strip() for part in raw.strip("[]").split(",") if part.strip()
                )

    if current_id is not None and current_status is not None:
        items.append(TaskboardItem(current_id, current_status, current_depends))

    return items


def select_next_item(items: list[TaskboardItem]) -> str | None:
    completed = {item.item_id for item in items if item.status == "done"}
    for item in items:
        if item.status == "done":
            continue
        if all(dep in completed for dep in item.depends_on):
            return item.item_id
    return None


def build_runpack(item_id: str | None) -> list[str]:
    runpack = ["codex/00-OVERNIGHT-RUNBOOK.md", "codex/prompts/00-LAUNCH-THIS-REPO.md"]
    if item_id is None:
        runpack.extend(
            [
                "codex/prompts/06-CI-LINT-AND-RELEASE.md",
                "codex/prompts/07-DOCS-FINAL-AUDIT.md",
            ]
        )
        return runpack

    runpack.extend(PROMPT_MAP.get(item_id, ["codex/prompts/05-TESTS-AND-VALIDATION.md"]))
    return runpack


def render_loop(item_id: str | None) -> str:
    prompts = build_runpack(item_id)
    lines = ["Switchyard Prompt-Pack Loop", ""]
    if item_id is None:
        lines.append("Current focus: all taskboard items are done; run final validation and docs audit.")
    else:
        lines.append(f"Current focus: {item_id}")
    lines.append("")
    lines.append("Prompt sequence:")
    lines.extend(f"- {prompt}" for prompt in prompts)
    lines.append("")
    lines.append("Validation loop:")
    lines.extend(f"- {command}" for command in BASELINE_COMMANDS)
    lines.append("")
    lines.append("Repeat: after each completed slice, update codex/taskboard.yaml and rerun this script.")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--taskboard",
        default="codex/taskboard.yaml",
        help="Path to the taskboard file.",
    )
    args = parser.parse_args()

    taskboard_path = pathlib.Path(args.taskboard)
    items = parse_taskboard(taskboard_path.read_text(encoding="utf-8"))
    next_item = select_next_item(items)
    print(render_loop(next_item))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
