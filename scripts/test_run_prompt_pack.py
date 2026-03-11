import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import run_prompt_pack


class PromptPackTests(unittest.TestCase):
    def test_select_next_item_returns_first_ready_non_done_item(self):
        text = """version: 1
items:
  - id: first
    status: done
    depends_on: []
  - id: second
    status: in_progress
    depends_on: [first]
  - id: third
    status: todo
    depends_on: [second]
"""
        items = run_prompt_pack.parse_taskboard(text)

        self.assertEqual(run_prompt_pack.select_next_item(items), "second")

    def test_build_runpack_for_trace_item_uses_api_validation_and_docs_prompts(self):
        plan = run_prompt_pack.build_runpack("switchyard.trace-and-debug-hooks")

        self.assertEqual(plan[0], "codex/00-OVERNIGHT-RUNBOOK.md")
        self.assertIn("codex/prompts/00-LAUNCH-THIS-REPO.md", plan)
        self.assertIn("codex/prompts/04-APIS-OR-PLUGIN-LAYER.md", plan)
        self.assertIn("codex/prompts/05-TESTS-AND-VALIDATION.md", plan)
        self.assertEqual(plan[-1], "codex/prompts/07-DOCS-FINAL-AUDIT.md")


if __name__ == "__main__":
    unittest.main()
