//! Verbs as commands. Each CLI verb opens the store it needs, runs one operation, and prints its
//! own output — so `main` is just a dispatch table and the `Node` stays the state holder.

use crate::cell::{Direction, Pending};
use crate::llm::{Models, OpenAiLlm};
use crate::Node;
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

/// One CLI verb: open a store rooted at `cwd`, do the work, and print the result.
pub trait Command {
    fn run(&self, cwd: &Path) -> Result<()>;
}

/// Open a store that reports progress to stderr (for the slow, LLM-backed commands).
fn open_verbose(cwd: &Path) -> Result<Node> {
    Ok(Node::open(cwd)?.on_progress(|m| eprintln!("  {m}")))
}

/// Show a proposed field and let the owner approve it (Enter) or replace it (type a new value).
/// Non-interactive input (a pipe, no TTY) accepts the proposal unchanged, so scripted use is unaffected.
fn confirm_field(label: &str, value: &str) -> Result<String> {
    use std::io::{stdin, stdout, IsTerminal, Write};
    let shown = if value.trim().is_empty() { "(none — free-form)" } else { value };
    println!("\n{label}:\n  {shown}");
    if !stdin().is_terminal() {
        return Ok(value.to_string());
    }
    print!("  Enter to accept, or type a replacement: ");
    stdout().flush()?;
    let mut line = String::new();
    stdin().read_line(&mut line)?;
    let line = line.trim();
    Ok(if line.is_empty() { value.to_string() } else { line.to_string() })
}

#[derive(Args)]
pub struct Init {
    /// Vault folder to create
    pub path: PathBuf,
}
impl Command for Init {
    fn run(&self, cwd: &Path) -> Result<()> {
        let vault = cwd.join(&self.path);
        Node::init(&vault)?;
        println!("initialized vault {}", vault.display());
        Ok(())
    }
}

#[derive(Args)]
pub struct Source {
    #[command(subcommand)]
    pub cmd: SourceCmd,
}

#[derive(Subcommand)]
pub enum SourceCmd {
    /// Register a source root (a vault or folder of markdown)
    Add { path: PathBuf },
    /// List registered roots
    List,
}

impl Command for Source {
    fn run(&self, cwd: &Path) -> Result<()> {
        let node = Node::open(cwd)?;
        match &self.cmd {
            SourceCmd::Add { path } => {
                node.add_root(path)?;
                println!("added root {}", path.display());
            }
            SourceCmd::List => {
                for root in node.roots()? {
                    println!("{}", root.display());
                }
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Sync;
impl Command for Sync {
    fn run(&self, cwd: &Path) -> Result<()> {
        let r = Node::open(cwd)?.sync()?;
        println!(
            "sync: +{} ~{} -{} ={}",
            r.added, r.updated, r.tombstoned, r.unchanged
        );
        Ok(())
    }
}

#[derive(Args)]
pub struct Ingest {
    /// Skip auto-refreshing cells affected by the new sources
    #[arg(long)]
    pub no_cells: bool,
}
impl Command for Ingest {
    fn run(&self, cwd: &Path) -> Result<()> {
        let (quick, smart) = (OpenAiLlm::from_env(), OpenAiLlm::recall_from_env());
        let m = Models {
            quick: &quick,
            smart: &smart,
        };
        let n = open_verbose(cwd)?.ingest(&m, !self.no_cells)?;
        println!("ingest: {n} summarized");
        Ok(())
    }
}

#[derive(Args)]
pub struct Cell {
    #[command(subcommand)]
    pub cmd: CellCmd,
}

#[derive(Subcommand)]
pub enum CellCmd {
    /// Create a cell: a policy synthesized into a cited brief
    Create {
        name: String,
        #[arg(long)]
        policy: String,
    },
    /// Re-run a cell's stored policy (a no-op if no relevant source changed)
    Refresh { name: String },
    /// Print a cell's cited brief
    Show { name: String },
    /// List defined cells
    List,
    /// Propose cells worth creating from the source summaries (LLM)
    Suggest,
}

impl Command for Cell {
    fn run(&self, cwd: &Path) -> Result<()> {
        let node = open_verbose(cwd)?;
        let (quick, smart) = (OpenAiLlm::from_env(), OpenAiLlm::recall_from_env());
        let m = Models {
            quick: &quick,
            smart: &smart,
        };
        match &self.cmd {
            CellCmd::Create { name, policy } => {
                let mut spec = node.propose_cell_spec(policy, &smart);
                println!("Proposed cell '{name}':");
                spec.rules = confirm_field("Matching rules — which sources belong in this cell", &spec.rules)?;
                spec.format = confirm_field("Output format — the shape of the brief", &spec.format)?;
                let n = node.create_cell_from_spec(name, &spec, &m)?;
                println!("created cell {name} ({n} source(s))");
            }
            CellCmd::Refresh { name } => {
                let n = node.cell_refresh_policy(name, &m)?;
                println!("refreshed cell {name} ({n} source(s))");
            }
            CellCmd::Show { name } => print!("{}", node.cell_show_policy(name)?),
            CellCmd::List => {
                for c in node.cell_list()? {
                    println!("{c}");
                }
            }
            CellCmd::Suggest => {
                let proposals = node.suggest_policy(&OpenAiLlm::recall_from_env())?;
                if proposals.is_empty() {
                    println!("(no cells worth proposing yet)");
                }
                for p in &proposals {
                    println!("{} — {}", p.name, p.policy);
                    if !p.sources.is_empty() {
                        println!("    evidence: {}", p.sources.join(", "));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Review {
    #[command(subcommand)]
    pub cmd: ReviewCmd,
}

#[derive(Subcommand)]
pub enum ReviewCmd {
    /// Review inbound assertions a `listen: ask` cell is waiting to absorb
    Listens {
        /// Limit to one cell
        #[arg(long)]
        cell: Option<String>,
    },
    /// Review outbound assertions a `speak: ask` cell wants to emit
    Speaks {
        /// Limit to one cell
        #[arg(long)]
        cell: Option<String>,
    },
}

impl Command for Review {
    fn run(&self, cwd: &Path) -> Result<()> {
        let node = open_verbose(cwd)?;
        let (quick, smart) = (OpenAiLlm::from_env(), OpenAiLlm::recall_from_env());
        let m = Models {
            quick: &quick,
            smart: &smart,
        };
        let (direction, cell) = match &self.cmd {
            ReviewCmd::Listens { cell } => (Direction::Listen, cell),
            ReviewCmd::Speaks { cell } => (Direction::Speak, cell),
        };
        let items: Vec<_> = node
            .pending_review(cell.as_deref())?
            .into_iter()
            .filter(|(_, p)| p.direction == direction)
            .collect();
        if items.is_empty() {
            println!("(nothing to review)");
        }
        let auto = node.review_auto();
        for (id, p) in items {
            // Truth gate: annotate each item with the matching review rule's suggestion; with
            // `review_auto: true`, an `auto-accept` is applied without prompting.
            if let Ok(Some(v)) = node.review_suggestion(&p, &smart) {
                println!("  (review rule suggests: {} — {})", v.decision, v.reason);
                if auto && v.decision == "auto-accept" {
                    node.apply_pending(&id, &m)?;
                    println!("  auto-applied");
                    continue;
                }
            }
            match review_action(&p)? {
                ReviewAction::Apply => {
                    node.apply_pending(&id, &m)?;
                    println!("  applied");
                }
                ReviewAction::Discard(reason) => {
                    node.discard_pending(&id, reason.as_deref())?;
                    println!("  discarded");
                }
                ReviewAction::Skip => {}
            }
        }
        Ok(())
    }
}

enum ReviewAction {
    Apply,
    Discard(Option<String>),
    Skip,
}

/// Show one queued change and ask the owner to apply/discard/skip it. Non-interactive input (no TTY)
/// lists it and skips, so a scripted `cell review` is a safe read-only listing.
fn review_action(p: &Pending) -> Result<ReviewAction> {
    use std::io::{stdin, stdout, IsTerminal, Write};
    let arrow = match p.direction {
        Direction::Speak => "speak →",
        Direction::Listen => "listen ←",
    };
    println!("\n[{arrow}] {}: {}", p.cell, p.claim);
    if !stdin().is_terminal() {
        return Ok(ReviewAction::Skip);
    }
    print!("  [a]pply / [d]iscard / [s]kip: ");
    stdout().flush()?;
    let mut line = String::new();
    stdin().read_line(&mut line)?;
    match line.trim() {
        "a" | "apply" => Ok(ReviewAction::Apply),
        "d" | "discard" => {
            print!("  reason (optional, helps the agent learn): ");
            stdout().flush()?;
            let mut reason = String::new();
            stdin().read_line(&mut reason)?;
            let reason = reason.trim();
            Ok(ReviewAction::Discard(
                (!reason.is_empty()).then(|| reason.to_string()),
            ))
        }
        _ => Ok(ReviewAction::Skip),
    }
}

#[derive(Args)]
pub struct Ask {
    pub question: String,
    /// Save the cited answer as a policy cell named for the question
    #[arg(long)]
    pub save: bool,
}
impl Command for Ask {
    fn run(&self, cwd: &Path) -> Result<()> {
        let node = open_verbose(cwd)?;
        let (quick, smart) = (OpenAiLlm::from_env(), OpenAiLlm::recall_from_env());
        let m = Models {
            quick: &quick,
            smart: &smart,
        };
        let answer = node.ask_policy(&m, &self.question, self.save)?;
        if answer.trim_start().starts_with("INSUFFICIENT") {
            println!("Hmm, I couldn't find anything about that in your sources yet — try a different question, or `celis ingest` more sources first.");
        } else {
            println!("{answer}");
            if self.save {
                eprintln!("(saved as {})", crate::markup::slug(&self.question));
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Summary;
impl Command for Summary {
    fn run(&self, cwd: &Path) -> Result<()> {
        let summaries = Node::open(cwd)?.summaries()?;
        if summaries.is_empty() {
            println!("(no summaries — run `celis ingest`)");
        } else {
            for (path, text) in &summaries {
                println!("{path} — {text}");
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct History {
    /// The cell whose attributed change history to show
    pub cell: String,
}
impl Command for History {
    fn run(&self, cwd: &Path) -> Result<()> {
        let lines = Node::open(cwd)?.history(&self.cell)?;
        if lines.is_empty() {
            println!("(no history — run an ingest, or git is unavailable)");
        } else {
            for l in &lines {
                println!("{l}");
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Viz {
    /// Output HTML file
    #[arg(long, default_value = "cells-graph.html")]
    pub out: PathBuf,
}
impl Command for Viz {
    fn run(&self, cwd: &Path) -> Result<()> {
        let graph = Node::open(cwd)?.citation_graph()?;
        std::fs::write(&self.out, crate::viz::render_graph_html(&graph))?;
        let citations: usize = graph.iter().map(|(_, s)| s.len()).sum();
        println!(
            "wrote {} ({} cells, {citations} citations)",
            self.out.display(),
            graph.len()
        );
        Ok(())
    }
}

#[derive(Args)]
pub struct Diff {
    /// The cell to diff
    pub cell: String,
    /// Window start (RFC3339 or YYYY-MM-DD); defaults to the cell's first synthesis
    #[arg(long)]
    pub from: Option<String>,
    /// Window end (RFC3339 or YYYY-MM-DD); defaults to the cell's latest synthesis
    #[arg(long)]
    pub to: Option<String>,
}
impl Command for Diff {
    fn run(&self, cwd: &Path) -> Result<()> {
        let diff =
            Node::open(cwd)?.cell_diff(&self.cell, self.from.as_deref(), self.to.as_deref())?;
        if diff.trim().is_empty() {
            println!("(no change in that window)");
        } else {
            print!("{diff}");
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Doctor;
impl Command for Doctor {
    fn run(&self, cwd: &Path) -> Result<()> {
        let issues = Node::open(cwd)?.doctor()?;
        if issues.is_empty() {
            println!("all cells healthy");
        } else {
            for i in &issues {
                println!("{i}");
            }
        }
        Ok(())
    }
}

#[derive(Args)]
pub struct Resolve {
    pub handle: Option<String>,
}
impl Command for Resolve {
    fn run(&self, cwd: &Path) -> Result<()> {
        let node = Node::open(cwd)?;
        match &self.handle {
            Some(h) => println!("{}", node.resolve(h)?),
            None => {
                let problems = node.check_footnotes()?;
                if problems.is_empty() {
                    println!("all footnotes resolve");
                } else {
                    for p in &problems {
                        println!("dangling: {p}");
                    }
                }
            }
        }
        Ok(())
    }
}
