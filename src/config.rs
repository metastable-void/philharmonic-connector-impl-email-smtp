use serde::Deserialize;

use crate::error::{Error, Result};
use philharmonic_connector_impl_api::ImplementationError;

const SMTP_PORT_25: u16 = 25;
const SMTP_STARTTLS_PORT: u16 = 587;
const SMTP_SMTPS_PORT: u16 = 465;

/// Endpoint config for the `email_smtp` implementation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmailSmtpConfig {
    /// SMTP submission-server hostname.
    pub host: String,
    /// Optional SMTP port. Omitted only enables auto-discovery in `auto` mode.
    pub port: Option<u16>,
    /// Submission transport selection.
    #[serde(default)]
    pub connection_mode: ConnectionMode,
    /// SMTP AUTH username.
    pub username: String,
    /// SMTP AUTH password.
    pub password: String,
    /// TLS verification posture.
    #[serde(default)]
    pub tls_strictness: TlsStrictness,
}

impl EmailSmtpConfig {
    /// Validate endpoint config and compute the per-call delivery attempts.
    pub fn prepare(self) -> std::result::Result<PreparedConfig, ImplementationError> {
        self.prepare_inner().map_err(ImplementationError::from)
    }

    pub(crate) fn prepare_inner(self) -> Result<PreparedConfig> {
        let host = non_empty(self.host, "host required")?;
        if matches!(self.port, Some(SMTP_PORT_25)) {
            return Err(Error::InvalidConfig(
                "port 25 is refused for SMTP submission".to_owned(),
            ));
        }

        let username = non_empty(self.username, "username required")?;
        let password = non_empty(self.password, "password required")?;
        let attempts = delivery_attempts(self.port, self.connection_mode, self.tls_strictness)?;

        Ok(PreparedConfig {
            host,
            username,
            password,
            attempts,
        })
    }
}

/// Validated SMTP endpoint config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedConfig {
    /// SMTP submission-server hostname.
    pub host: String,
    /// SMTP AUTH username.
    pub username: String,
    /// SMTP AUTH password.
    pub password: String,
    attempts: Vec<DeliveryAttempt>,
}

impl PreparedConfig {
    /// Ordered delivery attempts for a single `execute` call.
    pub fn attempts(&self) -> &[DeliveryAttempt] {
        &self.attempts
    }

    pub(crate) fn is_auto_discovery(&self) -> bool {
        self.attempts.len() > 1
    }
}

/// SMTP connection-mode knob from endpoint config.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// Force STARTTLS on the selected port.
    Starttls,
    /// Force implicit TLS on the selected port.
    Smtps,
    /// Infer mode from the port, or try 587 then 465 when no port is set.
    #[default]
    Auto,
}

/// TLS verification posture.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TlsStrictness {
    /// Require TLS with certificate and hostname verification.
    #[default]
    Strict,
    /// Require TLS while skipping server identity verification.
    Sloppy,
    /// Try TLS with full verification and fall back to plaintext on STARTTLS.
    Opportunistic,
    /// Try TLS while skipping identity verification and fall back to plaintext on STARTTLS.
    OpportunisticSloppy,
}

impl TlsStrictness {
    pub(crate) fn skips_server_identity(self) -> bool {
        matches!(self, Self::Sloppy | Self::OpportunisticSloppy)
    }
}

/// SMTP transport mode for one concrete attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportMode {
    /// STARTTLS over a plaintext SMTP connection.
    Starttls,
    /// Implicit TLS from connection start.
    Smtps,
}

/// One concrete host/port/transport attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryAttempt {
    /// SMTP port.
    pub port: u16,
    /// SMTP transport mode.
    pub transport_mode: TransportMode,
    /// TLS strictness for this attempt.
    pub tls_strictness: TlsStrictness,
}

impl DeliveryAttempt {
    /// TLS behavior selected for this concrete attempt.
    pub fn tls_plan(self) -> TlsPlan {
        let channel = match (self.transport_mode, self.tls_strictness) {
            (TransportMode::Starttls, TlsStrictness::Opportunistic)
            | (TransportMode::Starttls, TlsStrictness::OpportunisticSloppy) => {
                TlsChannel::StarttlsOpportunistic
            }
            (TransportMode::Starttls, _) => TlsChannel::StarttlsRequired,
            (TransportMode::Smtps, _) => TlsChannel::SmtpsWrapper,
        };
        TlsPlan {
            channel,
            skips_server_identity: self.tls_strictness.skips_server_identity(),
        }
    }
}

/// TLS channel behavior selected for a concrete attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlsChannel {
    /// STARTTLS must be negotiated before credentials or message data.
    StarttlsRequired,
    /// STARTTLS is attempted, with plaintext fallback if unavailable.
    StarttlsOpportunistic,
    /// TLS is active before SMTP protocol exchange.
    SmtpsWrapper,
}

/// Publicly inspectable TLS plan for tests and callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlsPlan {
    /// TLS channel behavior.
    pub channel: TlsChannel,
    /// Whether certificate and hostname verification are skipped.
    pub skips_server_identity: bool,
}

fn delivery_attempts(
    port: Option<u16>,
    mode: ConnectionMode,
    strictness: TlsStrictness,
) -> Result<Vec<DeliveryAttempt>> {
    match (mode, port) {
        (ConnectionMode::Auto, Some(port)) => Ok(vec![DeliveryAttempt {
            port,
            transport_mode: mode_for_port(port),
            tls_strictness: strictness,
        }]),
        (ConnectionMode::Auto, None) => Ok(vec![
            DeliveryAttempt {
                port: SMTP_STARTTLS_PORT,
                transport_mode: TransportMode::Starttls,
                tls_strictness: strictness,
            },
            DeliveryAttempt {
                port: SMTP_SMTPS_PORT,
                transport_mode: TransportMode::Smtps,
                tls_strictness: strictness,
            },
        ]),
        (ConnectionMode::Starttls, Some(port)) => Ok(vec![DeliveryAttempt {
            port,
            transport_mode: TransportMode::Starttls,
            tls_strictness: strictness,
        }]),
        (ConnectionMode::Smtps, Some(port)) => Ok(vec![DeliveryAttempt {
            port,
            transport_mode: TransportMode::Smtps,
            tls_strictness: strictness,
        }]),
        (ConnectionMode::Starttls, None) => Err(Error::InvalidConfig(
            "port required when connection_mode is starttls".to_owned(),
        )),
        (ConnectionMode::Smtps, None) => Err(Error::InvalidConfig(
            "port required when connection_mode is smtps".to_owned(),
        )),
    }
}

fn mode_for_port(port: u16) -> TransportMode {
    if port == SMTP_SMTPS_PORT {
        TransportMode::Smtps
    } else {
        TransportMode::Starttls
    }
}

fn non_empty(value: String, detail: &str) -> Result<String> {
    if value.trim().is_empty() {
        Err(Error::InvalidConfig(detail.to_owned()))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_without_port_discovers_starttls_then_smtps() {
        let attempts =
            delivery_attempts(None, ConnectionMode::Auto, TlsStrictness::Strict).expect("attempts");
        assert_eq!(
            attempts,
            vec![
                DeliveryAttempt {
                    port: 587,
                    transport_mode: TransportMode::Starttls,
                    tls_strictness: TlsStrictness::Strict
                },
                DeliveryAttempt {
                    port: 465,
                    transport_mode: TransportMode::Smtps,
                    tls_strictness: TlsStrictness::Strict
                }
            ]
        );
    }

    #[test]
    fn tls_plan_preserves_channel_and_identity_choices() {
        let strict = DeliveryAttempt {
            port: 587,
            transport_mode: TransportMode::Starttls,
            tls_strictness: TlsStrictness::Strict,
        };
        assert_eq!(
            strict.tls_plan(),
            TlsPlan {
                channel: TlsChannel::StarttlsRequired,
                skips_server_identity: false
            }
        );

        let sloppy = DeliveryAttempt {
            port: 587,
            transport_mode: TransportMode::Starttls,
            tls_strictness: TlsStrictness::Sloppy,
        };
        assert_eq!(
            sloppy.tls_plan(),
            TlsPlan {
                channel: TlsChannel::StarttlsRequired,
                skips_server_identity: true
            }
        );

        let opportunistic = DeliveryAttempt {
            port: 587,
            transport_mode: TransportMode::Starttls,
            tls_strictness: TlsStrictness::Opportunistic,
        };
        assert_eq!(
            opportunistic.tls_plan(),
            TlsPlan {
                channel: TlsChannel::StarttlsOpportunistic,
                skips_server_identity: false
            }
        );

        let opportunistic_sloppy = DeliveryAttempt {
            port: 465,
            transport_mode: TransportMode::Smtps,
            tls_strictness: TlsStrictness::OpportunisticSloppy,
        };
        assert_eq!(
            opportunistic_sloppy.tls_plan(),
            TlsPlan {
                channel: TlsChannel::SmtpsWrapper,
                skips_server_identity: true
            }
        );
    }
}
