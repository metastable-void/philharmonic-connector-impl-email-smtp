//! SMTP email submission implementation for Philharmonic connectors.
//!
//! `email_smtp` implements the shared
//! [`philharmonic_connector_impl_api::Implementation`] trait for the
//! normalized SMTP submission wire protocol described in the workspace
//! connector architecture docs. It receives decrypted endpoint config,
//! validates submission-server policy, fixes only the minimal MIME envelope
//! headers that submission servers commonly require, and sends the message
//! via `lettre` over rustls.

mod config;
mod connect;
mod error;
mod mime;
mod request;
mod response;

pub use crate::config::{
    ConnectionMode, DeliveryAttempt, EmailSmtpConfig, PreparedConfig, TlsChannel, TlsPlan,
    TlsStrictness, TransportMode,
};
#[doc(hidden)]
pub use crate::mime::normalize_mime_envelope_with;
pub use crate::mime::{NormalizedMime, normalize_mime_envelope};
pub use crate::request::{EmailSmtpRequest, PreparedRequest};
pub use crate::response::EmailSmtpResponse;
pub use philharmonic_connector_impl_api::{
    ConnectorCallContext, Implementation, ImplementationError, JsonValue, async_trait,
};

const NAME: &str = "email_smtp";

/// `email_smtp` connector implementation.
#[derive(Clone, Debug, Default)]
pub struct EmailSmtp;

impl EmailSmtp {
    /// Builds an SMTP connector implementation.
    pub fn new() -> Result<Self, ImplementationError> {
        Ok(Self)
    }
}

#[async_trait]
impl Implementation for EmailSmtp {
    fn name(&self) -> &str {
        NAME
    }

    async fn execute(
        &self,
        config: &JsonValue,
        request: &JsonValue,
        _ctx: &ConnectorCallContext,
    ) -> Result<JsonValue, ImplementationError> {
        let config: EmailSmtpConfig = serde_json::from_value(config.clone())
            .map_err(|e| error::Error::InvalidConfig(e.to_string()))
            .map_err(ImplementationError::from)?;
        let config = config.prepare_inner().map_err(ImplementationError::from)?;

        let request: EmailSmtpRequest = serde_json::from_value(request.clone())
            .map_err(|e| error::Error::InvalidRequest(e.to_string()))
            .map_err(ImplementationError::from)?;
        let request = request.prepare_inner().map_err(ImplementationError::from)?;

        let response = connect::submit(&config, &request)
            .await
            .map_err(ImplementationError::from)?;

        serde_json::to_value(response)
            .map_err(|e| error::Error::Internal(e.to_string()))
            .map_err(ImplementationError::from)
    }
}
