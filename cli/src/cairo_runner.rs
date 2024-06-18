use anyhow::Result;
use hdp_core::evaluator::evaluation_result_to_result_commitment;
use hdp_core::evaluator::result::ProcessedResult;
use std::process::{Command, Stdio};
use std::{error::Error, fs};
use tracing::{debug, info};

use anyhow::bail;
use hdp_primitives::datalake::output::Uint256;
use regex::Regex;

const COMPILED_CAIRO_PATH: &str = "build/compiled_cairo/hdp.json";

struct CairoRunResult {
    task_results: Vec<String>,
    results_root: String,
}

pub struct CairoRunner {
    pre_processed_result: ProcessedResult,
}

impl CairoRunner {
    pub fn new(pre_processed_result: ProcessedResult) -> Self {
        Self {
            pre_processed_result,
        }
    }

    pub fn run(
        &self,
        cairo_pie_file_path: String,
        input_file_path: String,
    ) -> Result<ProcessedResult, Box<dyn Error>> {
        let context = fs::read_to_string(&input_file_path)?;
        if context.is_empty() {
            return Err("Input file is empty".into());
        }
        let context = fs::read_to_string(COMPILED_CAIRO_PATH)?;
        if context.is_empty() {
            return Err("Cairo compilation failed".into());
        }

        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(COMPILED_CAIRO_PATH)
            .arg("--layout")
            .arg("starknet_with_keccak")
            .arg("--program_input")
            .arg(&input_file_path)
            .arg("--cairo_pie_output")
            .arg(&cairo_pie_file_path)
            .arg("--print_output")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = task.wait_with_output().expect("Failed to read stdout");
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result = self.parse_output(output_str.to_string())?;
        let final_result = self.construct_final_output(result)?;
        debug!("Final result: {:#?}", final_result.tasks);
        debug!("Final result root: {:#?}", final_result.results_root);

        info!("Cairo pie file saved in : {}", cairo_pie_file_path);

        Ok(final_result)
    }

    fn parse_output(&self, output: String) -> Result<CairoRunResult> {
        let task_result_re = Regex::new(r"Task Result\((\d+)\): (\S+)").unwrap();
        let mut task_results = vec![];
        for caps in task_result_re.captures_iter(&output) {
            let _ = &caps[1];
            let value = &caps[2];
            task_results.push(value.to_string());
        }
        let results_root_re = Regex::new(r"Results Root: (\S+) (\S+)").unwrap();
        if let Some(results_root_caps) = results_root_re.captures(&output) {
            let results_root_1 = &results_root_caps[1];
            let results_root_2 = &results_root_caps[2];
            let result_root = Uint256::from_strs(results_root_2, results_root_1)?;
            let combined_results_root = result_root.to_combined_string().to_string();
            Ok(CairoRunResult {
                task_results,
                results_root: combined_results_root,
            })
        } else {
            bail!("Results Root not found");
        }
    }

    fn construct_final_output(&self, cairo_run_result: CairoRunResult) -> Result<ProcessedResult> {
        // turn context into struct
        let mut final_processed_result = self.pre_processed_result.clone();
        final_processed_result.results_root = Some(cairo_run_result.results_root);
        final_processed_result.tasks = final_processed_result
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let mut task = task.clone();
                let compiled_result = cairo_run_result.task_results[i].clone();
                task.compiled_result = Some(compiled_result.clone());
                let result_commitment =
                    evaluation_result_to_result_commitment(&task.task_commitment, &compiled_result);
                task.result_commitment = Some(result_commitment.to_string());
                task
            })
            .collect();

        Ok(final_processed_result)
    }
}
