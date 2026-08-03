# GONI prototype lab

This repository contains runnable experiments, scripts, and configurations that
test bounded parts of the GONI blueprint. It is not a source of canonical GONI
architecture or governance.

The canonical knowledge repository is
[`duracell04/goni`](https://github.com/duracell04/goni). The machine-readable
[`canonical-basis.json`](canonical-basis.json) records the pre-migration
baseline, the final migration pin, and the canonical node IDs relevant
to the experiments in this repository.

## Authority boundary

- Canonical propositions, status, relations, and definitions live in `goni`.
- This lab may implement or test a bounded interpretation of a canonical node.
- A result may be submitted to `goni` as evidence only when it pins this
  repository and commit, names the tested boundary, and states limitations.
- A lab result cannot promote a canonical node's status or redefine the canon.
- Passing tests support only the behavior named by those tests; they do not
  prove the complete blueprint or a non-bypassable Delegation OS.

## Local checks

```bash
python scripts/validate_canonical_basis.py
python goni-lab/goni_lab.py bench --scenario goni-lab/scenarios/mixed.json
bash scripts/txt_lint.sh
cargo test --manifest-path software/kernel/Cargo.toml --workspace --all-features
```

The final canonical revision is pinned in `canonical-basis.json`. Verify that
the manifest remains fully pinned with:

```bash
python scripts/validate_canonical_basis.py --require-final
```
