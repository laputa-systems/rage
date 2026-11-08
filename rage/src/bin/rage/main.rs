#![forbid(unsafe_code)]

mod app;
mod messages;

#[allow(dead_code)]
#[path = "../shared/cli_schema.rs"]
mod schema;

fn main() -> std::process::ExitCode {
    app::run(std::env::args_os())
}
