use workflow_core_tdd::region_routing::start_implicit_region_probe;

use super::support::assert_region_probe;

macro_rules! implicit_case {
    ($name:ident, $region:literal) => {
        #[test]
        fn $name() {
            let label = concat!("e2e-implicit-", $region);
            let observation = start_implicit_region_probe($region, label);
            assert_eq!(observation.started_in_region, $region);
            assert_region_probe(&observation.run, $region, label);
        }
    };
}

implicit_case!(implicit_iad1_route_mints_iad1_tag, "iad1");
implicit_case!(implicit_arn1_route_mints_arn1_tag, "arn1");
implicit_case!(implicit_bom1_route_mints_bom1_tag, "bom1");
implicit_case!(implicit_cdg1_route_mints_cdg1_tag, "cdg1");
implicit_case!(implicit_cle1_route_mints_cle1_tag, "cle1");
implicit_case!(implicit_cpt1_route_mints_cpt1_tag, "cpt1");
implicit_case!(implicit_dub1_route_mints_dub1_tag, "dub1");
implicit_case!(implicit_fra1_route_mints_fra1_tag, "fra1");
implicit_case!(implicit_gru1_route_mints_gru1_tag, "gru1");
implicit_case!(implicit_hkg1_route_mints_hkg1_tag, "hkg1");
implicit_case!(implicit_hnd1_route_mints_hnd1_tag, "hnd1");
implicit_case!(implicit_icn1_route_mints_icn1_tag, "icn1");
implicit_case!(implicit_kix1_route_mints_kix1_tag, "kix1");
implicit_case!(implicit_lhr1_route_mints_lhr1_tag, "lhr1");
implicit_case!(implicit_pdx1_route_mints_pdx1_tag, "pdx1");
implicit_case!(implicit_sfo1_route_mints_sfo1_tag, "sfo1");
implicit_case!(implicit_sin1_route_mints_sin1_tag, "sin1");
implicit_case!(implicit_syd1_route_mints_syd1_tag, "syd1");
implicit_case!(implicit_yul1_route_mints_yul1_tag, "yul1");
