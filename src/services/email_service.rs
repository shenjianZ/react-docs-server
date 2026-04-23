use crate::config::auth::SmtpConfig;
use anyhow::Result;
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

#[derive(Clone)]
pub struct EmailService {
    config: SmtpConfig,
}

impl EmailService {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    pub async fn send_verification_code(
        &self,
        to_email: &str,
        code: &str,
        purpose_label: &str,
        ttl_seconds: u64,
        subject_prefix: &str,
    ) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("邮件服务未启用，请先配置 SMTP"));
        }

        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|e| anyhow::anyhow!("解析发件人失败: {}", e))?;
        let to: Mailbox = to_email
            .parse()
            .map_err(|e| anyhow::anyhow!("解析收件人失败: {}", e))?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(format!("{} {}验证码", subject_prefix, purpose_label))
            .body(format!(
                "您的 {} 验证码为：{}\n\n验证码 {} 秒内有效，请勿泄露给他人。",
                purpose_label, code, ttl_seconds
            ))
            .map_err(|e| anyhow::anyhow!("构建邮件失败: {}", e))?;

        let mut builder = if self.config.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.host)
                .map_err(|e| anyhow::anyhow!("初始化 SMTP 失败: {}", e))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
                .map_err(|e| anyhow::anyhow!("初始化 SMTP 失败: {}", e))?
        };

        builder = builder.port(self.config.port);
        if !self.config.username.is_empty() {
            builder = builder.credentials(Credentials::new(
                self.config.username.clone(),
                self.config.password.clone(),
            ));
        }

        builder
            .build()
            .send(email)
            .await
            .map_err(|e| anyhow::anyhow!("发送邮件失败: {}", e))?;

        Ok(())
    }
}
