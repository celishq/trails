use anyhow::Result;
use celis::command::{self, Command};
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(
    name = "celis",
    version,
    about = "Celis — cited cells over your sources (v6)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Initialize a .celis store in the current directory
    Init(command::Init),
    /// Manage source roots
    Source(command::Source),
    /// Snapshot registered roots into the hash log
    Sync(command::Sync),
    /// Sync sources and summarize them (= sync + summarize)
    Ingest(command::Ingest),
    /// Cells — a policy over the sources, synthesized into a cited brief
    Cell(command::Cell),
    /// Review changes queued by `ask`-gated cells (listens = inbound, speaks = outbound)
    Review(command::Review),
    /// Answer a question as an ephemeral cited cell
    Ask(command::Ask),
    /// Print the internal per-source summaries (the routing index) for debugging
    Summary(command::Summary),
    /// Show a cell's attributed change history (who changed what, when)
    History(command::History),
    /// Write an interactive HTML graph of cells and their cited sources
    Viz(command::Viz),
    /// Diff a cell's brief between two points in time
    Diff(command::Diff),
    /// Resolve a celis://source/... handle, or validate all footnotes if omitted
    Resolve(command::Resolve),
    /// Report unhealthy cells (broken, source-less, or dangling citations)
    Doctor(command::Doctor),
}

impl Cmd {
    fn command(&self) -> &dyn Command {
        match self {
            Cmd::Init(c) => c,
            Cmd::Source(c) => c,
            Cmd::Sync(c) => c,
            Cmd::Ingest(c) => c,
            Cmd::Cell(c) => c,
            Cmd::Review(c) => c,
            Cmd::Ask(c) => c,
            Cmd::Summary(c) => c,
            Cmd::History(c) => c,
            Cmd::Viz(c) => c,
            Cmd::Diff(c) => c,
            Cmd::Resolve(c) => c,
            Cmd::Doctor(c) => c,
        }
    }
}

fn main() -> Result<()> {
    let _ = dotenvy::dotenv(); // load .env if present; real env still wins
    let cli = Cli::parse();
    let cwd = env::current_dir()?;
    cli.cmd.command().run(&cwd)
}
