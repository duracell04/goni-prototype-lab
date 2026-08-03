# Quickstart

Status: the commands below are experimental entry points; their passing results
apply only to the named local checks.

Prerequisites for the composed demo: Docker and Docker Compose.

From the repository root:

1. Run `make canonical` to validate the canonical-basis manifest.
2. Run `make bench` for the synthetic lab benchmark.
3. Run `make smoke-local` to smoke-test `goni-http` with the local LLM stub.
4. Run `make demo` and `make smoke` for the composed path, including Qdrant.

Use `.env.example` for the composed demo's environment variables. These checks
do not validate the complete canonical blueprint.
