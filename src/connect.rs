use std::time::Duration;

use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{CertificateStore, Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::config::{DeliveryAttempt, PreparedConfig, TlsChannel, TlsStrictness};
use crate::error::{Error, Result};
use crate::request::PreparedRequest;
use crate::response::EmailSmtpResponse;

const SMTP_COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) async fn submit(
    config: &PreparedConfig,
    request: &PreparedRequest,
) -> Result<EmailSmtpResponse> {
    let envelope = request.envelope()?;
    let mut last_error = None;

    for (index, attempt) in config.attempts().iter().copied().enumerate() {
        let transport = build_transport(config, attempt)?;
        match transport.send_raw(&envelope, request.body.as_bytes()).await {
            Ok(_) => return Ok(EmailSmtpResponse::accepted(request.message_id.clone())),
            Err(error) => {
                let mapped = map_smtp_error(error);
                let can_try_next = config.is_auto_discovery()
                    && index + 1 < config.attempts().len()
                    && matches!(mapped, Error::UpstreamUnreachable(_));
                if can_try_next {
                    last_error = Some(mapped);
                    continue;
                }
                return Err(mapped);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        Error::Internal("SMTP auto-discovery produced no delivery attempts".to_owned())
    }))
}

fn build_transport(
    config: &PreparedConfig,
    attempt: DeliveryAttempt,
) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let credentials = Credentials::new(config.username.clone(), config.password.clone());
    let tls = build_tls(&config.host, attempt)?;
    Ok(
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            .port(attempt.port)
            .timeout(Some(SMTP_COMMAND_TIMEOUT))
            .tls(tls)
            .credentials(credentials)
            .build(),
    )
}

fn build_tls(host: &str, attempt: DeliveryAttempt) -> Result<Tls> {
    let parameters = build_tls_parameters(host, attempt.tls_strictness)?;
    Ok(match attempt.tls_plan().channel {
        TlsChannel::StarttlsRequired => Tls::Required(parameters),
        TlsChannel::StarttlsOpportunistic => Tls::Opportunistic(parameters),
        TlsChannel::SmtpsWrapper => Tls::Wrapper(parameters),
    })
}

fn build_tls_parameters(host: &str, strictness: TlsStrictness) -> Result<TlsParameters> {
    let mut builder =
        TlsParameters::builder(host.to_owned()).certificate_store(CertificateStore::WebpkiRoots);
    if strictness.skips_server_identity() {
        builder = builder
            .dangerous_accept_invalid_certs(true)
            .dangerous_accept_invalid_hostnames(true);
    }
    builder
        .build_rustls()
        .map_err(|e| Error::Internal(format!("failed to build rustls SMTP TLS parameters: {e}")))
}

fn map_smtp_error(error: lettre::transport::smtp::Error) -> Error {
    if let Some(code) = error.status() {
        return Error::SmtpStatus {
            status: u16::from(code),
            body: error.to_string(),
        };
    }

    if error.is_tls() {
        return Error::UpstreamUnreachable(format!("SMTP TLS handshake failed: {error}"));
    }

    if error.is_client() || error.is_response() || error.is_transport_shutdown() {
        return Error::Internal(error.to_string());
    }

    Error::UpstreamUnreachable(format!("SMTP connection failed: {error}"))
}
