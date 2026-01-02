use clap::Parser;

#[derive(Parser)]
#[command(name = "rsvp-term")]
#[command(about = "TUI for RSVP reading of markdown prose")]
struct Cli {
    /// Markdown file to read
    file: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    println!("Reading: {:?}", cli.file);
}
