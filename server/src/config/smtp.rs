pub struct SMTPConfig {
    pub from: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub password: String,
}

impl SMTPConfig {
    pub fn new() -> Self {
        Self {
            from: Self::get_from(),
            host: Self::get_host(),
            port: Self::get_port(),
            login: Self::get_login(),
            password: Self::get_password(),
        }
    }

    fn get_host() -> String {
        std::env::var("SMTP_HOST").expect("SMTP_HOST must be set")
    }

    fn get_port() -> u16 {
        match std::env::var("SMTP_PORT") {
            Ok(port_str) => port_str.parse().expect("SMTP_PORT must be a valid u16"),
            Err(_) => panic!("SMTP_PORT must be set"),
        }
    }

    fn get_login() -> String {
        std::env::var("SMTP_LOGIN").expect("SMTP_LOGIN must be set")
    }

    fn get_password() -> String {
        std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set")
    }

    fn get_from() -> String {
        std::env::var("SMTP_FROM").expect("SMTP_FROM must be set")
    }
}
