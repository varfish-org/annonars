//! Protocolbuffers gnomAD v3 VEP data structures.

use std::str::FromStr;

include!(concat!(
    env!("OUT_DIR"),
    "/annonars.gnomad.v1.vep_gnomad2.rs"
));

impl Vep {
    /// Returns number of fields in a gnomAD v3 VEP entry.
    pub fn num_fields() -> usize {
        68
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
            existing_variation: (!values[17].is_empty()).then(|| values[17].to_string()),
            dbsnp_id: (!values[18].is_empty()).then(|| values[18].to_string()),
            distance: (!values[19].is_empty()).then(|| values[19].to_string()),
            strand: (!values[20].is_empty()).then(|| values[20].to_string()),
            flags: (!values[21].is_empty()).then(|| values[21].to_string()),
            variant_class: (!values[22].is_empty()).then(|| values[22].to_string()),
            minimised: (!values[23].is_empty()).then(|| values[23].to_string()),
            symbol_source: (!values[24].is_empty()).then(|| values[24].to_string()),
            hgnc_id: (!values[25].is_empty()).then(|| values[25].to_string()),
            canonical: values[26] == "YES",
            tsl: (!values[27].is_empty())
                .then(|| values[27].parse())
                .transpose()?,
            appris: (!values[28].is_empty()).then(|| values[28].to_string()),
            ccds: (!values[29].is_empty()).then(|| values[29].to_string()),
            ensp: (!values[30].is_empty()).then(|| values[30].to_string()),
            swissprot: (!values[31].is_empty()).then(|| values[31].to_string()),
            trembl: (!values[32].is_empty()).then(|| values[32].to_string()),
            uniparc: (!values[33].is_empty()).then(|| values[33].to_string()),
            gene_pheno: (!values[34].is_empty()).then(|| values[34].to_string()),
            sift: (!values[35].is_empty())
                .then(|| -> Result<(String, f32), anyhow::Error> {
                    let tokens = values[35].split('(').collect::<Vec<_>>();
                    let mut tmp = tokens[1].chars();
                    tmp.next_back();
                    let score = tmp.as_str();
                    Ok((tokens[0].to_string(), score.parse::<f32>()?))
                })
                .transpose()?
                .map(|val| val.into()),
            polyphen: (!values[36].is_empty())
                .then(|| -> Result<(String, f32), anyhow::Error> {
                    let tokens = values[36].split('(').collect::<Vec<_>>();
                    let mut tmp = tokens[1].chars();
                    tmp.next_back();
                    let score = tmp.as_str();
                    Ok((tokens[0].to_string(), score.parse::<f32>()?))
                })
                .transpose()?
                .map(|val| val.into()),
            domains: (!values[37].is_empty())
                .then(|| {
                    let pairs = values[37].split('&').collect::<Vec<_>>();
                    pairs
                        .iter()
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
            hgvs_offset: (!values[38].is_empty()).then(|| values[38].to_string()),
            gmaf: (!values[39].is_empty())
                .then(|| values[39].split(':').next_back().unwrap().parse())
                .transpose()?,
            afr_maf: (!values[40].is_empty())
                .then(|| values[40].split(':').next_back().unwrap().parse())
                .transpose()?,
            amr_maf: (!values[41].is_empty())
                .then(|| values[41].split(':').next_back().unwrap().parse())
                .transpose()?,
            eas_maf: (!values[42].is_empty())
                .then(|| values[42].split(':').next_back().unwrap().parse())
                .transpose()?,
            eur_maf: (!values[43].is_empty())
                .then(|| values[43].split(':').next_back().unwrap().parse())
                .transpose()?,
            sas_maf: (!values[44].is_empty())
                .then(|| values[44].split(':').next_back().unwrap().parse())
                .transpose()?,
            aa_maf: (!values[45].is_empty())
                .then(|| values[45].split(':').next_back().unwrap().parse())
                .transpose()?,
            ea_maf: (!values[46].is_empty())
                .then(|| values[46].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_maf: (!values[47].is_empty())
                .then(|| values[47].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_adj_maf: (!values[48].is_empty())
                .then(|| values[48].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_afr_maf: (!values[49].is_empty())
                .then(|| values[49].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_amr_maf: (!values[50].is_empty())
                .then(|| values[50].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_eas_maf: (!values[51].is_empty())
                .then(|| values[51].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_fin_maf: (!values[52].is_empty())
                .then(|| values[52].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_nfe_maf: (!values[53].is_empty())
                .then(|| values[53].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_oth_maf: (!values[54].is_empty())
                .then(|| values[54].split(':').next_back().unwrap().parse())
                .transpose()?,
            exac_sas_maf: (!values[55].is_empty())
                .then(|| values[55].split(':').next_back().unwrap().parse())
                .transpose()?,
            clin_sig: (!values[56].is_empty()).then(|| values[56].to_string()),
            somatic: (!values[57].is_empty()).then(|| values[57].to_string()),
            pheno: (!values[58].is_empty()).then(|| values[58].to_string()),
            pubmed: (!values[59].is_empty()).then(|| values[59].to_string()),
            motif_name: (!values[60].is_empty()).then(|| values[60].to_string()),
            motif_pos: (!values[61].is_empty()).then(|| values[61].to_string()),
            high_inf_pos: (!values[62].is_empty()).then(|| values[62].to_string()),
            motif_score_change: (!values[63].is_empty()).then(|| values[63].to_string()),
            lof: (!values[64].is_empty()).then(|| values[64].to_string()),
            lof_filter: (!values[65].is_empty()).then(|| values[65].to_string()),
            lof_flags: (!values[66].is_empty()).then(|| values[66].to_string()),
            lof_info: (!values[67].is_empty()).then(|| values[67].to_string()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vep_from_string_snv() {
        let s = "\
        G|missense_variant|MODERATE|PCSK9|ENSG00000169174|Transcript|ENST00000302118|\
        protein_coding|1/12||ENST00000302118.5:c.89C>G|ENSP00000303208.5:p.Ala30Gly|379|89|30|\
        A/G|gCg/gGg||1||1||SNV|1|HGNC|20001|YES|||CCDS603.1|ENSP00000303208|Q8NBP7||\
        UPI00001615E1|1|deleterious_low_confidence(0.03)|possibly_damaging(0.583)|\
        Cleavage_site_(Signalp):SignalP-noTM&hmmpanther:PTHR10795&hmmpanther:PTHR10795:SF333\
        ||||||||||||||||||||||||||||||DE_NOVO_DONOR_PROB:0.146539915246404&MUTANT_DONOR_MES:\
        8.16693067332728&DE_NOVO_DONOR_POS:-119&INTRON_END:55509515&DE_NOVO_DONOR_MES_POS:-122&\
        INTRON_START:55505718&EXON_END:55505717&EXON_START:55505221&DE_NOVO_DONOR_MES:\
        -1.61109392005567";
        let vep = Vep::from_str(s).unwrap();

        insta::assert_yaml_snapshot!(vep);
    }

    #[test]
    fn test_vep_from_string_indel() {
        let s = "\
        A|frameshift_variant|HIGH|PCSK9|ENSG00000169174|Transcript|ENST00000452118|\
        protein_coding|4/6||ENST00000452118.2:c.547_548insA|ENSP00000401598.2:\
        p.Gly183GlufsTer23|627-628|547-548|183|G/EX|ggc/gAgc|rs527413419|1||1||insertion\
        |1|HGNC|20001|||||ENSP00000401598||B4DEZ9|UPI00017A6F55|1|||||A:0.0032||A:0.0113\
        |A:0.0014|A:0|A:0|A:0|||||||||||||||||||HC|||GERP_DIST:364.1444&BP_DIST:304&\
        PERCENTILE:0.885113268608414&DIST_FROM_LAST_EXON:210&50_BP_RULE:PASS&\
        PHYLOCSF_TOO_SHORT";
        let vep = Vep::from_str(s).unwrap();

        insta::assert_yaml_snapshot!(vep);
    }
}
