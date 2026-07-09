pub trait WlError {
    fn code(&self) -> u32;

    fn message(&self) -> &str;
}
