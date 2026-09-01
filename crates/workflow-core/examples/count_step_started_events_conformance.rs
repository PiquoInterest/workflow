use std::process::ExitCode;

use workflow_core::runtime::count_step_started_events::{
    StepEvent, StepStartScope, count_step_started_events, next_step_attempt_from_js_number,
};

fn print_error(message: &str) {
    println!("{{\"ok\":false,\"error\":\"{message}\"}}");
}

fn parse_number(value: &str) -> Option<f64> {
    match value {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => value.parse().ok(),
    }
}

fn parse_event(value: &str) -> Option<StepEvent> {
    let mut fields = value.splitn(3, '|');
    let kind = fields.next()?;
    let correlation_id = fields.next()?;
    let owner = fields.next().filter(|value| !value.is_empty());

    match kind {
        "started" => Some(StepEvent::started(correlation_id, owner)),
        "completed" => Some(StepEvent::completed(correlation_id)),
        _ => None,
    }
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let Some(operation) = arguments.next() else {
        return ExitCode::from(2);
    };

    match operation.as_str() {
        "next" => {
            let Some(value) = arguments.next().as_deref().and_then(parse_number) else {
                return ExitCode::from(2);
            };
            match next_step_attempt_from_js_number(value) {
                Ok(next) => println!("{{\"ok\":true,\"value\":{next}}}"),
                Err(error) => print_error(&error.message),
            }
        }
        "count" => {
            let Some(step_id) = arguments.next() else {
                return ExitCode::from(2);
            };
            let Some(scope) = arguments.next() else {
                return ExitCode::from(2);
            };
            let scope = if scope == "unscoped" {
                StepStartScope::Unscoped
            } else if scope == "total" {
                StepStartScope::TotalAttempts
            } else if let Some(owner) = scope.strip_prefix("owned|") {
                StepStartScope::OwnedBy(owner.to_owned())
            } else {
                return ExitCode::from(2);
            };
            let Some(events) = arguments
                .map(|argument| parse_event(&argument))
                .collect::<Option<Vec<_>>>()
            else {
                return ExitCode::from(2);
            };

            match count_step_started_events(Some(&events), &step_id, scope) {
                Ok(count) => println!("{{\"ok\":true,\"value\":{}}}", count.get()),
                Err(error) => print_error(&error.message),
            }
        }
        _ => return ExitCode::from(2),
    }

    ExitCode::SUCCESS
}
