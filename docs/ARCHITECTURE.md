# Architecture

- model.rs: strict schema, stable IDs, image/style/size limits, validation.
- presets.rs: 17 declarative themes and default component trees.
- editor.rs: atomic commands, per-template/per-screen state, bounded undo/redo.
- render.rs: escaped text and validated style values, shared standalone HTML renderer.
- browser.rs: small WASM interface with metadata-only inspection.
- edge.rs: Workers SDK and bounded API, assets binding.
- frontend/surface.css: original responsive styling, compiled into renderer.
- public/studio.js: browser-only DOM, sandbox preview, image compression, downloads.
- public/storage.js: IndexedDB revision compare-and-swap.
- public/studio.css: editor chrome.
- tests: native model, browser WASM, Workers HTTP, and actual browser.

No arbitrary HTML, JS, CSS or remote image URLs enter the model. Preview is an iframe with same-origin access for inspection but without script permissions. Standalone exports use a restrictive meta CSP. The remote API has no persistent storage. Images remain private to the browser. History shares raster strings via Rc; mutations are validated before committing.

Cloudflare serves static resources directly; the API is stateless and does not require D1, KV, R2, Durable Objects, Queues or paid add-ons. Future remote publishing should use authenticated metadata in D1 and hash-addressed R2 assets, optimistic concurrency, tenant isolation and explicit quotas. It is not part of this implementation.

Reproducibility: Rust version and direct dependencies are exact. Both lockfiles are committed from the first verification run. CI checks their presence and builds with --locked / npm ci. Dependabot proposes updates, not automatic deployments. No claim of zero defects or performance improvements without measurements.

## Graphic library

Raster media are byte-preserved under immutable Git-content-addressed IDs. Rust validates the ID shape and chooses default artwork from the compiled preset map. The browser loads collection indexes on demand and renders at most 24 thumbnails per page. The original source path map is documentation, not a served runtime dependency. Non-raster or unrecognized content is preserved outside Static Assets. HTML exports inline referenced library images with a bounded byte budget. The initial restore workflow imports only graphical data from a pinned source commit; normal CI remains read-only.

The media-integrity test hashes every restored object against its original Git blob ID, verifies provenance sizes, and validates all preset and collection references. Browser E2E decodes the actual artwork for each preset, selects media from the paginated library and validates a standalone export with network disabled. Exports modify only rendered image attributes and bound each streamed response before Base64 encoding. The one-time import workflow was removed after the successful restoration; the local reproducible importer remains documented.

## Catalog model

Static normalized catalogues contain the full recovered records where structured source data is available. Rust validates and queries immutable records; only the edited cards are materialized into the project document. Existing seed IDs are mapped without rewriting user changes. Catalog interaction state is session-local; the browser bridge creates only filter controls, attaches event handlers and inserts typed Rust-generated card markup. The complete catalogue is not cloned into undo history or IndexedDB.

Appearance is a shared, versioned workspace setting; all CSS is emitted from validated enums, bounds and colors. Original local fonts are served with their OFL notices. Standalone HTML exports embed the referenced fonts and licenses. Component styles remain sparse and typed; no arbitrary CSS is accepted.

The administrative preview is a separate typed Rust state machine (`demo_panel.rs`) shared across presets, with synthetic fixtures, bounded actions and escaped output. The browser adapter (`panel.js`) only attaches events and replaces Rust-rendered preview markup. No server, payment, authentication, real player accounts or game engine is connected. The maintenance switch affects only the local preview. The visual audit is isolated from deployment and uses the immutable original reference.
