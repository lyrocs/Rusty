/// Unified error type for the rustykit framework.
#[derive(Debug)]
pub enum RustyError {
    Display(String),
    Spi(String),
    I2c(String),
    Storage(String),
    Network(String),
    Sprite(crate::sprite::SprError),
    Config(String),
    Io(String),
}

impl std::fmt::Display for RustyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustyError::Display(msg) => write!(f, "Display error: {}", msg),
            RustyError::Spi(msg) => write!(f, "SPI error: {}", msg),
            RustyError::I2c(msg) => write!(f, "I2C error: {}", msg),
            RustyError::Storage(msg) => write!(f, "Storage error: {}", msg),
            RustyError::Network(msg) => write!(f, "Network error: {}", msg),
            RustyError::Sprite(e) => write!(f, "Sprite error: {}", e),
            RustyError::Config(msg) => write!(f, "Config error: {}", msg),
            RustyError::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for RustyError {}

impl From<Box<dyn std::error::Error>> for RustyError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        RustyError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RustyError>;
