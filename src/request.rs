use lettre::{Address, address::Envelope};
use philharmonic_connector_impl_api::ImplementationError;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::mime;

/// Request body for the `email_smtp` implementation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmailSmtpRequest {
    /// Envelope sender used for SMTP `MAIL FROM`.
    pub mail_from: String,
    /// Envelope recipients used for SMTP `RCPT TO`.
    pub recipients: Vec<String>,
    /// Full MIME message.
    pub body: String,
}

impl EmailSmtpRequest {
    /// Validate the request and normalize its MIME envelope.
    pub fn prepare(self) -> std::result::Result<PreparedRequest, ImplementationError> {
        self.prepare_inner().map_err(ImplementationError::from)
    }

    pub(crate) fn prepare_inner(self) -> Result<PreparedRequest> {
        let mail_from = non_empty(self.mail_from, "mail_from required")?;
        if self.recipients.is_empty() {
            return Err(Error::InvalidRequest(
                "recipients must not be empty".to_owned(),
            ));
        }
        let recipients = self
            .recipients
            .into_iter()
            .map(|recipient| non_empty(recipient, "recipient must not be empty"))
            .collect::<Result<Vec<_>>>()?;

        if self.body.is_empty() {
            return Err(Error::InvalidRequest("body required".to_owned()));
        }

        let normalized = mime::normalize_mime_envelope(&self.body);

        Ok(PreparedRequest {
            mail_from,
            recipients,
            body: normalized.body,
            message_id: normalized.message_id,
        })
    }
}

/// Validated request ready for lettre submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedRequest {
    /// Envelope sender.
    pub mail_from: String,
    /// Envelope recipients.
    pub recipients: Vec<String>,
    /// CRLF-normalized MIME message body.
    pub body: String,
    /// Message-Id header value used for this submission.
    pub message_id: String,
}

impl PreparedRequest {
    pub(crate) fn envelope(&self) -> Result<Envelope> {
        let from = parse_address("mail_from", &self.mail_from)?;
        let to = self
            .recipients
            .iter()
            .map(|recipient| parse_address("recipient", recipient))
            .collect::<Result<Vec<_>>>()?;
        Envelope::new(Some(from), to)
            .map_err(|e| Error::InvalidRequest(format!("invalid SMTP envelope: {e}")))
    }
}

fn parse_address(label: &str, value: &str) -> Result<Address> {
    value
        .parse::<Address>()
        .map_err(|e| Error::InvalidRequest(format!("invalid {label} address: {e}")))
}

fn non_empty(value: String, detail: &str) -> Result<String> {
    if value.trim().is_empty() {
        Err(Error::InvalidRequest(detail.to_owned()))
    } else {
        Ok(value)
    }
}
