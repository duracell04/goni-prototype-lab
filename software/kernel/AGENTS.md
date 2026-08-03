# AGENTS.md (`software/kernel`) - overrides

## Overrides
- Kernel crates must respect plane boundaries (Data / Context / Control / Execution). Do not bypass contracts for convenience.
- GONI's canonical contracts live in `duracell04/goni`, pinned by
  `../../canonical-basis.json`; this lab must not rewrite them locally.
- Any change that impacts routing, scheduling, or tool execution must name the
  relevant canonical node ID in the change rationale:
  - routing/scheduling: `SCHED-01`
  - tool execution/permissions: `TOOL-01`
- Experimental findings may be proposed as pinned evidence to `goni`. They do
  not promote canonical status or redefine a canonical node.

## Status honesty
- If an invariant is described as implemented or tested, point to the exact
  code and test boundary. Otherwise use "specified only" or "planned".
