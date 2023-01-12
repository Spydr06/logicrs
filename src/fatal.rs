pub trait FatalResult<T> {
    fn unwrap_or_die(self) -> T;
}

impl<T, E> FatalResult<T> for Result<T, E>
where E: std::fmt::Display {
    fn unwrap_or_die(self) -> T {
        match self {
            Ok(value) => value,
            Err(err) => die(err.to_string().as_str())
        }
    }
}

pub fn die<'a>(reason: &'a str) -> ! {
    error!("[FATAL] {reason}");
    panic!()
}
