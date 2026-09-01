use std::collections::BTreeSet;

use workflow_core_tdd::bench_rtt_stats::{
    BenchRttSummary, CdvArrival, RTT_HIST_EDGES_MS, RTT_INDEX_BUCKETS, RTT_PROGRESS_BINS,
    RTT_SIZE_BIN_EDGES_BYTES, RatePoint, RttIndexBucket, SizeRttSample, compute_cdv,
    histogram_rtt_samples, merge_mean_profiles, merge_rtt_summaries, progress_profile,
    rtt_index_bucket, rtt_size_bin, size_profile, steady_rate, summarize_delay_tail,
    summarize_rtt_samples,
};

fn hist_with(value: f64, count: usize) -> Vec<usize> {
    let mut hist = vec![0; RTT_HIST_EDGES_MS.len() + 1];
    let mut bin = 0;
    while bin < RTT_HIST_EDGES_MS.len() && value >= RTT_HIST_EDGES_MS[bin] {
        bin += 1;
    }
    hist[bin] = count;
    hist
}

fn options(values: impl IntoIterator<Item = f64>) -> Vec<Option<f64>> {
    values.into_iter().map(Some).collect()
}

fn clumped() -> Vec<CdvArrival> {
    vec![
        CdvArrival {
            seq: 0,
            written_at: 1_000.0,
            read_at: 1_030.0,
        },
        CdvArrival {
            seq: 1,
            written_at: 1_010.0,
            read_at: 1_030.0,
        },
        CdvArrival {
            seq: 2,
            written_at: 1_020.0,
            read_at: 1_031.0,
        },
        CdvArrival {
            seq: 3,
            written_at: 1_030.0,
            read_at: 1_060.0,
        },
        CdvArrival {
            seq: 4,
            written_at: 1_040.0,
            read_at: 1_060.0,
        },
        CdvArrival {
            seq: 5,
            written_at: 1_050.0,
            read_at: 1_061.0,
        },
    ]
}

fn summary() -> BenchRttSummary {
    BenchRttSummary {
        count: 10,
        best: 1.0,
        avg: 5.0,
        hist: hist_with(5.0, 10),
        p50: 5.0,
        p75: 6.0,
        p90: 8.0,
        p99: 9.0,
    }
}

#[test]
fn index_bucket_boundaries_match_stream_open_warmup_and_steady_state() {
    assert_eq!(rtt_index_bucket(0), RttIndexBucket::SeqZero);
    assert_eq!(rtt_index_bucket(1), RttIndexBucket::SeqOneThroughTwenty);
    assert_eq!(
        rtt_index_bucket(20),
        RttIndexBucket::SeqOneThroughTwenty
    );
    assert_eq!(rtt_index_bucket(21), RttIndexBucket::SeqTwentyOnePlus);
    assert_eq!(
        rtt_index_bucket(299),
        RttIndexBucket::SeqTwentyOnePlus
    );
}

#[test]
fn every_index_bucket_is_a_declared_key() {
    for seq in 0..300 {
        assert!(RTT_INDEX_BUCKETS.contains(&rtt_index_bucket(seq)));
    }
}

#[test]
fn progress_profile_bins_by_stream_fraction_independent_of_chunk_count() {
    let rtts = options((0..300).map(f64::from));
    let profile = progress_profile(&rtts);
    assert_eq!(profile.counts, vec![30; RTT_PROGRESS_BINS]);
    assert_eq!(profile.total_ms[0], 435.0);
    assert_eq!(profile.total_ms[RTT_PROGRESS_BINS - 1], 8_535.0);

    let small_input = [Some(5.0); 20];
    let small = progress_profile(&small_input);
    assert_eq!(small.counts, vec![2; RTT_PROGRESS_BINS]);
}

#[test]
fn progress_profile_skips_sparse_entries_defensively() {
    let mut rtts = vec![None; 100];
    rtts[0] = Some(7.0);
    rtts[99] = Some(9.0);
    let profile = progress_profile(&rtts);
    assert_eq!(profile.counts.iter().sum::<usize>(), 2);
    assert_eq!(profile.total_ms[0], 7.0);
    assert_eq!(profile.total_ms[RTT_PROGRESS_BINS - 1], 9.0);
}

#[test]
fn merge_mean_profiles_returns_none_without_profiles_and_sums_otherwise() {
    assert_eq!(merge_mean_profiles(&[]), None);
    assert_eq!(merge_mean_profiles(&[None]), None);

    let a_input = [Some(10.0); 10];
    let b_input = [Some(30.0); 10];
    let a = progress_profile(&a_input);
    let b = progress_profile(&b_input);
    let merged = merge_mean_profiles(&[Some(a), None, Some(b)]).unwrap();
    assert_eq!(merged.counts, vec![2; RTT_PROGRESS_BINS]);
    assert_eq!(merged.total_ms, vec![40.0; RTT_PROGRESS_BINS]);
}

#[test]
fn size_bins_use_doubling_edges() {
    assert_eq!(rtt_size_bin(100), 0);
    assert_eq!(rtt_size_bin(255), 0);
    assert_eq!(rtt_size_bin(256), 1);
    assert_eq!(rtt_size_bin(1_024), 3);
    assert_eq!(rtt_size_bin(8_192), RTT_SIZE_BIN_EDGES_BYTES.len());
    assert_eq!(rtt_size_bin(20_000), RTT_SIZE_BIN_EDGES_BYTES.len());
}

#[test]
fn sweep_pad_ladder_occupies_every_size_bin_once() {
    let bins = [160, 400, 760, 1_460, 3_060, 6_060, 12_060]
        .into_iter()
        .map(rtt_size_bin)
        .collect::<BTreeSet<_>>();
    assert_eq!(bins.len(), RTT_SIZE_BIN_EDGES_BYTES.len() + 1);
}

#[test]
fn size_profile_accumulates_count_and_total_rtt_per_bin() {
    let profile = size_profile(&[
        SizeRttSample {
            bytes: 160,
            rtt_ms: 10.0,
        },
        SizeRttSample {
            bytes: 200,
            rtt_ms: 20.0,
        },
        SizeRttSample {
            bytes: 12_060,
            rtt_ms: 50.0,
        },
    ]);
    assert_eq!(profile.counts[0], 2);
    assert_eq!(profile.total_ms[0], 30.0);
    assert_eq!(profile.counts[RTT_SIZE_BIN_EDGES_BYTES.len()], 1);
    assert_eq!(profile.total_ms[RTT_SIZE_BIN_EDGES_BYTES.len()], 50.0);
    assert_eq!(profile.counts.iter().sum::<usize>(), 3);
}

#[test]
fn histogram_bins_are_half_open_with_underflow_and_overflow_bins() {
    assert_eq!(histogram_rtt_samples(&[0.0, 0.5])[0], 2);
    let at_edge = histogram_rtt_samples(&[1.0]);
    assert_eq!(at_edge[0], 0);
    assert_eq!(at_edge[1], 1);
    let overflow = histogram_rtt_samples(&[5_000.0, 60_000.0]);
    assert_eq!(overflow[RTT_HIST_EDGES_MS.len()], 2);
}

#[test]
fn histogram_counts_sum_to_the_sample_count() {
    let samples = [0.0, 1.0, 3.0, 7.0, 59.0, 128.0, 438.0, 1_229.0, 9_999.0];
    let hist = histogram_rtt_samples(&samples);
    assert_eq!(hist.len(), RTT_HIST_EDGES_MS.len() + 1);
    assert_eq!(hist.iter().sum::<usize>(), samples.len());
}

#[test]
fn summarize_rtt_samples_returns_none_for_an_empty_bucket() {
    assert_eq!(summarize_rtt_samples(&[]), None);
}

#[test]
fn one_rtt_sample_collapses_every_stat_to_that_value() {
    assert_eq!(
        summarize_rtt_samples(&[7.0]),
        Some(BenchRttSummary {
            count: 1,
            best: 7.0,
            avg: 7.0,
            hist: hist_with(7.0, 1),
            p50: 7.0,
            p75: 7.0,
            p90: 7.0,
            p99: 7.0,
        })
    );
}

#[test]
fn rtt_percentiles_use_nearest_rank_via_ceiling() {
    let samples = (1..=100).rev().map(f64::from).collect::<Vec<_>>();
    assert_eq!(
        summarize_rtt_samples(&samples),
        Some(BenchRttSummary {
            count: 100,
            best: 1.0,
            avg: 50.5,
            hist: histogram_rtt_samples(&samples),
            p50: 50.0,
            p75: 75.0,
            p90: 90.0,
            p99: 99.0,
        })
    );
}

#[test]
fn rtt_summary_rounds_to_one_decimal_millisecond() {
    let summary = summarize_rtt_samples(&[1.0, 2.0, 2.44]).unwrap();
    assert_eq!(summary.avg, 1.8);
    assert_eq!(summary.p99, 2.4);
}

#[test]
fn summarize_delay_tail_returns_none_without_samples() {
    assert_eq!(summarize_delay_tail(&[]), None);
}

#[test]
fn delay_tail_max_catches_a_single_stall_hidden_by_p99() {
    let mut samples = vec![2.0; 299];
    samples.push(800.0);
    let tail = summarize_delay_tail(&samples).unwrap();
    assert_eq!(tail.max_ms, 800.0);
    assert_eq!(tail.p99_ms, 2.0);
    assert_eq!(tail.count, 300);
    assert_eq!(tail.avg_ms, 4.7);
}

#[test]
fn steady_rate_returns_none_when_the_window_cannot_define_a_rate() {
    assert_eq!(steady_rate(&[]), None);
    assert_eq!(
        steady_rate(&[RatePoint {
            at_ms: 0.0,
            bytes: 100,
        }]),
        None
    );
    assert_eq!(
        steady_rate(&[
            RatePoint {
                at_ms: 5.0,
                bytes: 1,
            },
            RatePoint {
                at_ms: 5.0,
                bytes: 1,
            },
        ]),
        None
    );
}

#[test]
fn steady_rate_computes_chunks_and_kib_per_second_over_the_trimmed_window() {
    let points = (0..100)
        .map(|index| RatePoint {
            at_ms: f64::from(index * 10),
            bytes: 1_024,
        })
        .collect::<Vec<_>>();
    let rate = steady_rate(&points).unwrap();
    assert_eq!(rate.window_chunks, 80);
    assert_eq!(rate.chunks_per_sec, 100.0);
    assert_eq!(rate.kib_per_sec, 100.0);
}

#[test]
fn steady_rate_trimming_excludes_warmup_and_drain() {
    let mut points = vec![RatePoint {
        at_ms: 0.0,
        bytes: 100,
    }];
    points.extend((0..20).map(|index| RatePoint {
        at_ms: 1_000.0 + f64::from(index * 10),
        bytes: 100,
    }));
    points.push(RatePoint {
        at_ms: 10_000.0,
        bytes: 100,
    });
    assert!(steady_rate(&points).unwrap().chunks_per_sec > 90.0);
}

#[test]
fn clumped_delivery_reads_as_negative_catch_up_and_positive_stalls() {
    let cdv = compute_cdv(&clumped());
    assert_eq!(cdv.cdv_ms, vec![-10.0, -9.0, 19.0, -10.0, -9.0]);
    assert_eq!(cdv.skipped_pairs, 0);
}

#[test]
fn cdv_matches_the_telescoping_ctt_identity() {
    let arrivals = clumped();
    let ctt = arrivals
        .iter()
        .map(|arrival| arrival.read_at - arrival.written_at)
        .collect::<Vec<_>>();
    let cdv = compute_cdv(&arrivals);
    let expected = ctt
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect::<Vec<_>>();
    assert_eq!(cdv.cdv_ms, expected);
    assert_eq!(
        cdv.cdv_ms.iter().sum::<f64>(),
        ctt[ctt.len() - 1] - ctt[0]
    );
}

#[test]
fn cdv_is_immune_to_a_constant_reader_clock_offset() {
    let mut skewed = clumped();
    for arrival in &mut skewed {
        arrival.read_at -= 5_000.0;
    }
    assert_eq!(
        compute_cdv(&skewed).cdv_ms,
        compute_cdv(&clumped()).cdv_ms
    );
}

#[test]
fn positive_cdv_is_indexed_by_the_later_sequence_and_padded() {
    let cdv = compute_cdv(&clumped());
    assert_eq!(cdv.positive_by_seq.len(), 6);
    assert_eq!(cdv.positive_by_seq[3], Some(19.0));
    assert_eq!(
        cdv.positive_by_seq
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        vec![19.0]
    );
}

#[test]
fn cdv_counts_duplicates_reorders_and_non_adjacent_pairs() {
    let arrivals = [
        CdvArrival {
            seq: 0,
            written_at: 1_000.0,
            read_at: 1_030.0,
        },
        CdvArrival {
            seq: 2,
            written_at: 1_020.0,
            read_at: 1_050.0,
        },
        CdvArrival {
            seq: 1,
            written_at: 1_010.0,
            read_at: 1_051.0,
        },
        CdvArrival {
            seq: 1,
            written_at: 1_010.0,
            read_at: 1_052.0,
        },
    ];
    let cdv = compute_cdv(&arrivals);
    assert_eq!(cdv.cdv_ms, Vec::<f64>::new());
    assert_eq!(cdv.duplicate_seqs, 1);
    assert_eq!(cdv.reordered_arrivals, 1);
    assert_eq!(cdv.skipped_pairs, 3);
}

#[test]
fn one_chunk_has_no_cdv_pair() {
    let cdv = compute_cdv(&[CdvArrival {
        seq: 0,
        written_at: 1_000.0,
        read_at: 1_030.0,
    }]);
    assert_eq!(cdv.cdv_ms, Vec::<f64>::new());
    assert_eq!(cdv.skipped_pairs, 0);
}

#[test]
fn merge_rtt_summaries_returns_none_when_no_iteration_produced_the_bucket() {
    assert_eq!(merge_rtt_summaries(&[]), None);
    assert_eq!(merge_rtt_summaries(&[None, None]), None);
}

#[test]
fn one_rtt_summary_passes_through_unchanged() {
    let value = summary();
    assert_eq!(
        merge_rtt_summaries(&[None, Some(value.clone())]),
        Some(value)
    );
}

#[test]
fn merged_rtt_count_sums_best_is_minimum_and_average_is_weighted() {
    let mut first = summary();
    first.count = 10;
    first.best = 2.0;
    first.avg = 10.0;
    let mut second = summary();
    second.count = 30;
    second.best = 1.0;
    second.avg = 2.0;

    let merged = merge_rtt_summaries(&[Some(first), Some(second)]).unwrap();
    assert_eq!(merged.count, 40);
    assert_eq!(merged.best, 1.0);
    assert_eq!(merged.avg, 4.0);
}

#[test]
fn merged_histograms_sum_element_by_element() {
    let mut first = summary();
    first.count = 10;
    first.hist = hist_with(5.0, 10);
    let mut second = summary();
    second.count = 30;
    second.hist = hist_with(128.0, 30);

    let merged = merge_rtt_summaries(&[Some(first), Some(second)]).unwrap();
    let mut expected = hist_with(5.0, 10);
    let other = hist_with(128.0, 30);
    for (left, right) in expected.iter_mut().zip(other) {
        *left += right;
    }
    assert_eq!(merged.hist, expected);
    assert_eq!(merged.hist.iter().sum::<usize>(), 40);
}

#[test]
fn merged_percentiles_use_percentile_of_percentiles() {
    let summaries = (1..=10)
        .map(|index| {
            let mut value = summary();
            value.p50 = f64::from(index);
            value.p90 = f64::from(index * 10);
            value.p99 = f64::from(index * 100);
            Some(value)
        })
        .collect::<Vec<_>>();
    let merged = merge_rtt_summaries(&summaries).unwrap();
    assert_eq!(merged.p50, 5.0);
    assert_eq!(merged.p90, 90.0);
    assert_eq!(merged.p99, 1_000.0);
}
