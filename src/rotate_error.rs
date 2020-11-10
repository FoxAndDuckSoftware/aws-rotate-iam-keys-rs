//! Represents an Error that has occurred with rotate-iam-keys.

use rusoto_core::RusotoError;
use std::error::Error;
use std::fmt;

#[cfg(test)]
#[path = "./rotate_error_test.rs"]
mod rotate_error_test;

/// This is an error message from the application, not underlying libraries.
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

impl<E: Error + 'static> From<RusotoError<E>> for RotateError {
    fn from(err: RusotoError<E>) -> Self {
        Self {
            message: format!("{}", err),
        }
    }
}

impl Clone for RotateError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
        }
    }
}

impl fmt::Display for RotateError {
    /// Display message
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for RotateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RotateError")
            .field("message", &self.message)
            .finish()
    }
}
