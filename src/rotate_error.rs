/// Represents an Error that has occurred with rotate-iam-keys.
///
/// This is an error message from the application, not underlying libraries.
#[derive(Clone, Debug, PartialEq)]
pub struct RotateError {
    /// The underlying error message for rotate error.
    pub message: String,
}

impl RotateError {
    /// Create a new Rotate Error.
    ///
    /// * `message` — The Error message for this RotateError.
    pub fn new<S>(message: S) -> RotateError
    where
        S: ToString,
    {
        RotateError {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rotate_error() {
        let e = RotateError::new("Hello, World!");
        assert_eq!("Hello, World!", e.message)
    }
}
