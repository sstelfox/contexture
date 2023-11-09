#[macro_export]
macro_rules! syscall {
    ($name:ident ( $($args:expr),* $(,)? )) => {
        match unsafe { libc::$name($($args),*) } {
            -1 => Err(std::io::Error::last_os_error()),
            ret => Ok(ret),
        }
    }
}
