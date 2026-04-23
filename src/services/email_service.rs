use crate::config::auth::SmtpConfig;
use anyhow::Result;
use lettre::{
    message::{header::ContentType, Mailbox, SinglePart},
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

        let html_body = format!(
            r#"<!doctype html>
<html lang="zh-CN">
<body style="margin:0;background:#f6f7fb;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;color:#111827;">
  <div style="max-width:520px;margin:0 auto;padding:32px 20px;">
    <div style="background:#ffffff;border:1px solid #e5e7eb;border-radius:12px;padding:28px;">
      <h1 style="margin:0 0 12px;font-size:20px;line-height:28px;">{}验证码</h1>
      <p style="margin:0 0 20px;font-size:14px;line-height:22px;color:#4b5563;">请使用下面的 6 位数字完成{}验证：</p>
      <div style="letter-spacing:8px;font-size:32px;font-weight:700;line-height:44px;text-align:center;background:#f3f4f6;border-radius:10px;padding:14px 8px;color:#111827;">{}</div>
      <p style="margin:20px 0 0;font-size:13px;line-height:20px;color:#6b7280;">验证码 {} 秒内有效。若不是你本人操作，请忽略本邮件。</p>
    </div>
  </div>
</body>
</html>"#,
            purpose_label, purpose_label, code, ttl_seconds
        );

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(format!("{} {}验证码", subject_prefix, purpose_label))
            .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html_body))
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
