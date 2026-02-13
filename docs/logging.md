# Logging Guidelines

## Goals

- Keep logs searchable and consistent across frontend and backend.
- Preserve high-signal operational events.
- Remove noisy logs that do not help debugging or incident response.

## Frontend Format

- Use `createLogger(scope)` from `src/lib/utils/logger.ts`.
- Unified format: `[MUA][LEVEL][Scope] message`.
- Attach structured metadata as object context instead of string concatenation.

## Backend Format

- Use Rust `log` macros with a unified prefix style in message bodies.
- Recommended message shape: `[Core::Area] event details`.

## Level Policy

- `error`: user-visible failures, command failures, startup failures, sidecar failures.
- `warn`: recoverable issues, degraded mode, fallback path used.
- `info`: lifecycle milestones and major state transitions.
- `debug`: high-frequency or verbose diagnostics gated by debug log level.

## Keep / Remove Rules

- Keep logs that include operation name, identifier, and failure cause.
- Remove logs that only restate obvious UI actions.
- Avoid bare `console.error(e)` without context.
- Avoid `console.log` in production flows unless intentionally debug-only.

## Migration Priority

1. App boot and sidecar integration.
2. Task sync and task operations.
3. Torrent import/confirm flows.
4. Settings read/write flows.
