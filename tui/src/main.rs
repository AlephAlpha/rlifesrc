mod cli;
mod tui;

fn main() {
    let args = cli::parse_args().unwrap();
    cli::search(args);
}
