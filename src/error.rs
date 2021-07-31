use std::error;
use std::process;

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, location, message);
}

pub fn fatal<T, E: error::Error>(result: Result<T, E>, exit_code: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            println!("fatal: {}", error.to_string());
            process::exit(exit_code);
        }
    }
}
