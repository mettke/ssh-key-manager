use crate::serde::Serialize;
use std::{error, fmt};

/// Error happing on the Template Engine
#[derive(Debug)]
pub enum RenderError<T: TemplateEngine> {
    /// Custom Error from the underlying template engine
    Custom(T::TemplateError),
}

impl<T: TemplateEngine> fmt::Display for RenderError<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(err) => write!(f, "Custom Template Error: {}", err),
        }
    }
}

impl<T> error::Error for RenderError<T>
where
    T: TemplateEngine,
    <T as TemplateEngine>::TemplateError: 'static,
{
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Custom(err) => Some(err),
        }
    }
}

/// Provides methods required for a template engine.
pub trait TemplateEngine: Sized + Send + Sync + fmt::Debug {
    /// the custom error for the template engine.
    type TemplateError: error::Error;

    /// Compiles a template with the given name using the given data
    ///
    /// # Errors
    /// Fail when there is no template with that name or when the data couldn't be
    /// serialized
    fn render<T: Serialize>(
        &self,
        name: &str,
        data: &T,
    ) -> Result<String, RenderError<Self>>;
}
