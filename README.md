# philharmonic-connector-impl-email-smtp

SMTP email-sending connector implementation for the Philharmonic
workflow orchestration system.

## Status

**Not yet implemented.** This crate is reserved as part of the
[Philharmonic](https://github.com/metastable-void/philharmonic-workspace)
crate family and will provide email sending via SMTP as a
connector implementation, using
[`lettre`](https://crates.io/crates/lettre) for the transport
layer.

Workflow steps will be able to send emails (plain text and HTML)
through a configured SMTP submission server, with per-step
encrypted credentials managed by the connector layer's
COSE_Encrypt0 payload channel.

Implementation is scheduled for Phase 7 Tier 2, after the
Tier 1 data-layer connectors shipped (2026-04-27). See
[`ROADMAP.md` §Phase 7 Tier 2](https://github.com/metastable-void/philharmonic-workspace/blob/main/ROADMAP.md)
for the timeline.

## This is not name squatting

This name reservation is part of an active, shipped project.
The parent workspace has 21 published crates on crates.io,
a full API server, end-to-end integration tests, and a
working deployment pipeline. This crate is next in the
implementation queue — not a speculative hold.

If you believe this name conflicts with your project, please
open an issue at the
[workspace repository](https://github.com/metastable-void/philharmonic-workspace/issues).

## License

Dual-licensed under `Apache-2.0 OR MPL-2.0`.

## Contributing

Developed as part of the
[Philharmonic workspace](https://github.com/metastable-void/philharmonic-workspace).
See
[`CONTRIBUTING.md`](https://github.com/metastable-void/philharmonic-workspace/blob/main/CONTRIBUTING.md).
