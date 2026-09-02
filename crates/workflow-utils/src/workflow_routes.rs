use std::fmt::Write as _;
use std::sync::{OnceLock, RwLock};

pub const WORKFLOW_ROUTE_BASE: &str = "/.well-known/workflow/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowRoute {
    Flow,
    Manifest,
    Webhook(String),
    Health,
    /// Compatibility sentinel used to reject the retired standalone route.
    Step,
}

impl WorkflowRoute {
    fn endpoint(&self) -> Result<String, String> {
        match self {
            Self::Flow | Self::Health => Ok("flow".to_owned()),
            Self::Manifest => Ok("manifest.json".to_owned()),
            Self::Webhook(token) => Ok(format!("webhook/{}", encode_uri_component(token))),
            Self::Step => Err("Unsupported workflow route: step".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowRoutes {
    base_path: String,
}

impl WorkflowRoutes {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base_path: String::new(),
        }
    }

    pub fn set_workflow_base_path(&mut self, base_path: Option<&str>) {
        self.base_path.clear();
        self.base_path.push_str(base_path.unwrap_or_default());
    }

    pub fn create_workflow_base_url(&self, origin: &str) -> Result<String, String> {
        validate_absolute_url(origin)?;
        let without_fragment = origin.split_once('#').map_or(origin, |(value, _)| value);
        let without_query = without_fragment
            .split_once('?')
            .map_or(without_fragment, |(value, _)| value);
        Ok(format!(
            "{}{}",
            without_query.trim_end_matches('/'),
            self.base_path
        ))
    }

    pub fn create_workflow_url(
        &self,
        base_url: &str,
        route: WorkflowRoute,
    ) -> Result<String, String> {
        let parsed = ParsedAbsoluteUrl::parse(base_url)?;
        let endpoint = route.endpoint()?;
        let path = parsed.path.trim_end_matches('/');
        let search = if route == WorkflowRoute::Health {
            "?__health"
        } else {
            ""
        };

        Ok(format!(
            "{}://{}{}{WORKFLOW_ROUTE_BASE}/{endpoint}{search}",
            parsed.scheme, parsed.authority, path
        ))
    }

    #[must_use]
    pub fn create_workflow_health_endpoint(&self) -> String {
        format!(
            "{}{}{}",
            self.base_path, WORKFLOW_ROUTE_BASE, "/flow?__health"
        )
    }
}

static GLOBAL_BASE_PATH: OnceLock<RwLock<String>> = OnceLock::new();

fn global_base_path() -> &'static RwLock<String> {
    GLOBAL_BASE_PATH.get_or_init(|| RwLock::new(String::new()))
}

/// Sets the process-wide base path used by the free-function route helpers.
pub fn set_workflow_base_path(base_path: Option<&str>) {
    let mut configured = global_base_path()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    configured.clear();
    configured.push_str(base_path.unwrap_or_default());
}

fn configured_routes() -> WorkflowRoutes {
    let configured = global_base_path()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    WorkflowRoutes {
        base_path: configured.clone(),
    }
}

pub fn create_workflow_base_url(origin: &str) -> Result<String, String> {
    configured_routes().create_workflow_base_url(origin)
}

pub fn create_workflow_url(base_url: &str, route: WorkflowRoute) -> Result<String, String> {
    configured_routes().create_workflow_url(base_url, route)
}

#[must_use]
pub fn create_workflow_health_endpoint() -> String {
    configured_routes().create_workflow_health_endpoint()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedAbsoluteUrl<'a> {
    scheme: &'a str,
    authority: &'a str,
    path: &'a str,
}

impl<'a> ParsedAbsoluteUrl<'a> {
    fn parse(value: &'a str) -> Result<Self, String> {
        validate_absolute_url(value)?;
        let (scheme, rest) = value
            .split_once("://")
            .ok_or_else(|| invalid_url(value))?;
        let without_fragment = rest.split_once('#').map_or(rest, |(part, _)| part);
        let without_query = without_fragment
            .split_once('?')
            .map_or(without_fragment, |(part, _)| part);
        let boundary = without_query.find('/').unwrap_or(without_query.len());
        let (authority, path) = without_query.split_at(boundary);
        if authority.is_empty() {
            return Err(invalid_url(value));
        }

        Ok(Self {
            scheme,
            authority,
            path,
        })
    }
}

fn validate_absolute_url(value: &str) -> Result<(), String> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(invalid_url(value));
    }
    let (scheme, rest) = value
        .split_once("://")
        .ok_or_else(|| invalid_url(value))?;
    let mut chars = scheme.chars();
    let first = chars.next().ok_or_else(|| invalid_url(value))?;
    let starts_with_delimiter = matches!(rest.as_bytes().first(), Some(b'/' | b'?' | b'#'));
    if !first.is_ascii_alphabetic()
        || !chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
        || rest.is_empty()
        || starts_with_delimiter
    {
        return Err(invalid_url(value));
    }
    Ok(())
}

fn invalid_url(value: &str) -> String {
    format!("Invalid absolute URL: {value}")
}

fn encode_uri_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            )
        {
            encoded.push(char::from(byte));
        } else {
            write!(&mut encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    encoded
}
