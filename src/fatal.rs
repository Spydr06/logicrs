pub trait FatalResult<T> {
    fn unwrap_or_die(self) -> T;
}

impl<T, E> FatalResult<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn unwrap_or_die(self) -> T {
        self.unwrap_or_else(|err| die(&err.to_string()))
    }
}

pub fn die(reason: &str) -> ! {
    error!("[FATAL] {reason}");
    panic!()
}
