# Warp Localization

This crate owns the shared locale model, catalog loading, and template placeholder validation used by Warp clients and CLI surfaces.

## Catalogs

- Bundled app catalogs live in `app/assets/bundled/locales/`.
- `en-US.json` is the default fallback catalog and must contain every shipped key.
- `zh-CN.json` must keep the same keys and placeholder names as `en-US.json`.
- User-visible app strings should be looked up through `app/src/localization.rs` helpers instead of embedding English literals in UI builders.

## Regression Coverage

Run the focused localization checks with:

```bash
cargo test -p warp_localization
```

The test suite validates catalog parity, placeholder parity, schema translation keys, and selected high-risk UI call sites that previously regressed.
