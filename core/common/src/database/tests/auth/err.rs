use std::{error, fmt};

#[derive(Debug, Copy, Clone)]
pub struct TestErr;
impl error::Error for TestErr {}
impl fmt::Display for TestErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError")
    }
}
