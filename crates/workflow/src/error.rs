use crate::{bindings::WorkflowError, http};

/// A type alias for Result with WorkflowError as the error type.
pub type Result<T> = std::result::Result<T, WorkflowError>;

impl WorkflowError {
    /// Creates a new WorkflowError with the given id and error message.
    ///
    /// # Arguments
    ///
    /// * `id` - A String that holds the identifier for the error.
    /// * `error` - A String that contains the error message.
    ///
    /// # Returns
    ///
    /// A new instance of WorkflowError.
    pub fn new(id: Option<String>, error: String) -> Self {
        Self { id, error }
    }
}

impl From<http::Error> for WorkflowError {
    /// Converts an http::Error into a WorkflowError.
    ///
    /// This implementation handles two types of http::Error:
    /// - Status errors: Uses the status id as the WorkflowError id.
    /// - Transport errors: Uses the transport id as the WorkflowError id.
    ///
    /// The error message is created by debug-formatting the http::Error.
    ///
    /// # Arguments
    ///
    /// * `value` - The http::Error to convert.
    ///
    /// # Returns
    ///
    /// A new WorkflowError instance.
    fn from(value: http::Error) -> Self {
        WorkflowError {
            id: Some(value.id.to_string()),
            error: format!("{:?}", value),
        }
    }
}
