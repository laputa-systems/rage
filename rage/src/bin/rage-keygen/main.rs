#![forbid(unsafe_code)]

mod app;

#[path = "../rage/keygen.rs"]
mod keygen;

#[allow(dead_code)]
#[path = "../shared/cli_schema.rs"]
mod schema;

#[path = "../rage/messages.rs"]
mod messages;

fn main() -> std::process::ExitCode {
    app::run(std::env::args_os())
}
