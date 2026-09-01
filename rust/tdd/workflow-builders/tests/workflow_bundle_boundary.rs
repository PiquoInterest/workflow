use workflow_builders_tdd::workflow_bundle_boundary::workflow_bundle_inputs;

fn assert_no_zod_inputs(inputs: &[String]) {
    assert!(
        inputs
            .iter()
            .all(|input| !input.replace('\\', "/").contains("/node_modules/zod/"))
    );
}

#[test]
fn does_not_bundle_world_schemas_into_a_minimal_workflow() {
    let observation = workflow_bundle_inputs(
        r#"export async function minimal() { "use workflow"; return 1; }"#,
    )
    .unwrap();

    assert_no_zod_inputs(&observation.inputs);
}

#[test]
fn does_not_bundle_world_schemas_for_core_workflow_apis() {
    let observation = workflow_bundle_inputs(
        r#"
      import { createHook, setAttributes } from '@workflow/core';

      async function basicStep(value: number) {
        "use step";
        return value + 1;
      }

      export async function realisticWorkflow() {
        "use workflow";
        await setAttributes({ phase: 'started' });
        const hook = createHook<number>();
        return basicStep(await hook);
      }
    "#,
    )
    .unwrap();

    assert_no_zod_inputs(&observation.inputs);
}
