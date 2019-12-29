mod cli;
mod tui;

fn main() {
    let args = cli::parse_args().unwrap_or_else(|e| e.exit());
    cli::search(args);
}
