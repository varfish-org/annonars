//! Code generate for protobufs by `prost-build`.

use std::str::FromStr;

include!(concat!(env!("OUT_DIR"), "/annonars.gnomad.vep_gnomad4.rs"));
include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.vep_gnomad4.serde.rs"
));

impl Vep {
    /// Returns number of fields in a gnomAD v4 VEP entry.
    pub fn num_fields() -> usize {
        46
    }
}

impl FromStr for Vep {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s.split('|').collect::<Vec<_>>();

        if values.len() < 2 {
            dbg!(s);
        }

        Ok(Vep {
            allele: values[0].to_string(),
            consequence: values[1].to_string(),
            impact: values[2].to_string(),
            symbol: values[3].to_string(),
            gene: values[4].to_string(),
            feature_type: values[5].to_string(),
            feature: values[6].to_string(),
            feature_biotype: values[7].to_string(),
            exon: (!values[8].is_empty()).then(|| values[8].to_string()),
            intron: (!values[9].is_empty()).then(|| values[9].to_string()),
            hgvsc: (!values[10].is_empty()).then(|| values[10].to_string()),
            hgvsp: (!values[11].is_empty()).then(|| values[11].to_string()),
            cdna_position: (!values[12].is_empty()).then(|| values[12].to_string()),
            cds_position: (!values[13].is_empty()).then(|| values[13].to_string()),
            protein_position: (!values[14].is_empty()).then(|| values[14].to_string()),
            amino_acids: (!values[15].is_empty()).then(|| values[15].to_string()),
            codons: (!values[16].is_empty()).then(|| values[16].to_string()),
            allele_num: (!values[17].is_empty())
                .then(|| values[17].parse())
                .transpose()
                .map_err(|e| anyhow::anyhow!("problem parsing vep/ALLELE_NUM: {}", e))?,
            distance: (!values[18].is_empty()).then(|| values[18].to_string()),
            strand: (!values[19].is_empty()).then(|| values[19].to_string()),
            flags: (!values[20].is_empty()).then(|| values[20].to_string()),
            variant_class: (!values[21].is_empty()).then(|| values[21].to_string()),
            symbol_source: (!values[22].is_empty()).then(|| values[22].to_string()),
            hgnc_id: (!values[23].is_empty()).then(|| values[23].to_string()),
            canonical: (!values[24].is_empty()).then(|| values[24] == "YES"),
            mane_select: (!values[25].is_empty()).then(|| values[25] == "YES"),
            mane_plus_clinical: (!values[26].is_empty()).then(|| values[26] == "YES"),
            tsl: (!values[27].is_empty())
                .then(|| values[27].parse())
                .transpose()
                .map_err(|e| anyhow::anyhow!("problem parsing vep/TSL: {}", e))?,
            appris: (!values[28].is_empty()).then(|| values[28].to_string()),
            ccds: (!values[29].is_empty()).then(|| values[29].to_string()),
            ensp: (!values[30].is_empty()).then(|| values[30].to_string()),
            uniprot_isoform: (!values[31].is_empty()).then(|| values[31].to_string()),
            source: (!values[32].is_empty()).then(|| values[32].to_string()),
            domains: (!values[33].is_empty())
                .then(|| {
                    let pairs = values[33].split('&').collect::<Vec<_>>();
                    pairs
                        .into_iter()
                        .filter(|s| *s != "null")
                        .map(|p| {
                            let tmp = p.split(':').collect::<Vec<_>>();
                            super::vep_common::Domain {
                                source: tmp[0].to_string(),
                                id: tmp[1].to_string(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            mirna: (!values[34].is_empty()).then(|| values[34].to_string()),
            hgvs_offset: (!values[35].is_empty()).then(|| values[35].to_string()),
            pubmed: (!values[36].is_empty()).then(|| values[36].to_string()),
            motif_name: (!values[37].is_empty()).then(|| values[37].to_string()),
            motif_pos: (!values[38].is_empty()).then(|| values[38].to_string()),
            high_inf_pos: (!values[39].is_empty()).then(|| values[39].to_string()),
            motif_score_change: (!values[40].is_empty()).then(|| values[40].to_string()),
            transcription_factors: (!values[41].is_empty()).then(|| values[41].to_string()),
            lof: (!values[42].is_empty()).then(|| values[42].to_string()),
            lof_filter: (!values[43].is_empty()).then(|| values[43].to_string()),
            lof_flags: (!values[44].is_empty()).then(|| values[44].to_string()),
            lof_info: (!values[45].is_empty()).then(|| values[45].to_string()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vep_from_string() {
        let s = "\
        C|upstream_gene_variant|MODIFIER|DDX11L1|ENSG00000223972|\
        Transcript|ENST00000450305|transcribed_unprocessed_pseudogene||||||||||\
        1|1979|1||SNV|HGNC|HGNC:37102|YES||||||||Ensembl|||||||||||||||,C|\
        upstream_gene_variant|MODIFIER|DDX11L1|ENSG00000223972|Transcript|\
        ENST00000456328|processed_transcript||||||||||1|1838|1||SNV|HGNC|\
        HGNC:37102||||1|||||Ensembl|||||||||||||||,C|downstream_gene_variant|\
        MODIFIER|WASH7P|ENSG00000227232|Transcript|ENST00000488147|\
        unprocessed_pseudogene||||||||||1|4373|-1||SNV|HGNC|HGNC:38034|YES||||||||\
        Ensembl|||||||||||||||,C|downstream_gene_variant|MODIFIER|WASH7P|653635|\
        Transcript|NR_024540.1|transcribed_pseudogene||||||||||1|4331|-1||SNV|\
        EntrezGene|HGNC:38034|YES||||||||RefSeq|||||||||||||||,C|\
        upstream_gene_variant|MODIFIER|DDX11L1|100287102|Transcript|NR_046018.2|\
        transcribed_pseudogene||||||||||1|1843|1||SNV|EntrezGene|HGNC:37102|YES\
        ||||||||RefSeq|||||||||||||||";
        let vep = Vep::from_str(s).unwrap();

        insta::assert_yaml_snapshot!(vep);
    }
}
