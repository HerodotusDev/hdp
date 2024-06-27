use std::path::PathBuf;

use hdp_primitives::request_types::DataProcessorRequest;

pub(crate) fn exec_local_run(
    tasks_request_file: PathBuf,
    rpc_url: Option<String>,
    output_file: Option<String>,
    cairo_input: Option<String>,
    pie_file: Option<String>,
) {
    let tasks_request = std::fs::read_to_string(tasks_request_file).expect("Failed to read file");
    let tasks_request: DataProcessorRequest =
        serde_json::from_str(&tasks_request).expect("Failed to parse tasks request");
}
