// Return early with an error if a condition is not satisfied.
#[doc(hidden)]
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            return Err($crate::Error::new($msg.into()));
        };
    };

    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($crate::Error::new($msg.into()));
        };
    };
}

// Return early with an error.
#[doc(hidden)]
#[macro_export]
macro_rules! bail {
    ($msg:literal) => {
        return Err($crate::Error::new($msg.into()));
    };

    ($msg:expr) => {
        return Err($crate::Error::new($msg.into()));
    };

    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::new(&*format!($fmt, $($arg)*)));
    };
}
