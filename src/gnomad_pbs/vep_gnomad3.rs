//! Protocolbuffers gnomAD v3 VEP data structures.

use std::str::FromStr;

include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.v1.vep_gnomad3.rs"
));

impl Vep {
    /// Returns number of fields in a gnomAD v2 VEP entry.
    pub fn num_fields() -> usize {
        45
    }
}

impl FromStr for Vep {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s.split('|').collect::<Vec<_>>();

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
            dbsnp_id: (!values[17].is_empty()).then(|| values[17].to_string()),
            distance: (!values[18].is_empty()).then(|| values[18].to_string()),
            strand: (!values[19].is_empty()).then(|| values[19].to_string()),
            variant_class: (!values[20].is_empty()).then(|| values[20].to_string()),
            minimised: (!values[21].is_empty()).then(|| values[21].to_string()),
            symbol_source: (!values[22].is_empty()).then(|| values[22].to_string()),
            hgnc_id: (!values[23].is_empty()).then(|| values[23].to_string()),
            canonical: (!values[24].is_empty()).then(|| values[24] == "YES"),
            tsl: (!values[25].is_empty())
                .then(|| values[25].parse())
                .transpose()?,
            appris: (!values[26].is_empty()).then(|| values[26].to_string()),
            ccds: (!values[27].is_empty()).then(|| values[27].to_string()),
            ensp: (!values[28].is_empty()).then(|| values[28].to_string()),
            swissprot: (!values[29].is_empty()).then(|| values[29].to_string()),
            trembl: (!values[30].is_empty()).then(|| values[30].to_string()),
            uniparc: (!values[31].is_empty()).then(|| values[31].to_string()),
            gene_pheno: (!values[32].is_empty()).then(|| values[32].to_string()),
            sift: (!values[33].is_empty())
                .then(|| -> Result<(String, f32), anyhow::Error> {
                    let tokens = values[33].split('(').collect::<Vec<_>>();
                    let mut tmp = tokens[1].chars();
                    tmp.next_back();
                    let score = tmp.as_str();
                    Ok((tokens[0].to_string(), score.parse::<f32>()?))
                })
                .transpose()?
                .map(|val| val.into()),
            polyphen: (!values[34].is_empty())
                .then(|| -> Result<(String, f32), anyhow::Error> {
                    let tokens = values[34].split('(').collect::<Vec<_>>();
                    let mut tmp = tokens[1].chars();
                    tmp.next_back();
                    let score = tmp.as_str();
                    Ok((tokens[0].to_string(), score.parse::<f32>()?))
                })
                .transpose()?
                .map(|val| val.into()),
            domains: (!values[35].is_empty())
                .then(|| {
                    let pairs = values[35].split('&').collect::<Vec<_>>();
                    pairs
                        .iter()
                        .map(|p| {
                            let tmp = p.split(':').collect::<Vec<_>>();
                            if tmp.len() < 2 {
                                dbg!(&values, &p);
                            }
                            super::vep_common::Domain {
                                source: tmp[0].to_string(),
                                id: tmp[1].to_string(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            hgvs_offset: (!values[36].is_empty()).then(|| values[36].to_string()),
            motif_name: (!values[37].is_empty()).then(|| values[37].to_string()),
            motif_pos: (!values[38].is_empty()).then(|| values[38].to_string()),
            high_inf_pos: (!values[39].is_empty()).then(|| values[39].to_string()),
            motif_score_change: (!values[40].is_empty()).then(|| values[40].to_string()),
            lof: (!values[41].is_empty()).then(|| values[41].to_string()),
            lof_filter: (!values[42].is_empty()).then(|| values[42].to_string()),
            lof_flags: (!values[43].is_empty()).then(|| values[43].to_string()),
            lof_info: (!values[44].is_empty()).then(|| values[44].to_string()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vep_from_string() {
        let s = "\
        G|missense_variant|MODERATE|MT-ND5|ENSG00000198786|Transcript|ENST00000361567|\
        protein_coding|1/1||ENST00000361567.2:c.208A>G|ENSP00000354813.2:p.Thr70Ala|208|\
        208|70|T/A|Aca/Gca|1||1|SNV||HGNC|HGNC:7461|YES||P1||ENSP00000354813||||1|\
        deleterious_low_confidence(0.020)|benign(0.033)|ENSP_mappings:5xtc&ENSP_mappings:5xtd&\
        ENSP_mappings:5xth&ENSP_mappings:5xti&ENSP_mappings:5xti&Pfam:PF00662&PANTHER:PTHR42829&\
        PANTHER:PTHR42829&TIGRFAM:TIGR01974|||||||||";
        let vep = Vep::from_str(s).unwrap();

        insta::assert_yaml_snapshot!(vep);
    }
}
