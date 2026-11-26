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
