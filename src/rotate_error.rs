//! Represents an Error that has occurred with rotate-iam-keys.
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

#[cfg(test)]
mod tests {
    use super::RotateError;
    const TEST_ERROR: &str = "Hello, World!";

    #[test]
    fn new_rotate_error() {
        let e = RotateError::new(&TEST_ERROR);
        assert_eq!("Hello, World!", e.message)
    }
}
