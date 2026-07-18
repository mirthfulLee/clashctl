#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("{0}")]
    InteractiveError(#[from] crate::interactive::InteractiveError),

    #[error("Clashctl error: {0}")]
    ClashCtl(#[from] crate::clashctl::Error),

    #[error("Clash/Mihomo API `{endpoint}` failed: {source}")]
    ApiEndpoint {
        endpoint: &'static str,
        source: crate::clashctl::Error,
    },

    #[error("Clash/Mihomo API `{0}` stream ended unexpectedly")]
    ApiStreamEnded(&'static str),

    #[error("{0}")]
    BackgroundError(String),

    #[error("TUI error")]
    TuiError(#[from] std::io::Error),

    #[error("TUI backend error")]
    TuiBackendErr,

    #[error("TUI interuptted error")]
    TuiInterupttedErr,

    #[error("TUI internal error")]
    TuiInternalErr,

    #[error("Set logger error ({0})")]
    SetLoggerError(#[from] log::SetLoggerError),
}

impl<T> From<std::sync::mpsc::SendError<T>> for TuiError {
    fn from(_: std::sync::mpsc::SendError<T>) -> Self {
        Self::TuiBackendErr
    }
}

pub type TuiResult<T> = std::result::Result<T, TuiError>;
