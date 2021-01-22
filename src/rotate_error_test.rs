use super::RotateError;

const TEST_ERROR: &str = "Hello, World!";
const TEST_PROFILE: &str = "foo";

#[test]
fn new_rotate_error() {
    let e = RotateError::new(&TEST_ERROR, Some(TEST_PROFILE.to_string()));
    assert_eq!("Hello, World!", format!("{}", e));
    assert_eq!("foo", e.profile.unwrap());
}

#[test]
fn new_simple_rotate_error() {
    let e = RotateError::new_simple(&TEST_ERROR);
    assert_eq!("Hello, World!", format!("{}", e));
    assert!(e.profile.is_none())
}
