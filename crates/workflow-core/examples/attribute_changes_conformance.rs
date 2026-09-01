use std::fmt::Write as _;
use std::process::ExitCode;

use workflow_core::attribute_changes::{
    AttributeChange, AttributeField, AttributeInput, FatalError, NormalizeAttributeOptions,
    normalize_attribute_changes,
};

fn encode_hex(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.bytes() {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn print_outcome(result: Result<Vec<AttributeChange>, FatalError>) {
    match result {
        Ok(changes) => {
            print!("ok");
            for change in changes {
                print!("\t{}\t", encode_hex(&change.key));
                match change.value {
                    Some(value) => print!("s{}", encode_hex(&value)),
                    None => print!("n"),
                }
            }
            println!();
        }
        Err(error) => println!("err\t{}", encode_hex(&error.message)),
    }
}

fn parse_record(
    mut arguments: impl Iterator<Item = String>,
) -> Option<(AttributeInput, NormalizeAttributeOptions)> {
    let allow_reserved_attributes = match arguments.next()?.as_str() {
        "0" => false,
        "1" => true,
        _ => return None,
    };

    let mut fields = Vec::new();
    loop {
        let Some(key) = arguments.next() else {
            break;
        };
        let value_argument = arguments.next()?;
        let value = if value_argument == "n" {
            None
        } else {
            Some(value_argument.strip_prefix("s:")?.to_owned())
        };
        fields.push(AttributeField { key, value });
    }

    Some((
        AttributeInput::Record(fields),
        NormalizeAttributeOptions {
            allow_reserved_attributes,
        },
    ))
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let Some(operation) = arguments.next() else {
        return ExitCode::from(2);
    };

    let parsed = match operation.as_str() {
        "record" => parse_record(arguments),
        "null" => Some((AttributeInput::Null, NormalizeAttributeOptions::default())),
        "array" => Some((
            AttributeInput::Array(vec!["phase".to_owned(), "init".to_owned()]),
            NormalizeAttributeOptions::default(),
        )),
        "string" => Some((
            AttributeInput::String("phase=init".to_owned()),
            NormalizeAttributeOptions::default(),
        )),
        "number" => Some((
            AttributeInput::Number(42.0),
            NormalizeAttributeOptions::default(),
        )),
        _ => None,
    };

    let Some((input, options)) = parsed else {
        return ExitCode::from(2);
    };
    print_outcome(normalize_attribute_changes(input, options));
    ExitCode::SUCCESS
}
