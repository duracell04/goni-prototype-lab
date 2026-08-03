# Repository reality map

## Authority

`duracell04/goni` is the canonical source for GONI propositions and status.
This repository is an experimental lab. Its canonical baseline, final pin, and
relevant node IDs are recorded in `canonical-basis.json`.

Experiments can produce repository-and-commit-pinned observations for review in
`goni`. They cannot promote a node's status or redefine the canon.

## Status vocabulary

- **Specified only:** canonical design intent exists; this lab has no matching
  implementation evidence.
- **Implemented, untested here:** code exists, but no named passing test in this
  repository supports the stated behavior.
- **Implemented and tested within boundary:** a named test supports only the
  stated component behavior, not the complete canonical proposition.
- **Planned:** no testable implementation is present.

## Demonstrated component boundaries

| Component boundary | Lab status | Local evidence | Canonical node |
|---|---|---|---|
| Context selection stays within its budget and is deterministic for its fixture | Implemented and tested within boundary | `selector_respects_budget_and_is_deterministic` in `software/kernel/goni-context/src/lib.rs` | `ADR-INDEX-SW` |
| TXT schema guard rejects the tested disallowed construction | Implemented and tested within boundary | `software/kernel/goni-schema/tests/txt_axiom.rs` | `PRIV-01` |
| Agent manifest parser accepts the two tested manifest forms | Implemented and tested within boundary | parser tests in `software/kernel/goni-agent/src/lib.rs` | `AGENT-MANIFEST-01` |
| Scheduler applies the tested class preference and background limit | Implemented and tested within boundary | tests in `software/kernel/goni-sched/src/lib.rs` | `SCHED-01` |
| Receipt hash chain verifies the covered fixture | Implemented and tested within boundary | `receipt_chain_verifies` in `software/kernel/goni-receipts/src/lib.rs` | `REC-01` |
| Policy helpers return the covered allow/deny decisions | Implemented and tested within boundary | unit tests in `software/kernel/goni-policy/src/lib.rs` | `TOOL-01`, `LSC-01`, `NET-01` |
| HTTP completion route with local LLM stub | Implemented and smoke-tested within boundary | `scripts/run_smoke_local.sh` | `CI-VALIDATION-01` |

## Present but not established by tests

- `software/kernel/goni-egress-gate/src/main.rs` contains an HTTP allowlist
  proxy, but the repository does not demonstrate container-level non-bypass.
- `software/kernel/goni-router/src/lib.rs` contains a prototype router, but the
  repository does not demonstrate a regret bound.
- `software/kernel/goni-context/src/lib.rs` contains Arrow conversion code, but
  the repository does not demonstrate the complete zero-copy objective.
- Memory-write and redaction policy helpers are not demonstrated as wired into
  every runtime path.
- The frontend is a parked, non-buildable stub.
- Gateway behavior, a full UI, ITCR, and the complete Delegation OS remain
  specified only or planned here.

## CI boundary

`.github/workflows/ci.yml` runs for pushes, pull requests, and manual dispatch.
It validates the canonical-basis manifest, compiles Python entry points, runs
the RAG-slice unit tests and a synthetic benchmark smoke test, runs the
text-confinement lint, checks, tests, and runs Clippy on the Rust workspace,
then smoke-tests `goni-http` with its local LLM stub. These checks establish
only their named boundaries.

## Known gaps

- The local smoke test intentionally skips Qdrant health because it tests the
  orchestrator-plus-stub boundary; the composed smoke path still requires
  Qdrant.
