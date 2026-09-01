pub fn get_vercel_dashboard_url(
    team_slug: &str,
    project_name: &str,
    resource: &str,
    id: Option<&str>,
    environment: Option<&str>,
) -> String {
    let _ = (team_slug, project_name, resource, id, environment);
    panic!("TDD RED: packages/cli/src/lib/inspect/vercel-api.test.ts implementation pending")
}
