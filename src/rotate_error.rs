//! Represents an Error that has occurred with rotate-iam-keys.

#[cfg(test)]
#[path = "./rotate_error_test.rs"]
mod rotate_error_test;

/// This is an error message from the application, not underlying libraries.
#[derive(Clone, Debug, PartialEq)]
pub struct RotateError {
    /// The underlying error message for rotate error.
    pub message: String,
}

impl RotateError {
    /// Create a new `RotateError`.
    ///
    /// * `message` — The Error message for this `RotateError`.
    pub fn new<S>(message: &S) -> Self
    where
        S: ToString,
    {
        Self {
            message: message.to_string(),
        }
    }
}