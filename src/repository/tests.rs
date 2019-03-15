use crate::repository::GitRepository;

#[test]
pub fn remove_spaces_after_newline_empty() {
    let result = GitRepository::remove_spaces_after_newline(&[]);
    assert_eq!(0, result.len());
}

#[test]
pub fn remove_spaces_after_newline_singleton() {
    let result = GitRepository::remove_spaces_after_newline(b"0");
    assert_eq!(1, result.len());
}

#[test]
pub fn remove_spaces_after_newline_nothing() {
    let result = GitRepository::remove_spaces_after_newline(b"abcdef");
    assert_eq!(b"abcdef", result.as_slice());
}

#[test]
pub fn remove_spaces_after_newline_1() {
    let result = GitRepository::remove_spaces_after_newline(b"\n abc \n ");
    assert_eq!(b"\nabc \n", result.as_slice());
}

#[test]
pub fn remove_spaces_after_newline_2() {
    let result = GitRepository::remove_spaces_after_newline(b"\n\n");
    assert_eq!(b"\n\n", result.as_slice());
}

#[test]
pub fn remove_spaces_after_newline_3() {
    let result = GitRepository::remove_spaces_after_newline(b"  ");
    assert_eq!(b"  ", result.as_slice());
}