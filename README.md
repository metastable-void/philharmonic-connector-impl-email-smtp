# philharmonic-connector-impl-email-smtp

SMTP email submission connector implementation for the Philharmonic
workflow orchestration system.

## Status

Implemented. `email_smtp` submits caller-supplied MIME messages
through configured SMTP submission servers in the `email` realm.
Endpoint config carries `host`, optional `port`,
`connection_mode`, required `username` / `password`, and
`tls_strictness`; request JSON carries `{mail_from, recipients,
body}`. The implementation uses `lettre` over rustls with
aws-lc-rs and webpki-roots, rejects port 25, and adds only the
minimal MIME envelope headers required for reliable submission.

## License

Dual-licensed under `Apache-2.0 OR MPL-2.0`.

## Contributing

Developed as part of the
[Philharmonic workspace](https://github.com/metastable-void/philharmonic-workspace).
See
[`CONTRIBUTING.md`](https://github.com/metastable-void/philharmonic-workspace/blob/main/CONTRIBUTING.md).
