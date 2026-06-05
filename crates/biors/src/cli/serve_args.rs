use clap::Args;

pub(crate) const DEFAULT_SERVICE_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_SERVICE_PORT: u16 = 8787;
pub(crate) const DEFAULT_MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = DEFAULT_SERVICE_HOST)]
    pub host: String,
    #[arg(long, default_value_t = DEFAULT_SERVICE_PORT)]
    pub port: u16,
    #[arg(long, default_value_t = DEFAULT_MAX_BODY_BYTES)]
    pub max_body_bytes: usize,
}

impl ServeArgs {
    pub(crate) fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub(crate) fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
