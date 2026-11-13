use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::runtime::config::MailSection;
use crate::runtime::error::AppError;

/// SMTP 邮件发送器；未配置 SMTP_HOST 时进入开发模式，验证码仅输出到日志。
pub struct Mailer {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: String,
}

impl Mailer {
    pub fn new(config: &MailSection) -> Self {
        if config.host.trim().is_empty() {
            tracing::warn!("SMTP_HOST 未配置，邮件进入开发模式（验证码输出到日志）");
            return Self {
                transport: None,
                from: config.from.clone(),
            };
        }

        let builder = match config.tls.as_str() {
            "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host),
            "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
                &config.host,
            )),
            _ => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host),
        };

        let transport = match builder {
            Ok(mut builder) => {
                builder = builder.port(config.port);
                if !config.username.is_empty() {
                    builder = builder.credentials(Credentials::new(
                        config.username.clone(),
                        config.password.clone(),
                    ));
                }
                Some(builder.build())
            }
            Err(error) => {
                tracing::error!(%error, "SMTP 传输初始化失败，邮件进入开发模式");
                None
            }
        };

        Self {
            transport,
            from: config.from.clone(),
        }
    }

    pub fn is_dev(&self) -> bool {
        self.transport.is_none()
    }

    pub async fn send_verification_code(
        &self,
        to: &str,
        code: &str,
        purpose_label: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let Some(transport) = &self.transport else {
            tracing::info!(to, code, purpose = purpose_label, "开发模式验证码");
            return Ok(());
        };

        let message = Message::builder()
            .from(
                self.from
                    .parse()
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("SMTP_FROM 格式错误: {e}")))?,
            )
            .to(
                to.parse()
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("收件邮箱格式错误: {e}")))?,
            )
            .subject(format!("NovelCast {purpose_label}验证码"))
            .header(ContentType::TEXT_PLAIN)
            .body(format!(
                "您的 NovelCast {purpose_label}验证码为：{code}\n\n验证码 {} 分钟内有效，请勿泄露给他人。如非本人操作，请忽略本邮件。",
                ttl_secs / 60
            ))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("邮件构建失败: {e}")))?;

        transport
            .send(message)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("邮件发送失败: {e}")))?;
        Ok(())
    }
}
