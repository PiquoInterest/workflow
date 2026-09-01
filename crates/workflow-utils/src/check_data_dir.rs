use std::fs;
use std::path::{Component, Path, PathBuf};

pub const POSSIBLE_WORKFLOW_DATA_PATHS: &[&str] =
    &[".next/workflow-data", ".workflow-data", "workflow-data"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirContext {
    pub cwd: PathBuf,
    pub home: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDataDirResult {
    pub data_dir: Option<PathBuf>,
    pub project_dir: PathBuf,
    pub short_name: String,
    pub error: Option<String>,
}

#[must_use]
pub fn find_workflow_data_dir(input: &str, context: &DataDirContext) -> WorkflowDataDirResult {
    let absolute_input = to_absolute_path(input, context);

    if !is_directory(&absolute_input) {
        return WorkflowDataDirResult {
            data_dir: None,
            project_dir: absolute_input.clone(),
            short_name: get_dir_short_name(&absolute_input),
            error: Some("Folder does not exist".to_owned()),
        };
    }

    if let Some(project_dir) = project_for_data_dir(&absolute_input) {
        return WorkflowDataDirResult {
            data_dir: Some(absolute_input),
            short_name: get_dir_short_name(&project_dir),
            project_dir,
            error: None,
        };
    }

    let mut current_dir = absolute_input.clone();
    loop {
        if let Some(data_dir) = first_data_dir_in(&current_dir) {
            return WorkflowDataDirResult {
                data_dir: Some(data_dir),
                project_dir: current_dir.clone(),
                short_name: get_dir_short_name(&current_dir),
                error: None,
            };
        }

        let Some(parent) = current_dir.parent() else {
            break;
        };
        if parent == current_dir {
            break;
        }
        current_dir = parent.to_path_buf();
    }

    WorkflowDataDirResult {
        data_dir: None,
        project_dir: absolute_input.clone(),
        short_name: get_dir_short_name(&absolute_input),
        error: None,
    }
}

fn to_absolute_path(input: &str, context: &DataDirContext) -> PathBuf {
    let expanded = expand_tilde(input, &context.home);
    let path = if expanded.as_os_str().is_empty() {
        context.cwd.clone()
    } else if expanded.is_absolute() {
        expanded
    } else {
        context.cwd.join(expanded)
    };
    normalize_lexically(&path)
}

fn expand_tilde(input: &str, home: &Path) -> PathBuf {
    if input == "~" {
        return home.to_path_buf();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return home.join(rest);
    }
    #[cfg(windows)]
    if let Some(rest) = input.strip_prefix("~\\") {
        return home.join(rest);
    }
    PathBuf::from(input)
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn is_directory(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

fn project_for_data_dir(absolute_path: &Path) -> Option<PathBuf> {
    for suffix in POSSIBLE_WORKFLOW_DATA_PATHS {
        let suffix_path = Path::new(suffix);
        if !absolute_path.ends_with(suffix_path) {
            continue;
        }

        let mut project_dir = absolute_path.to_path_buf();
        for _ in suffix_path.components() {
            project_dir.pop();
        }
        return Some(project_dir);
    }
    None
}

fn first_data_dir_in(project_dir: &Path) -> Option<PathBuf> {
    POSSIBLE_WORKFLOW_DATA_PATHS
        .iter()
        .map(|candidate| project_dir.join(candidate))
        .find(|candidate| is_directory(candidate))
}

fn get_dir_short_name(project_dir: &Path) -> String {
    let names: Vec<_> = project_dir
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    match names.as_slice() {
        [] => {
            let display = project_dir.to_string_lossy();
            let trimmed = display.trim_matches(std::path::MAIN_SEPARATOR);
            if trimmed.is_empty() {
                "/".to_owned()
            } else {
                trimmed.to_owned()
            }
        }
        [name] => name.clone(),
        _ => format!("{}/{}", names[names.len() - 2], names[names.len() - 1]),
    }
}
