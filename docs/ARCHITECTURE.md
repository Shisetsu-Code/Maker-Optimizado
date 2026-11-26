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

Reproducibility: Rust version and direct dependencies are exact. CI generates initial lockfiles if absent and exposes them as artifacts. Check in both before production. Dependabot proposes updates, not automatic deployments. No claim of zero defects or performance improvements without measurements.
