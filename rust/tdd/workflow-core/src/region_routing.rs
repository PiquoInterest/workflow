pub const ALL_REGIONS: [&str; 19] = [
    "iad1", "arn1", "bom1", "cdg1", "cle1", "cpt1", "dub1", "fra1", "gru1", "hkg1", "hnd1", "icn1",
    "kix1", "lhr1", "pdx1", "sfo1", "sin1", "syd1", "yul1",
];

pub const DETAILED_REGIONS: [&str; 3] = ["iad1", "sfo1", "fra1"];

fn pending<T>() -> T {
    panic!("TDD RED: packages/core/e2e/e2e-region.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionProbeObservation {
    pub run_id: String,
    pub is_tagged: bool,
    pub tagged_region: String,
    pub label: String,
    pub workflow_region: String,
    pub step_region: String,
    pub server_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplicitRegionProbeObservation {
    pub started_in_region: String,
    pub run: RegionProbeObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossRegionStreamObservation {
    pub run_id: String,
    pub writer_tagged_region: String,
    pub tail_index_before_read: usize,
    pub server_status_before_read: String,
    pub reader_region: String,
    pub reader_tail_index: usize,
    pub return_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRegionObservation {
    pub run_id: String,
    pub is_tagged: bool,
    pub tagged_region: String,
    pub hook_run_id: String,
    pub metadata_custom_data: String,
    pub payload_messages: Vec<String>,
    pub server_status: String,
}

pub fn start_explicit_region_probe(region: &str, label: &str) -> RegionProbeObservation {
    let _ = (region, label);
    pending()
}

pub fn start_concurrent_region_probes(
    regions: &[&str],
    runs_per_region: usize,
) -> Vec<RegionProbeObservation> {
    let _ = (regions, runs_per_region);
    pending()
}

pub fn start_all_region_probes(regions: &[&str]) -> Vec<RegionProbeObservation> {
    let _ = regions;
    pending()
}

pub fn start_implicit_region_probe(region: &str, label: &str) -> ImplicitRegionProbeObservation {
    let _ = (region, label);
    pending()
}

pub fn observe_cross_region_stream(
    writer_region: &str,
    reader_region: &str,
    chunk_count: usize,
) -> CrossRegionStreamObservation {
    let _ = (writer_region, reader_region, chunk_count);
    pending()
}

pub fn resume_hook_in_region(region: &str, label: &str) -> HookRegionObservation {
    let _ = (region, label);
    pending()
}
