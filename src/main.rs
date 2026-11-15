//! `easy-note` and `n` are the same binary: `n` runs as `easy-note add`.

use easy_note_mcp::surfaces;

fn main() {
    let args = surfaces::cli::normalize_args(std::env::args_os().collect());
    std::process::exit(surfaces::cli::run(args));
}
