use alloy::primitives::U256;
use anyhow::bail;
use hdp::hdp_run;
use hdp::preprocessor::module_registry::ModuleRegistry;
use hdp::primitives::ChainId;
use hdp::primitives::{
    aggregate_fn::{integer::Operator, FunctionContext},
    task::{
        datalake::{
            block_sampled::{
                AccountField, BlockSampledCollection, BlockSampledCollectionType,
                BlockSampledDatalake, HeaderField,
            },
            compute::Computation,
            datalake_type::DatalakeType,
            envelope::DatalakeEnvelope,
            transactions::{
                IncludedTypes, TransactionField, TransactionReceiptField, TransactionsCollection,
                TransactionsCollectionType, TransactionsInBlockDatalake,
            },
            DatalakeCompute,
        },
        TaskEnvelope,
    },
};
use inquire::{InquireError, Select};
use std::{path::PathBuf, str::FromStr};
use tracing::error;

pub async fn run_interactive() -> anyhow::Result<()> {
    println!("Welcome to Herodotus Data Processor interactive CLI! 🛰️");
    println!(
        r"
        _   _   ____    ____
        | | | | |  _ \  |  _ \
        | |_| | | | | | | |_) |
        |  _  | | |_| | |  __/
        |_| |_| |____/  |_|
"
    );

    let variants = TaskEnvelope::variants();
    let task_opts: Vec<&str> = variants.iter().map(AsRef::as_ref).collect();
    let task_ans: &str =
        Select::new("Step 1. What type of task you want to process?", task_opts).with_help_message("1. Datalake Compute : You can use defined type of datalake and computation functions to compute data. More easier to use, more faster to compute.\n 2. Module : Define your own logic of data and computation over cairo contract. More flexible, but more difficult to use.").prompt()?;

    let tasks = match task_ans {
        "DATALAKE_COMPUTE" => {
            let variants = DatalakeType::variants();
            let datalake_opts: Vec<&str> = variants.iter().map(AsRef::as_ref).collect();

            println!("Let's try to generate datalake and compute! ");

            let ans: Result<&str, InquireError> =
                Select::new("Step 2. What's your datalake type?", datalake_opts).prompt();

            let datalake_envelope: DatalakeEnvelope = match ans {
                Ok(choice) => {
                    let datalake_type = DatalakeType::from_str(choice)?;
                    match datalake_type {
                        DatalakeType::BlockSampled => {
                            // ================== Block Sampled Datalake Fields ==================
                            // 0. Chain ID
                            let chain_id: String = inquire::Text::new("Chain ID")
                                .with_help_message("What is the chain ID? (Enter to set default)")
                                .with_default("ETHEREUM_SEPOLIA")
                                .prompt()?;
                            // 1. Block range start
                            let block_range_start: u64 = inquire::Text::new("Block range start")
                                .with_help_message(
                                    "What is the block range start? (Enter to set default)",
                                )
                                .with_default("4952200")
                                .prompt()?
                                .parse()?;
                            // 2. Block range end
                            let block_range_end: u64 = inquire::Text::new("Block range end")
                                .with_help_message(
                                    "What is the block range end? (Enter to set default)",
                                )
                                .with_default("4952229")
                                .prompt()?
                                .parse()?;
                            // 3. Increment
                            let increment: u64 = inquire::Text::new("Increment")
                                .with_help_message(
                                    "How many blocks to skip in the range? (Enter to set default)",
                                )
                                .with_default("1")
                                .prompt()?
                                .parse()?;
                            // 4. Sampled Property
                            // 4.1. Block Sampled Collection Type
                            let variants: Vec<String> = BlockSampledCollectionType::variants();
                            let collection_opts: Vec<&str> =
                                variants.iter().map(AsRef::as_ref).collect();
                            let collection_ans: &str = Select::new(
                                "Sample Property: Select block sample type",
                                collection_opts,
                            )
                            .with_help_message("What type of block sample do you want to process?")
                            .prompt()?;
                            let collection_type =
                                BlockSampledCollectionType::from_str(collection_ans)?;
                            // 4.2. Detail Sampled Property
                            let sampled_property = match collection_type {
                                BlockSampledCollectionType::Header => {
                                    let variants: Vec<String> = HeaderField::variants();
                                    let header_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let header_ans: &str =
                                        Select::new("Select detail header property", header_opts).with_help_message("What header property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("header.{}", header_ans)
                                }
                                BlockSampledCollectionType::Account => {
                                    let address = inquire::Text::new("Enter target address")
                                        .with_help_message("Enter target address")
                                        .prompt()?;
                                    let variants: Vec<String> = AccountField::variants();
                                    let account_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let account_ans: &str =
                                        Select::new("Select detail account property", account_opts)
                                        .with_help_message("What account property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("account.{}.{}", address, account_ans)
                                }
                                BlockSampledCollectionType::Storage => {
                                    let address = inquire::Text::new("Enter target address")
                                        .with_help_message("Enter target address")
                                        .prompt()?;
                                    let storage_key =
                                        inquire::Text::new("Enter target storage key")
                                            .with_help_message("Enter the storage key")
                                            .prompt()?;
                                    format!("storage.{}.{}", address, storage_key)
                                }
                            };
                            let block_sampled_datalake = BlockSampledDatalake::new(
                                ChainId::from_str(&chain_id)?,
                                block_range_start,
                                block_range_end,
                                increment,
                                BlockSampledCollection::from_str(&sampled_property)?,
                            );
                            DatalakeEnvelope::BlockSampled(block_sampled_datalake)
                        }
                        DatalakeType::TransactionsInBlock => {
                            // 0. Chain ID
                            let chain_id: String = inquire::Text::new("Chain ID")
                                .with_help_message("What is the chain ID? (Enter to set default)")
                                .with_default("ETHEREUM_SEPOLIA")
                                .prompt()?;
                            let target_block: u64 = inquire::Text::new("Enter target block number")
                                .with_help_message(
                                    "What block you target to get transactions? (Enter to set default)",
                                )
                                .with_default("4952200")
                                .prompt()?
                                .parse()?;
                            let start_index: u64 = inquire::Text::new("Start index")
                            .with_help_message(
                                "What is the start index of transactions in the block? (Enter to set default)",
                            )
                            .with_default("0")
                            .prompt()?
                            .parse()?;
                            // TODO: have end index dynamically fetch by block number
                            let end_index: u64 = inquire::Text::new("End index")
                            .with_help_message(
                                "What is the end index of transactions in the block? (Enter to set default)",
                            )
                            .with_default("10")
                            .prompt()?
                            .parse()?;
                            let increment: u64 = inquire::Text::new("Increment")
                                .with_help_message(
                                    "How many transactions to skip in the range? (Enter to set default)",
                                )
                                .with_default("1")
                                .prompt()?
                                .parse()?;
                            let included_types: Vec<u8> = inquire::Text::new("Included types")
                                .with_help_message(
                                    "What type of transactions to include? (Enter to set default)",
                                )
                                .with_default("1,1,1,1")
                                .prompt()?
                                .split(',')
                                .map(|s| s.parse().unwrap())
                                .collect();
                            let variants = TransactionsCollectionType::variants();
                            let collection_opts: Vec<&str> =
                                variants.iter().map(AsRef::as_ref).collect();
                            let collection_ans: &str = Select::new(
                                "Sample Property: Select block sample type",
                                collection_opts,
                            )
                            .with_help_message("What type of block sample do you want to process?")
                            .prompt()?;
                            let collection_type =
                                TransactionsCollectionType::from_str(collection_ans)?;
                            let sampled_property = match collection_type {
                                TransactionsCollectionType::Transactions => {
                                    let variants: Vec<String> = TransactionField::variants();
                                    let transaction_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let transaction_ans: &str =
                                        Select::new("Select detail transaction property", transaction_opts)
                                        .with_help_message("What transaction property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("tx.{}", transaction_ans)
                                }
                                TransactionsCollectionType::TransactionReceipts => {
                                    let variants = TransactionReceiptField::variants();
                                    let transaction_receipt_opts: Vec<&str> =
                                        variants.iter().map(AsRef::as_ref).collect();
                                    let transaction_receipt_ans: &str =
                                        Select::new("Select detail transaction receipt property", transaction_receipt_opts)
                                        .with_help_message("What transaction receipt property do you want to sample? (all properties are decodable from rlp encoded data)")
                                            .prompt()?;
                                    format!("tx_receipt.{}", transaction_receipt_ans)
                                }
                            };
                            let transactions_datalake = TransactionsInBlockDatalake::new(
                                ChainId::from_str(&chain_id)?,
                                target_block,
                                TransactionsCollection::from_str(&sampled_property)?,
                                start_index,
                                end_index,
                                increment,
                                IncludedTypes::from(&included_types),
                            );
                            DatalakeEnvelope::TransactionsInBlock(transactions_datalake)
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                    bail!(e);
                }
            };

            let task_opts: Vec<&str> = vec!["AVG", "SUM", "MIN", "MAX", "COUNT"];

            let aggregate_fn_id = Select::new("Select the aggregation function", task_opts)
                .with_help_message(
                    "Step 3. What type of aggregation do you want to perform on the datalake?",
                )
                .prompt()?;

            let aggregate_fn_ctx = match aggregate_fn_id {
                "COUNT" => {
                    let operator_ans: String = Select::new(
                        "Select the COURNIF operator",
                        vec!["=", "!=", ">", ">=", "<", "<="],
                    )
                    .with_help_message("How would like to set opersation case?")
                    .prompt()?
                    .into();
                    let value_to_compare: String = inquire::Text::new("Enter the value to compare")
                        .with_help_message("Make sure to input Uint256 value")
                        .prompt()?;
                    Some(FunctionContext::new(
                        Operator::from_symbol(&operator_ans)?,
                        U256::from_str(&value_to_compare)?,
                    ))
                }
                _ => None,
            };

            let target_datalake_compute = DatalakeCompute::new(
                datalake_envelope,
                Computation::new(aggregate_fn_id.parse()?, aggregate_fn_ctx),
            );
            vec![TaskEnvelope::DatalakeCompute(target_datalake_compute)]
        }
        "MODULE" => {
            println!("Let's try to generate module! ");
            let module_program_hash: String =
                inquire::Text::new("Enter Program Hash of Module. Make sure you had registered:")
                    .with_default(
                        "0x064041a339b1edd10de83cf031cfa938645450f971d2527c90d4c2ce68d7d412",
                    )
                    .prompt()?;
            let module_inputs: Vec<String> = inquire::Text::new(
                "Enter Module inputs. Split them using comma : ",
            )
            .with_help_message(
                "This values will goes through Cairo Contract's input. (Enter to set default)",
            )
            .with_default("0x5222a4,0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5")
            .prompt()?
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

            let module_registry = ModuleRegistry::new();
            let module = module_registry
                .get_extended_module_from_class_source_string(
                    Some(module_program_hash),
                    None,
                    module_inputs,
                )
                .await?;

            vec![TaskEnvelope::Module(module)]
        }
        _ => {
            bail!("Invalid task");
        }
    };

    let allow_run: bool = inquire::Confirm::new("Do you want to run the full processor? Running the processor will generate input for Cairo program and PIE file")
    .with_default(true)
    .prompt()?;
    if allow_run {
        println!("Make sure to position correct rpc url related env variables.");

        let output_file: PathBuf = inquire::Text::new("Enter Batch proof file path: ")
            .with_default("batch.json")
            .prompt()?
            .into();
        let cairo_input: PathBuf = inquire::Text::new("Enter Cairo input file path:")
            .with_default("input.json")
            .prompt()?
            .into();
        let pie_file: PathBuf = inquire::Text::new("Enter PIE output file path:")
            .with_default("hdp_pie.zip")
            .prompt()?
            .into();
        let config = hdp_run::HdpRunConfig::init(
            None,
            None,
            cairo_input,
            false,
            None,
            Some(output_file),
            Some(pie_file),
        );

        hdp::run(&config, tasks).await?
    }
    Ok(())
}
