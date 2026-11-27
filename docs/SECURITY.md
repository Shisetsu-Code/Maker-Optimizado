# Security boundary and verification

Maker Studio is a local-first design editor and static demonstration. No real
account, funds, wager, credentials, external game runtime or remote project
storage is implemented. Do not treat the simulated administration as a
production management system.

## Controls

- Rust forbids unsafe code in the project. The Worker is stateless and limits
  validation requests to 256 KiB; JSON is required for the validation endpoint.
- Typed schema, escaped user text, allowlisted style properties and enums.
- Images in projects must be PNG, JPEG or WebP data URLs with valid container
  headers, bounded byte length and dimensions (4096 px per side and 16 million
  pixels maximum). Uploads are preflighted before browser decoding (8 MiB,
  40 million pixels and 16,384 px per side); canvas re-encodes them at <=1600 px.
  Animated PNG/WebP containers are rejected for user uploads/imports. Header
  preflight is not a full codec validation or malware scan.
- Legacy project imports cannot fall back from invalid current-schema projects.
  Previous editor limits of 40 assets, 50 pages and 150 rules per page are
  enforced; unresolved report strings are bounded.
- Preview uses a sandbox without allow-scripts. Static editor, HTML preview
  and JSON API use explicit CSP, no-sniff and same-origin resource policies.
  IndexedDB stores projects locally, with compare-and-swap revisions.
- Workflow actions are pinned to the exact SHAs previously used by successful
  CI. Security workflow audits both committed dependency lockfiles using npm
  and RustSec cargo-audit 0.23.0; no advisory exclusions are configured.
- Native/WASM adversarial regression tests, HTTP security header tests and
  Chromium browser tests run in regular CI. Visual parity is separately
  measured and is not a security certification.

## Limitations

No penetration test of a deployed instance, independent full source review,
or assurance of zero vulnerabilities is implied. Real remote publishing would
require tenant authentication, authorization, quotas and storage isolation.
The reference branch/history remains separate from the clean runtime. Only
the local, authorized raster library is served; unknown catalogue thumbnails
remain missing until a trusted source is provided.

The serialized workspace must fit the same 24 MiB limit used on import,
including after editing or legacy migration. History is bounded by both 32
entries and 48 MiB of combined serialized snapshots (a conservative weight,
not a heap measurement). Legacy assets share validated raster buffers and
the cumulative image budget is checked before applying each replacement.

## Dependency audit (2026-11-26)

Initial Security verification run 45909759371: npm reported zero known
vulnerabilities; RustSec scanned 82 crates against 1,568 advisories and found
no vulnerability errors. It reported one informational unmaintained warning,
[RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543.html), for
chrono 0.4.45, transitively required by the current Cloudflare worker 0.9.0 SDK.
No patched chrono version is available. The warning is visible, not ignored;
track removal by the upstream SDK rather than fork the dependency. A green
dependency audit is not a penetration test or proof of future safety.

The offline thumbnail importer accepts explicitly authorized local files
and exact item IDs, validates SHA-256, bounds paths to a selected directory,
preflights the raster using the Rust engine, and re-encodes through isolated
Chromium with networking disabled. It cannot overwrite an existing thumbnail.
The import command defaults to dry-run; no API integration has been configured.
