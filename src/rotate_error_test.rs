use super::RotateError;

const TEST_ERROR: &str = "Hello, World!";

#[test]
fn new_rotate_error() {
    let e = RotateError::new(&TEST_ERROR);
    assert_eq!("Hello, World!", format!(e))
}
