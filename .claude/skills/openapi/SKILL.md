---
name: openapi
description: Apply when changing the REST API surface — actix handlers, utoipa annotations, or request/response types under server/.
---

# OpenAPI / REST schema

- Server: actix-web + utoipa, behind the `server` feature. `ApiDoc` in `src/server/run/mod.rs` (module `openapi`); Swagger UI via `utoipa-swagger-ui`.
- Spec is checked in at `openapi.schema.yaml` (repo root, ~296 KB). A CI **Schema** job regenerates it and `diff`s against the committed file (stripping the `version:` line), so it must stay in sync.

## Hard rule
After ANY API-surface change (routes, handlers, `#[utoipa::path]`, schema structs), regenerate and commit the schema in the same PR, or CI fails:

```
cargo run -- server schema --output-file openapi.schema.yaml
```

Because the file is large, expect big diffs — review them. The `version:` line doesn't matter (CI strips it).
