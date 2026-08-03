"""Validate this lab's machine-readable pointer to the canonical GONI corpus."""

import argparse
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "canonical-basis.json"
FULL_SHA = re.compile(r"^[0-9a-f]{40}$")
NODE_ID = re.compile(r"^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+$")


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)


def validate(require_final: bool = False) -> bool:
    try:
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {MANIFEST.relative_to(ROOT)}: {exc}")
        return False

    ok = True
    expected_keys = {
        "version",
        "canonical_repository",
        "baseline_revision",
        "final_revision",
        "pin_state",
        "replacement_policy",
        "relevant_node_ids",
        "evidence_policy",
    }
    if set(data) != expected_keys:
        fail(f"manifest keys must be exactly {sorted(expected_keys)}")
        ok = False

    if data.get("version") != 1:
        fail("version must be 1")
        ok = False
    if data.get("canonical_repository") != "https://github.com/duracell04/goni":
        fail("canonical_repository must identify duracell04/goni")
        ok = False

    baseline = data.get("baseline_revision")
    if not isinstance(baseline, str) or not FULL_SHA.fullmatch(baseline):
        fail("baseline_revision must be a full lowercase Git SHA")
        ok = False

    pin_state = data.get("pin_state")
    final = data.get("final_revision")
    if pin_state == "awaiting_final_migration_sha":
        if final is not None:
            fail("final_revision must be null while awaiting the final migration SHA")
            ok = False
        if require_final:
            fail("the final canonical revision is required for this validation mode")
            ok = False
    elif pin_state == "pinned":
        if not isinstance(final, str) or not FULL_SHA.fullmatch(final):
            fail("final_revision must be a full lowercase Git SHA when pinned")
            ok = False
    else:
        fail("pin_state must be awaiting_final_migration_sha or pinned")
        ok = False

    policy = data.get("replacement_policy")
    if not isinstance(policy, dict) or policy.get("placeholder", "missing") is not None:
        fail("replacement_policy.placeholder must be null")
        ok = False
    elif policy.get("required_revision_format") != "full_lowercase_git_sha":
        fail("replacement_policy must require a full lowercase Git SHA")
        ok = False

    node_ids = data.get("relevant_node_ids")
    if not isinstance(node_ids, list) or not node_ids:
        fail("relevant_node_ids must be a non-empty list")
        ok = False
    elif node_ids != sorted(set(node_ids)):
        fail("relevant_node_ids must be unique and sorted")
        ok = False
    else:
        for node_id in node_ids:
            if not isinstance(node_id, str) or not NODE_ID.fullmatch(node_id):
                fail(f"invalid canonical node ID: {node_id!r}")
                ok = False

    evidence = data.get("evidence_policy")
    required_true = {
        "submission_requires_full_commit_sha",
        "submission_requires_test_boundary",
        "submission_requires_limitations",
    }
    required_false = {
        "may_promote_canonical_status",
        "may_redefine_canonical_nodes",
    }
    if not isinstance(evidence, dict):
        fail("evidence_policy must be an object")
        ok = False
    else:
        for field in sorted(required_true):
            if evidence.get(field) is not True:
                fail(f"evidence_policy.{field} must be true")
                ok = False
        for field in sorted(required_false):
            if evidence.get(field) is not False:
                fail(f"evidence_policy.{field} must be false")
                ok = False

    return ok


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--require-final",
        action="store_true",
        help="fail unless final_revision is a full SHA and pin_state is pinned",
    )
    args = parser.parse_args()
    if not validate(require_final=args.require_final):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
