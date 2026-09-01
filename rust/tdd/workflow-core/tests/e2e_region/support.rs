use workflow_core_tdd::region_routing::RegionProbeObservation;

pub fn allowed_execution_regions(region: &str) -> Vec<&'static str> {
    match region {
        "arn1" => vec!["arn1", "fra1", "dub1"],
        "bom1" => vec!["bom1", "sin1", "hkg1"],
        "cdg1" => vec!["cdg1", "lhr1", "fra1"],
        "cle1" => vec!["cle1", "iad1", "pdx1"],
        "cpt1" => vec!["cpt1", "fra1", "lhr1"],
        "dub1" => vec!["dub1", "lhr1", "fra1"],
        "fra1" => vec!["fra1", "cdg1", "dub1"],
        "gru1" => vec!["gru1", "iad1", "cle1"],
        "hkg1" => vec!["hkg1", "sin1", "syd1"],
        "hnd1" => vec!["hnd1", "kix1", "sin1"],
        "iad1" => vec!["iad1", "cle1", "pdx1"],
        "icn1" => vec!["icn1", "kix1", "syd1", "hnd1"],
        "kix1" => vec!["kix1", "hnd1", "syd1"],
        "lhr1" => vec!["lhr1", "cdg1", "arn1"],
        "pdx1" => vec!["pdx1", "sfo1", "cle1"],
        "sfo1" => vec!["sfo1", "pdx1", "cle1"],
        "sin1" => vec!["sin1", "hkg1", "syd1"],
        "syd1" => vec!["syd1", "sin1", "hkg1"],
        "yul1" => vec!["yul1", "iad1", "pdx1"],
        _ => vec![],
    }
}

pub fn assert_region_probe(
    observation: &RegionProbeObservation,
    intended_region: &str,
    label: &str,
) {
    assert!(observation.run_id.starts_with("wrun_"));
    assert!(observation.is_tagged);
    assert_eq!(observation.tagged_region, intended_region);
    assert_eq!(observation.label, label);

    let allowed = allowed_execution_regions(intended_region);
    assert!(
        allowed.contains(&observation.workflow_region.as_str()),
        "workflow executed in {}, tagged {}",
        observation.workflow_region,
        intended_region
    );
    assert!(
        allowed.contains(&observation.step_region.as_str()),
        "step executed in {}, tagged {}",
        observation.step_region,
        intended_region
    );
    assert_eq!(observation.server_status, "completed");
}
