use serde::Serialize;

/// Success response for `email_smtp`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailSmtpResponse {
    /// Always true on successful SMTP acceptance.
    pub accepted: bool,
    /// Message-Id header value used for this submission.
    pub message_id: String,
}

impl EmailSmtpResponse {
    pub(crate) fn accepted(message_id: String) -> Self {
        Self {
            accepted: true,
            message_id,
        }
    }
}
