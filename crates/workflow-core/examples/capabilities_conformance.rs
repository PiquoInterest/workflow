use workflow_core::capabilities::get_run_capabilities;

fn main() {
    let version = std::env::args().nth(1);
    let capabilities = get_run_capabilities(version.as_deref());
    let supported_formats = capabilities
        .supported_formats
        .iter()
        .map(|format| format!("\"{}\"", format.as_prefix()))
        .collect::<Vec<_>>()
        .join(",");

    println!(
        "{{\"supportedFormats\":[{supported_formats}],\"framedByteStreams\":{}}}",
        capabilities.framed_byte_streams
    );
}
