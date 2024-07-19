//! Common CLI code.

/// Commonly used command line arguments.
#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    /// Verbosity of the program
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

/// Output format to write.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, clap::ValueEnum, strum::Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum OutputFormat {
    /// JSONL format.
    #[default]
    Jsonl,
}

/// Local genome release for command line arguments.
#[derive(
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    clap::ValueEnum,
    strum::Display,
    strum::EnumString,
    enum_map::Enum,
    serde::Serialize,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "lowercase")]
pub enum GenomeRelease {
    /// GRCh37 genome release.
    #[strum(serialize = "grch37")]
    #[default]
    Grch37,
    /// GRCh38 genome release.
    #[strum(serialize = "grch38")]
    Grch38,
}

impl From<GenomeRelease> for biocommons_bioutils::assemblies::Assembly {
    fn from(val: GenomeRelease) -> Self {
        match val {
            GenomeRelease::Grch37 => biocommons_bioutils::assemblies::Assembly::Grch37p10,
            GenomeRelease::Grch38 => biocommons_bioutils::assemblies::Assembly::Grch38,
        }
    }
}

/// Construct the `indicatif` style for progress bars.
pub fn indicatif_style() -> indicatif::ProgressStyle {
    let tpl = "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
    {human_pos}/{human_len} ({per_sec})";
    indicatif::ProgressStyle::with_template(tpl)
        .unwrap()
        .with_key(
            "eta",
            |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            },
        )
        .progress_chars("#>-")
}

/// Construct an `indicatif` progress bar with the common style.
///
/// Also, we will enable a steady tick every 0.1s and hide in tests.
pub fn progress_bar(#[allow(unused_variables)] len: usize) -> indicatif::ProgressBar {
    #[cfg(test)]
    let pb = indicatif::ProgressBar::hidden();
    #[cfg(not(test))]
    let pb = indicatif::ProgressBar::new(len as u64).with_style(indicatif_style());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Canonical chromosome names.
///
/// Note that the mitochondrial genome runs under two names.
pub const CANONICAL: &[&str] = &[
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17",
    "18", "19", "20", "21", "22", "X", "Y", "M", "MT",
];

/// Make a chromosome name canonical.
pub fn canonicalize(chrom: &str) -> String {
    let chrom = chrom.strip_prefix("chr").unwrap_or(chrom);
    if chrom == "M" {
        "MT".to_string()
    } else {
        chrom.to_string()
    }
}

/// Return whether the given chromosome name is a canonical one.
///
/// The prefix `"chr"` is stripped from the name before checking.
pub fn is_canonical(chrom: &str) -> bool {
    let chrom = chrom.strip_prefix("chr").unwrap_or(chrom);
    CANONICAL.contains(&chrom)
}

/// Build windows for a given assembly.
pub fn build_genome_windows(
    assembly: biocommons_bioutils::assemblies::Assembly,
    window_size: Option<usize>,
) -> Result<Vec<(String, usize, usize)>, anyhow::Error> {
    let mut result = Vec::new();

    for seq in &biocommons_bioutils::assemblies::ASSEMBLY_INFOS[assembly].sequences {
        if is_canonical(&seq.name) {
            let window_size = window_size.unwrap_or(seq.length);
            let mut start = 0;
            let mut end = window_size;
            while start < seq.length {
                if end > seq.length {
                    end = seq.length;
                }
                result.push((seq.name.clone(), start, end));
                start = end;
                end += window_size;
            }
        }
    }

    Ok(result)
}

/// Helpers to extract chromosome name from `<release>:<chrom>` string.
pub mod extract_chrom {
    use crate::common::spdi;

    /// Get chromosome from the SPDI variant.
    ///
    /// If the optional genome release was given then it is compared to the one specified
    /// in `expected_genome_release` and stripped (comparision is case insensitive).
    pub fn from_var(
        variant: &spdi::Var,
        expected_genome_release: Option<&str>,
    ) -> Result<String, anyhow::Error> {
        if variant.sequence.contains(':') {
            let mut iter = variant.sequence.rsplitn(2, ':');
            let chromosome = iter.next().unwrap();
            if let Some(genome_release) = iter.next() {
                if let Some(expected_genome_release) = expected_genome_release {
                    if genome_release.to_lowercase() != expected_genome_release.to_lowercase() {
                        return Err(anyhow::anyhow!(
                            "genome release mismatch (lowercase): expected {}, got {}",
                            expected_genome_release,
                            genome_release
                        ));
                    }
                }
            }
            Ok(super::canonicalize(chromosome))
        } else {
            Ok(super::canonicalize(&variant.sequence))
        }
    }

    /// Get chromosome from the SPDI position.
    ///
    /// See `from_var` for details.
    pub fn from_pos(
        pos: &spdi::Pos,
        expected_genome_release: Option<&str>,
    ) -> Result<String, anyhow::Error> {
        if pos.sequence.contains(':') {
            let mut iter = pos.sequence.rsplitn(2, ':');
            let chromosome = iter.next().unwrap();
            if let Some(genome_release) = iter.next() {
                if let Some(expected_genome_release) = expected_genome_release {
                    if genome_release.to_lowercase() != expected_genome_release.to_lowercase() {
                        return Err(anyhow::anyhow!(
                            "genome release mismatch (lowercase): expected {}, got {}",
                            expected_genome_release,
                            genome_release
                        ));
                    }
                }
            }
            Ok(super::canonicalize(chromosome))
        } else {
            Ok(super::canonicalize(&pos.sequence))
        }
    }

    /// Get chromosome from the SPDI range.
    ///
    /// See `from_var` for details.
    pub fn from_range(
        range: &spdi::Range,
        expected_genome_release: Option<&str>,
    ) -> Result<String, anyhow::Error> {
        if range.sequence.contains(':') {
            let mut iter = range.sequence.rsplitn(2, ':');
            let chromosome = iter.next().unwrap();
            if let Some(genome_release) = iter.next() {
                if let Some(expected_genome_release) = expected_genome_release {
                    if genome_release.to_lowercase() != expected_genome_release.to_lowercase() {
                        return Err(anyhow::anyhow!(
                            "genome release mismatch (lowercase): expected {}, got {}",
                            expected_genome_release,
                            genome_release
                        ));
                    }
                }
            }
            Ok(super::canonicalize(chromosome))
        } else {
            Ok(super::canonicalize(&range.sequence))
        }
    }
}
