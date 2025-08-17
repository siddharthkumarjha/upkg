#[macro_export]
macro_rules! lua_err_context {
    () => {
        |_| LuaError::external(format!("[{}:{}]", file!(), line!()))
    };

    ($arg:expr) => {
        |_| LuaError::external(format!("[{}:{}]: {}", file!(), line!(), $arg))
    };

    ($fmt:expr, $($arg:tt)*) => {
        |_| LuaError::external(format!(
            "[{}:{}] {}",
            file!(), line!(),
            format!($fmt, $($arg)*)
        ))
    };
}

#[macro_export]
macro_rules! io_err_context {
    () => {
        |err| std::io::Error::new(err.kind(), format!("[{}:{}] {}", file!(), line!(), err))
    };

    ($arg:expr) => {
        |err| std::io::Error::new(err.kind(), format!("[{}:{}]: {} {}", file!(), line!(), $arg, err))
    };

    ($fmt:expr, $($arg:tt)*) => {
        |err| std::io::Error::new(err.kind(), format!(
            "[{}:{}] {} {}",
            file!(), line!(),
            format!($fmt, $($arg)*),
            err
        ))
    };
}
