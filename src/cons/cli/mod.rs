//! Command line interface for UCSC 100 vertebrate conservation data.

pub mod import;
pub mod query;

/// Common helpers for command line arguments.
pub mod args {
    /// Argument-related variables.
    pub mod vars {
        use crate::common::spdi;

        /// Argument group for specifying one of variant, position, or range.
        #[derive(clap::Args, Debug, Clone, Default)]
        #[group(required = true, multiple = false)]
        pub struct ArgsQuery {
            /// Specify variant to query for.
            #[arg(long, group = "query")]
            pub variant: Option<spdi::Var>,
            /// Specify position to query for.
            #[arg(long, group = "query")]
            pub position: Option<spdi::Pos>,
            /// Specify range to query for.
            #[arg(long, group = "query")]
            pub range: Option<spdi::Range>,
            /// Specify accession to query for.
            #[arg(long, group = "query")]
            pub accession: Option<String>,
            /// Query for all variants.
            #[arg(long, group = "query")]
            pub all: bool,
        }
    }
}
