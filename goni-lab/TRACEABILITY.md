# Canonical node to experiment traceability

Resolve every canonical node against the full `goni` revision in
`canonical-basis.json`. Until `pin_state` becomes `pinned`, the IDs below refer
to the recorded pre-migration baseline and are awaiting final-path resolution.

| Canonical node | Lab interpretation | Implementation | Test boundary | Lab status / limitation |
|---|---|---|---|---|
| `PRIV-01` | TXT schema guard | `software/kernel/goni-schema/src/macros.rs` | `software/kernel/goni-schema/tests/txt_axiom.rs` | Tested construction only; no runtime-wide confinement claim |
| `SCHEMA-DSL-01` | Schema DSL/macros | `software/kernel/goni-schema/src/macros.rs` | None | Implemented, untested here |
| `AGENT-MANIFEST-01` | Agent manifest parsing | `software/kernel/goni-agent/src/lib.rs` | two parser unit tests in the same file | Parser forms only; no execution enforcement claim |
| `ADR-INDEX-SW` | Deterministic context selection interpretation | `software/kernel/goni-context/src/lib.rs` | `selector_respects_budget_and_is_deterministic` | One algorithm/fixture boundary; no approximation guarantee |
| `ZCO-01` | Arrow candidate conversion | `software/kernel/goni-context/src/lib.rs` | None | No complete zero-copy evidence |
| `SPINE-01` | Prototype spine identifiers | `software/kernel/goni-schema/src/macros.rs` | None | Implemented, untested here |
| `SCHED-01` | Class preference and background admission limit | `software/kernel/goni-sched/src/lib.rs` | scheduler unit tests in the same file | Simple ordering/limit only; no stability claim |
| `TOOL-01` | Capability types and policy helpers | `software/kernel/goni-cap/src/lib.rs`; `software/kernel/goni-policy/src/lib.rs`; `software/kernel/goni-tools/src/lib.rs` | policy helper unit tests | Helpers are not evidence of universal mediation |
| `REC-01` | JSONL receipt hash chain | `software/kernel/goni-receipts/src/lib.rs` | `receipt_chain_verifies` | Covered chain behavior only |
| `LSC-01` | Memory-write policy helper | `software/kernel/goni-policy/src/lib.rs` | `memory_write_requires_evidence` | Helper is not wired into every runtime path |
| `NET-01` | Redaction helper and egress-gate prototype | `software/kernel/goni-policy/src/lib.rs`; `software/kernel/goni-egress-gate/src/main.rs` | redaction helper unit test only | No container-level non-bypass evidence |
| `ITCR-01` | Inference-time compute controls | None identified | None | Specified only / planned |
| `SS-01`, `AXIOMS-01` | Symbolic substrate/axioms | Partial schema types | TXT construction test only | No complete symbolic-substrate evidence |

Evidence proposed back to `goni` must pin this repository's full commit SHA,
identify the canonical node, state the exact tested boundary and observed
result, and preserve limitations. Submission does not change canonical status.
