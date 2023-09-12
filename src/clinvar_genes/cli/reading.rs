//! Reading JSONL data for per-gene ClinVar information.

/// Reading of gene per-impact counts records.
pub mod gene_impact {
    /// SO terms for impact on gene
    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
    pub enum Impact {
        /// 3' UTR variant
        #[serde(rename = "3_prime_UTR_variant")]
        ThreePrimeUtrVariant,
        /// 5' UTR variant
        #[serde(rename = "5_prime_UTR_variant")]
        FivePrimeUtrVariant,
        /// downstream gene variant
        #[serde(rename = "downstream_gene_variant")]
        DownstreamGeneVariant,
        /// frameshift variant
        #[serde(rename = "frameshift_variant")]
        FrameshiftVariant,
        /// inframe deletion
        #[serde(rename = "inframe_indel")]
        InframeIndel,
        /// start lost
        #[serde(rename = "start_lost")]
        StartLost,
        /// intron variant
        #[serde(rename = "intron_variant")]
        IntronVariant,
        /// missense variant
        #[serde(rename = "missense_variant")]
        MissenseVariant,
        /// non-coding transcript variant
        #[serde(rename = "non_coding_transcript_variant")]
        NonCodingTranscriptVariant,
        /// stop gained
        #[serde(rename = "stop_gained")]
        StopGained,
        /// no sequence alteration
        #[serde(rename = "no_sequence_alteration")]
        NoSequenceAlteration,
        /// splice acceptor variant
        #[serde(rename = "splice_acceptor_variant")]
        SpliceAcceptorVariant,
        /// splice donor variant
        #[serde(rename = "splice_donor_variant")]
        SpliceDonorVariant,
        /// stop lost
        #[serde(rename = "stop_lost")]
        StopLost,
        /// synonymous variant
        #[serde(rename = "synonymous_variant")]
        SyonymousVariant,
        /// upstream gene variant
        #[serde(rename = "upstream_gene_variant")]
        UpstreamGeneVariant,
    }

    impl From<Impact> for crate::clinvar_genes::pbs::Impact {
        fn from(val: Impact) -> Self {
            match val {
                Impact::ThreePrimeUtrVariant => {
                    crate::clinvar_genes::pbs::Impact::ThreePrimeUtrVariant
                }
                Impact::FivePrimeUtrVariant => {
                    crate::clinvar_genes::pbs::Impact::FivePrimeUtrVariant
                }
                Impact::DownstreamGeneVariant => {
                    crate::clinvar_genes::pbs::Impact::DownstreamTranscriptVariant
                }
                Impact::FrameshiftVariant => crate::clinvar_genes::pbs::Impact::FrameshiftVariant,
                Impact::InframeIndel => crate::clinvar_genes::pbs::Impact::InframeIndel,
                Impact::StartLost => crate::clinvar_genes::pbs::Impact::StartLost,
                Impact::IntronVariant => crate::clinvar_genes::pbs::Impact::IntronVariant,
                Impact::MissenseVariant => crate::clinvar_genes::pbs::Impact::MissenseVariant,
                Impact::NonCodingTranscriptVariant => {
                    crate::clinvar_genes::pbs::Impact::NonCodingTranscriptVariant
                }
                Impact::StopGained => crate::clinvar_genes::pbs::Impact::StopGained,
                Impact::NoSequenceAlteration => {
                    crate::clinvar_genes::pbs::Impact::NoSequenceAlteration
                }
                Impact::SpliceAcceptorVariant => {
                    crate::clinvar_genes::pbs::Impact::SpliceAcceptorVariant
                }
                Impact::SpliceDonorVariant => crate::clinvar_genes::pbs::Impact::SpliceDonorVariant,
                Impact::StopLost => crate::clinvar_genes::pbs::Impact::StopLost,
                Impact::SyonymousVariant => crate::clinvar_genes::pbs::Impact::SynonymousVariant,
                Impact::UpstreamGeneVariant => {
                    crate::clinvar_genes::pbs::Impact::UpstreamTranscriptVariant
                }
            }
        }
    }

    /// ACMG clinical significance
    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
    pub enum ClinicalSignificance {
        /// Benign
        #[serde(rename = "benign")]
        Benign,
        /// Likely benign
        #[serde(rename = "likely benign")]
        LikelyBenign,
        /// Uncertain significance
        #[serde(rename = "uncertain significance")]
        UncertainSignificance,
        /// Likely pathogenic
        #[serde(rename = "likely pathogenic")]
        LikelyPathogenic,
        /// Pathogenic
        #[serde(rename = "pathogenic")]
        Pathogenic,
    }

    /// Gene-wise counts record.
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Record {
        /// HGNC gene ID
        pub hgnc: String,
        /// Per-impact counts
        pub counts: indexmap::IndexMap<Impact, Vec<u32>>,
    }
}

/// Reading of ACMG class by frequency counts records.
pub mod counts_by_freq {
    /// Coarsened clinical significance
    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
    pub enum CoarseClinicalSignificance {
        /// Likely benign / benign
        #[serde(rename = "benign")]
        Benign,
        /// Uncertain significance
        #[serde(rename = "uncertain")]
        Uncertain,
        /// Likely pathogenic / pathogenic
        #[serde(rename = "pathogenic")]
        Pathogenic,
    }

    impl From<CoarseClinicalSignificance> for crate::clinvar_genes::pbs::CoarseClinicalSignificance {
        fn from(val: CoarseClinicalSignificance) -> Self {
            match val {
                CoarseClinicalSignificance::Benign => {
                    crate::clinvar_genes::pbs::CoarseClinicalSignificance::CoarseBenign
                }
                CoarseClinicalSignificance::Uncertain => {
                    crate::clinvar_genes::pbs::CoarseClinicalSignificance::CoarseUncertain
                }
                CoarseClinicalSignificance::Pathogenic => {
                    crate::clinvar_genes::pbs::CoarseClinicalSignificance::CoarsePathogenic
                }
            }
        }
    }

    /// Per-pathogenicity counts.
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Record {
        /// HGNC gene ID
        pub hgnc: String,
        /// Per-impact counts
        pub counts: indexmap::IndexMap<CoarseClinicalSignificance, Vec<u32>>,
    }
}
