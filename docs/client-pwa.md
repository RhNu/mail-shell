# Client PWA Design

## Summary

`client/` already uses `vite-plugin-pwa` with `registerType: 'autoUpdate'`, but it does not expose update state to the user and it does not provide a global connectivity signal.

This document defines a minimal PWA-related UX layer for the SolidJS client:

- show a modal when a new service worker version is ready
- let the user choose between updating now or deferring for the current tab session
- show a global warning banner when the browser is offline or the API is unreachable

The design keeps update state and connectivity state separate, because they represent different failure modes and need different UI.

## Goals

- Add a user-visible update flow on top of the existing service worker registration.
- Keep update handling global instead of page-specific.
- Warn when the app cannot reach the backend, including both browser-offline and API-unreachable cases.
- Preserve the existing page-level error handling for query-specific failures.

## Non-Goals

- Do not claim the app is fully offline-capable.
- Do not add an offline-ready success toast.
- Do not replace page-level query error banners with the global connectivity banner.
- Do not add persistent "ignore this update forever" storage.

## Current State

`client/vite.config.ts` already enables `vite-plugin-pwa` with:

- `registerType: 'autoUpdate'`
- `injectRegister: 'auto'`

`client/src/main.tsx` does not currently register any explicit UI around service worker lifecycle events.

`client/src/app/app-shell.tsx` is the correct global mount point for cross-route UI because all screens render inside it.

The backend already exposes `/api/healthz`, and the client has a `useHealthQuery()` helper that can be reused or adapted for global connectivity checks.

## Proposed Architecture

Add a small global system-status layer inside `AppShell`. It will render two separate concerns:

1. `PwaUpdateController`
2. `ConnectivityBanner`

Suggested file layout:

- `client/src/features/system/pwa-update-controller.tsx`
- `client/src/features/system/use-connectivity-status.ts`
- `client/src/components/pwa-update-modal.tsx`
- `client/src/components/connectivity-banner.tsx`

`AppShell` will mount the controller and banner near the top-level layout so every route sees the same global behavior.

## Update Flow

`PwaUpdateController` will use `useRegisterSW` from `virtual:pwa-register/solid`.

Behavior:

- When `needRefresh()` becomes `true`, open a modal.
- Ignore `offlineReady()` for user-facing UI because the app is not meaningfully usable offline.
- If the user chooses `Update now`, call `updateServiceWorker(true)`.
- If the user chooses `Later`, suppress the modal for the remainder of the current page session.

Session suppression should be in-memory only:

- no `localStorage`
- no IndexedDB
- no cookie

That keeps the behavior simple:

- the current tab stops prompting after `Later`
- a future tab or future visit may prompt again if an unapplied update is still pending

## Connectivity Flow

The connectivity warning is a global banner, not a modal.

It has two trigger classes:

1. Browser offline
2. API unreachable while browser is still online

### Browser Offline

Use `navigator.onLine` plus `online` / `offline` window events.

When offline:

- show a warning banner immediately
- use wording that clearly points to the network being disconnected

### API Unreachable

Use a dedicated lightweight health check against `/api/healthz`.

Recommended polling policy:

- poll every 30 seconds during healthy operation
- after a failure, retry every 10 seconds until recovery
- require 2 consecutive failures before promoting to global "service unreachable"

This avoids treating a single transient timeout as a global outage and avoids coupling the banner to arbitrary screen-level query failures.

When health checks recover:

- clear the failure streak
- remove the banner automatically

## UI Design

### Update Modal

Purpose: user decision for applying a newly available version.

Content:

- title: `发现新版本`
- body: `有新内容可用。立即更新将刷新当前页面。`
- primary action: `立即更新`
- secondary action: `稍后`

Behavior:

- modal opens only when a new version is ready
- `稍后` dismisses it for the current tab session only
- `立即更新` refreshes through the service worker update path

### Connectivity Banner

Purpose: passive but persistent visibility into the app's inability to reach the backend.

Offline copy:

- `网络已断开，应用当前无法连接服务器。`

Service-unreachable copy:

- `暂时无法连接 mail-shell 服务，正在重试。`

Behavior:

- banner has no manual dismiss action
- banner disappears automatically after recovery
- page-level error banners remain visible where queries fail and provide local retry affordances

## Integration Points

### `client/src/app/app-shell.tsx`

Add:

- `PwaUpdateController` near the shell root
- `ConnectivityBanner` above the main routed content

This ensures:

- one instance only
- consistent behavior across route changes
- no route component needs to understand service worker lifecycle details

### `client/src/features/health/queries.ts`

The existing health query helper can either be reused directly or wrapped by `use-connectivity-status.ts`.

The preferred direction is to keep the connectivity-specific polling and failure-threshold logic inside `use-connectivity-status.ts` so health-check UX policy does not leak into general feature queries.

## Error Handling

Keep responsibilities separate:

- global connectivity banner: broad system reachability signal
- page `ErrorBanner`: local fetch or rendering failure

Examples:

- a malformed response on one feature endpoint should not necessarily trigger the global connectivity banner
- browser offline should trigger the global banner even before any page query runs
- `/api/healthz` repeatedly failing should trigger the global banner even if the current route has not yet performed a new query

## Testing Strategy

Add focused client tests around the new behavior.

### Connectivity Tests

Test `use-connectivity-status.ts` for:

- offline state when `navigator.onLine` is `false`
- service-unreachable state after 2 consecutive health-check failures
- recovery back to healthy state after a successful health check

### Update Tests

Test the update controller / modal for:

- modal visible when `needRefresh()` is `true`
- modal hidden after `Later` for the current render session
- `updateServiceWorker(true)` called when `Update now` is clicked

## Open Implementation Notes

- Reuse existing visual language from `client/src/components/ui/error-banner.tsx` where practical, but do not force the update modal into an error style.
- Keep the connectivity banner visually distinct from destructive errors; it is a warning state, not data loss confirmation.
- Avoid adding a larger global state store unless follow-up requirements justify it.
