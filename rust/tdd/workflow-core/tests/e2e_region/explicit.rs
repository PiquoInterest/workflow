use std::collections::BTreeSet;

use workflow_core_tdd::region_routing::{
    ALL_REGIONS, DETAILED_REGIONS, start_all_region_probes, start_concurrent_region_probes,
    start_explicit_region_probe,
};

use super::support::assert_region_probe;

macro_rules! explicit_case {
    ($name:ident, $region:literal) => {
        #[test]
        fn $name() {
            let label = concat!("e2e-explicit-", $region);
            let observation = start_explicit_region_probe($region, label);
            assert_region_probe(&observation, $region, label);
        }
    };
}

explicit_case!(explicit_iad1_start_is_tagged_and_executes_nearby, "iad1");
explicit_case!(explicit_sfo1_start_is_tagged_and_executes_nearby, "sfo1");
explicit_case!(explicit_fra1_start_is_tagged_and_executes_nearby, "fra1");

#[test]
fn concurrent_starts_across_detailed_regions_remain_isolated() {
    let observations = start_concurrent_region_probes(&DETAILED_REGIONS, 3);
    assert_eq!(observations.len(), DETAILED_REGIONS.len() * 3);

    let mut run_ids = BTreeSet::new();
    for region in DETAILED_REGIONS {
        for index in 0..3 {
            let label = format!("e2e-concurrent-{region}-{index}");
            let observation = observations
                .iter()
                .find(|candidate| candidate.label == label)
                .unwrap_or_else(|| panic!("missing observation for {label}"));
            assert_region_probe(observation, region, &label);
            assert!(run_ids.insert(observation.run_id.clone()));
        }
    }
}

#[test]
fn every_provisioned_region_executes_and_completes_a_tagged_run() {
    let observations = start_all_region_probes(&ALL_REGIONS);
    assert_eq!(observations.len(), ALL_REGIONS.len());

    for region in ALL_REGIONS {
        let label = format!("e2e-all-{region}");
        let observation = observations
            .iter()
            .find(|candidate| candidate.label == label)
            .unwrap_or_else(|| panic!("missing observation for {region}"));
        assert_region_probe(observation, region, &label);
    }
}
