//! Common CLI code.

/// Commonly used command line arguments.
#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    /// Verbosity of the program
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

/// Local genome release for command line arguments.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, clap::ValueEnum, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum GenomeRelease {
    /// GRCh37 genome release.
    Grch37,
    /// GRCh38 genome release.
    Grch38,
}

impl From<GenomeRelease> for hgvs::static_data::Assembly {
    fn from(val: GenomeRelease) -> Self {
        match val {
            GenomeRelease::Grch37 => hgvs::static_data::Assembly::Grch37,
            GenomeRelease::Grch38 => hgvs::static_data::Assembly::Grch38,
        }
    }
}

/// Construct the `indicatif` style for progress bars.
pub fn indicatif_style() -> indicatif::ProgressStyle {
    let tpl = "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
    {human_pos}/{human_len} ({eta})";
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

/// Canonical chromosome names.
///
/// Note that the mitochondrial genome runs under two names.
pub const CANONICAL: &[&str] = &[
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17",
    "18", "19", "20", "21", "22", "X", "Y", "M", "MT",
];

/// Return whether the given chromosome name is a canonical one.
///
/// The prefix `"chr"` is stripped from the name before checking.
pub fn is_canonical(chrom: &str) -> bool {
    let chrom = chrom.strip_prefix("chr").unwrap_or(chrom);
    CANONICAL.contains(&chrom)
}

/// Build windows for a given assembly.
pub fn build_genome_windows(
    assembly: hgvs::static_data::Assembly,
    window_size: Option<usize>,
) -> Result<Vec<(String, usize, usize)>, anyhow::Error> {
    let mut result = Vec::new();

    for seq in &hgvs::static_data::ASSEMBLY_INFOS[assembly].sequences {
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
