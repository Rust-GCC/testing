#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        {
            use colored::Colorize;

            eprintln!("[{}] {}", "info".green(), format_args!($($t)*));
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => {
        {
            use colored::Colorize;

            eprintln!("[{}] {}", "WARN".yellow(), format_args!($($t)*));
        }
    }
}
