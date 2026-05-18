# Changelog

All notable changes to this crate are documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No unreleased changes.

## [0.1.0] - 2026-05-18

- Added the first substantive `email_smtp` connector
  implementation.
- Added SMTP submission over `lettre` with rustls, aws-lc-rs,
  webpki-roots, required AUTH credentials, port-25 rejection,
  explicit connection-mode policy, and four-valued TLS
  strictness.
- Added minimal MIME envelope fixing and tests for the locked
  connection-mode, TLS-strictness, config-validation, and MIME
  normalization matrices.

## [0.0.1]

- Added crate-level doc comment.

## [0.0.0]

Name reservation on crates.io. No functional content yet.
