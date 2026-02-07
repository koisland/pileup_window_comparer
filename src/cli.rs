use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::paired::Mode;

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RegionType {
    /// Evaluate region with format: `chrom:st-end`
    #[arg(short, long)]
    pub region: Option<String>,

    /// Evaluate regions in BED3 file.
    #[arg(short, long)]
    pub bed: Option<PathBuf>,

    /// Evaluate all regions in chromosomes lengths. Accepts `fai`.
    #[arg(short, long)]
    pub lengths_chrom: Option<PathBuf>,
}

/// Summarize bedgraph over windows.
#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
pub struct WindowArgs {
    /// Bgzipped bedgraph. Must be indexed via `tabix -p bed`.
    #[arg(short, long)]
    pub infile: String,

    #[command(flatten)]
    pub over: RegionType,

    /// Window size to aggregate over.
    #[arg(short, long, default_value_t = 5000)]
    pub window: usize,

    /// Threads to use.
    #[arg(short, long, default_value_t = 4)]
    pub threads: usize,
}

/// Calculate ratio or difference between a treatment and control bedgraph over windows.
#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
pub struct PairArgs {
    /// Treatment bgzipped bedgraph. Must be indexed via `tabix -p bed`.
    #[arg(short, long)]
    pub treatment: String,

    /// Control bgzipped bedgraph. Must be indexed via `tabix -p bed`.
    #[arg(short, long)]
    pub control: String,

    #[command(flatten)]
    pub over: RegionType,

    /// Window size to aggregate over.
    #[arg(short, long, default_value_t = 5000)]
    pub window: usize,

    /// Mode to compare by.
    #[arg(short, long)]
    pub mode: Mode,

    /// Threads to use.
    #[arg(short, long, default_value_t = 4)]
    pub threads: usize,
}

/// (W)indow (A)ggregated (B)edgraphs
#[derive(Debug, Subcommand)]
pub enum Commands {
    Window(WindowArgs),
    Paired(PairArgs),
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
