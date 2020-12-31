use crate::OutputFormat;
use anyhow::{Context as AnyhowContext, Result};
use git_commitgraph::{graph::verify::Outcome, Graph};
use std::{io, path::Path};

/// A general purpose context for many operations provided here
pub struct Context<W1: io::Write, W2: io::Write> {
    /// A stream to which to output errors
    pub err: W2,
    /// A stream to which to output operation results
    pub out: W1,
    pub output_statistics: Option<OutputFormat>,
}

impl Default for Context<Vec<u8>, Vec<u8>> {
    fn default() -> Self {
        Context {
            err: Vec::new(),
            out: Vec::new(),
            output_statistics: None,
        }
    }
}

pub fn graph_or_file<W1, W2>(
    path: impl AsRef<Path>,
    Context {
        err: _err,
        mut out,
        output_statistics,
    }: Context<W1, W2>,
) -> Result<git_commitgraph::graph::verify::Outcome>
where
    W1: io::Write,
    W2: io::Write,
{
    let g = Graph::at(path).with_context(|| "Could not open commit graph")?;

    #[allow(clippy::unnecessary_wraps, clippy::unknown_clippy_lints)]
    fn noop_processor(_commit: &git_commitgraph::file::Commit<'_>) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
    let stats = g
        .verify_integrity(noop_processor)
        .with_context(|| "Verification failure")?;

    match output_statistics {
        Some(OutputFormat::Human) => drop(print_statistics(&mut out, &stats)),
        #[cfg(feature = "serde1")]
        Some(OutputFormat::Json) => serde_json::to_writer_pretty(out, &stats)?,
        _ => {}
    }

    Ok(stats)
}

fn print_statistics(out: &mut impl io::Write, stats: &Outcome) -> io::Result<()> {
    writeln!(out, "number of commits with the given number of parents")?;
    let mut parent_counts: Vec<_> = stats.parent_counts.iter().map(|(a, b)| (*a, *b)).collect();
    parent_counts.sort_by_key(|e| e.0);
    for (parent_count, commit_count) in parent_counts.into_iter() {
        writeln!(out, "\t{:>2}: {}", parent_count, commit_count)?;
    }
    writeln!(out, "\t->: {}", stats.num_commits)?;

    write!(out, "\nlongest path length between two commits: ")?;
    if let Some(n) = stats.longest_path_length {
        writeln!(out, "{}", n)?;
    } else {
        writeln!(out, "unknown")?;
    }

    Ok(())
}
