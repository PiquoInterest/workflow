use workflow_core_tdd::local_build::{
    ALL_LOCAL_BUILD_PROJECTS, BoundedCommandOutput, COMMAND_OUTPUT_LIMIT_BYTES, CommandExit,
    CommandFailure, FileReadError, LOCAL_BUILD_TIMEOUT_MS, LocalBuildOptions, LocalBuildProject,
    OutputStream, SOURCE_MAP_WARNING, WorldTarget, gate_optional_file, local_build_plan,
    run_local_build_case,
};

fn assert_project_contract(project: LocalBuildProject) {
    let observation = run_local_build_case(project, LocalBuildOptions::default());
    assert_eq!(observation.plan.project, project);
    assert!(observation.build_succeeded);
    assert!(!observation.output.truncated);
    assert!(
        observation
            .plan
            .forbidden_output_fragments
            .iter()
            .all(|fragment| !observation.output.combined_text().contains(fragment))
    );
    if observation.plan.diagnostics_manifest_path.is_some() {
        assert!(observation.diagnostics_manifest_found);
    }
    if observation.plan.esm_step_registration_path.is_some() {
        assert!(observation.esm_bundle_uses_native_import_meta);
    }
    if observation.plan.forbidden_legacy_step_route_path.is_some() {
        assert!(observation.legacy_step_route_absent);
    }
    if observation.plan.source_map_fixture.cleanup_in_finally {
        assert!(observation.source_map_fixture_cleaned_up);
    }
}

macro_rules! project_case {
    ($name:ident, $project:expr) => {
        #[test]
        fn $name() {
            assert_project_contract($project);
        }
    };
}

project_case!(example_builds_without_errors, LocalBuildProject::Example);
project_case!(
    next_webpack_builds_without_errors,
    LocalBuildProject::NextWebpack
);
project_case!(
    next_turbopack_builds_without_errors,
    LocalBuildProject::NextTurbopack
);
project_case!(nitro_builds_without_errors, LocalBuildProject::Nitro);
project_case!(vite_builds_without_errors, LocalBuildProject::Vite);
project_case!(
    sveltekit_builds_without_errors,
    LocalBuildProject::SvelteKit
);
project_case!(nuxt_builds_without_errors, LocalBuildProject::Nuxt);
project_case!(hono_builds_without_errors, LocalBuildProject::Hono);
project_case!(express_builds_without_errors, LocalBuildProject::Express);
project_case!(fastify_builds_without_errors, LocalBuildProject::Fastify);
project_case!(nest_builds_without_errors, LocalBuildProject::Nest);
project_case!(astro_builds_without_errors, LocalBuildProject::Astro);
project_case!(
    tanstack_start_builds_without_errors,
    LocalBuildProject::TanstackStart
);

#[test]
fn project_matrix_matches_the_source_parameterization() {
    assert_eq!(
        ALL_LOCAL_BUILD_PROJECTS.map(LocalBuildProject::as_str),
        [
            "example",
            "nextjs-webpack",
            "nextjs-turbopack",
            "nitro",
            "vite",
            "sveltekit",
            "nuxt",
            "hono",
            "express",
            "fastify",
            "nest",
            "astro",
            "tanstack-start",
        ]
    );
}

#[test]
fn every_command_is_bounded_and_kills_its_process_group_on_timeout() {
    for project in ALL_LOCAL_BUILD_PROJECTS {
        let plan = local_build_plan(project, LocalBuildOptions::default());
        assert_eq!(plan.build.program, "pnpm");
        assert_eq!(plan.build.args, vec!["build".to_owned()]);
        assert_eq!(plan.build.timeout_ms, LOCAL_BUILD_TIMEOUT_MS);
        assert_eq!(plan.build.output_limit_bytes, COMMAND_OUTPUT_LIMIT_BYTES);
        assert!(plan.build.kill_process_group_on_timeout);
        if let Some(preflight) = plan.preflight {
            assert_eq!(preflight.timeout_ms, LOCAL_BUILD_TIMEOUT_MS);
            assert_eq!(preflight.output_limit_bytes, COMMAND_OUTPUT_LIMIT_BYTES);
            assert!(preflight.kill_process_group_on_timeout);
        }
    }
}

#[test]
fn sveltekit_runs_the_package_import_preflight_before_building() {
    let plan = local_build_plan(LocalBuildProject::SvelteKit, LocalBuildOptions::default());
    let preflight = plan.preflight.expect("sveltekit preflight");
    assert_eq!(preflight.program, "current-node");
    assert_eq!(preflight.args[0], "-e");
    assert!(preflight.args[1].contains("import('workflow/sveltekit')"));
    assert!(preflight.args[1].contains("workflow/sveltekit import ok"));
}

#[test]
fn diagnostics_paths_follow_the_world_target_and_builder() {
    let local = LocalBuildOptions::default();
    assert_eq!(
        local_build_plan(LocalBuildProject::Example, local).diagnostics_manifest_path,
        Some(".vercel/output/diagnostics/workflows-manifest.json")
    );
    assert_eq!(
        local_build_plan(LocalBuildProject::NextWebpack, local).diagnostics_manifest_path,
        Some(".next/diagnostics/workflows-manifest.json")
    );
    assert_eq!(
        local_build_plan(LocalBuildProject::Vite, local).diagnostics_manifest_path,
        None
    );

    let vercel = LocalBuildOptions {
        world_target: WorldTarget::Vercel,
        ci: false,
    };
    for project in ALL_LOCAL_BUILD_PROJECTS {
        assert_eq!(
            local_build_plan(project, vercel).diagnostics_manifest_path,
            Some(".vercel/output/diagnostics/workflows-manifest.json")
        );
    }
}

#[test]
fn example_requires_native_esm_import_meta_and_removes_the_legacy_route() {
    let plan = local_build_plan(LocalBuildProject::Example, LocalBuildOptions::default());
    assert_eq!(
        plan.esm_step_registration_path,
        Some(".vercel/output/functions/.well-known/workflow/v1/flow.func/__step_registrations.mjs")
    );
    assert_eq!(
        plan.forbidden_legacy_step_route_path,
        Some(".vercel/output/functions/.well-known/workflow/v1/step.func")
    );
}

#[test]
fn next_builders_forbid_the_legacy_step_route() {
    for project in [
        LocalBuildProject::NextWebpack,
        LocalBuildProject::NextTurbopack,
    ] {
        let plan = local_build_plan(project, LocalBuildOptions::default());
        assert_eq!(
            plan.forbidden_legacy_step_route_path,
            Some("app/.well-known/workflow/v1/step")
        );
    }
}

#[test]
fn turbopack_fixture_cleanup_is_finally_guarded_but_ci_can_preserve_built_inputs() {
    let local = local_build_plan(
        LocalBuildProject::NextTurbopack,
        LocalBuildOptions::default(),
    );
    assert!(local.source_map_fixture.setup_before_build);
    assert!(!local.source_map_fixture.preserve_after_build);
    assert!(local.source_map_fixture.cleanup_in_finally);

    let ci = local_build_plan(
        LocalBuildProject::NextTurbopack,
        LocalBuildOptions {
            world_target: WorldTarget::Local,
            ci: true,
        },
    );
    assert!(ci.source_map_fixture.setup_before_build);
    assert!(ci.source_map_fixture.preserve_after_build);
    assert!(!ci.source_map_fixture.cleanup_in_finally);
}

#[test]
fn build_output_rejects_generic_errors_and_source_map_warnings() {
    let plan = local_build_plan(LocalBuildProject::Astro, LocalBuildOptions::default());
    assert_eq!(
        plan.forbidden_output_fragments,
        vec!["Error:", SOURCE_MAP_WARNING]
    );
}

#[test]
fn output_capture_preserves_stream_order_without_exceeding_the_shared_byte_cap() {
    let mut output = BoundedCommandOutput::new(8);
    output.append(OutputStream::Stdout, b"abc");
    output.append(OutputStream::Stderr, b"DEF");
    output.append(OutputStream::Stdout, b"ghi");

    assert_eq!(output.stdout, b"abcgh");
    assert_eq!(output.stderr, b"DEF");
    assert_eq!(output.combined, b"abcDEFgh");
    assert_eq!(output.seen_bytes, 9);
    assert_eq!(output.accepted_bytes, 8);
    assert!(output.truncated);
    assert_eq!(output.stdout_text(), "abcgh");
    assert_eq!(output.stderr_text(), "DEF");
    assert_eq!(output.combined_text(), "abcDEFgh");
}

#[test]
fn zero_byte_output_cap_never_buffers_child_output() {
    let mut output = BoundedCommandOutput::new(0);
    output.append(OutputStream::Stderr, b"unbounded build noise");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(output.combined.is_empty());
    assert_eq!(output.seen_bytes, 21);
    assert_eq!(output.accepted_bytes, 0);
    assert!(output.truncated);
}

#[test]
fn command_failure_names_the_argv_exit_and_bounded_combined_output() {
    let mut output = BoundedCommandOutput::new(64);
    output.append(OutputStream::Stdout, b"building\n");
    output.append(OutputStream::Stderr, b"failed\n");
    let failure = CommandFailure {
        program: "pnpm".to_owned(),
        args: vec!["build".to_owned()],
        exit: CommandExit::Code(2),
        output,
    };
    assert_eq!(
        failure.to_string(),
        "Command \"pnpm build\" failed with exit code 2\nbuilding\nfailed\n"
    );
}

#[test]
fn signal_failure_is_not_misreported_as_an_exit_code() {
    assert_eq!(
        CommandExit::Signal("SIGKILL".to_owned()).to_string(),
        "signal SIGKILL"
    );
}

#[test]
fn optional_file_gate_skips_only_not_found_and_propagates_other_io_failures() {
    assert_eq!(
        gate_optional_file::<String>(Err(FileReadError::NotFound)),
        Ok(None)
    );
    assert_eq!(
        gate_optional_file(Ok("bundle".to_owned())),
        Ok(Some("bundle".to_owned()))
    );
    assert_eq!(
        gate_optional_file::<String>(Err(FileReadError::Io("permission denied".to_owned()))),
        Err(FileReadError::Io("permission denied".to_owned()))
    );
}

#[test]
fn all_artifact_reads_are_required_to_fail_closed() {
    for project in ALL_LOCAL_BUILD_PROJECTS {
        assert!(local_build_plan(project, LocalBuildOptions::default()).file_reads_fail_closed);
    }
}
