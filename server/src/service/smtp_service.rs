use mail_send::{SmtpClientBuilder, mail_builder::MessageBuilder};

use crate::{
    config::AppConfig,
    error::app_error::{AppError, AppResult},
};

#[derive(Clone, Debug)]
pub struct EmailMessage<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub text_body: Option<&'a str>,
    pub html_body: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct SMTPService {
    from: String,
    host: String,
    port: u16,
    login: String,
    password: String,
}

impl SMTPService {
    pub fn new(config: &AppConfig) -> Self {
        SMTPService {
            from: config.smtp_config.from.clone(),
            host: config.smtp_config.host.clone(),
            port: config.smtp_config.port,
            login: config.smtp_config.login.clone(),
            password: config.smtp_config.password.clone(),
        }
    }

    async fn send_message(&self, message: EmailMessage<'_>) -> Result<(), mail_send::Error> {
        let mut mail_builder = MessageBuilder::new()
            .from(("", self.from.as_str()))
            .to(("", message.to))
            .subject(message.subject);

        if let Some(text) = message.text_body {
            mail_builder = mail_builder.text_body(text);
        }

        if let Some(html) = message.html_body {
            mail_builder = mail_builder.html_body(html);
        }

        SmtpClientBuilder::new(self.host.as_str(), self.port)
            .implicit_tls(false)
            .credentials((self.login.as_str(), self.password.as_str()))
            .connect()
            .await?
            .send(mail_builder)
            .await?;

        Ok(())
    }

    pub async fn send_registration_otp(&self, email: &str, otp: u32) -> AppResult<()> {
        self.send_message(EmailMessage {
            subject: "Код для подтверждения регистрации",
            to: email,
            text_body: Some(&format!("Ваш код подтверждения: {}", otp)),
            html_body: None,
        })
        .await
        .map_err(|err| AppError::Internal(anyhow::anyhow!("Failed to send email: {}", err)))?;
        Ok(())
    }
}
