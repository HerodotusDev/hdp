pub mod aggregation_fn;
use aggregation_fn::AggregationFunction;
use alloy_dyn_abi::DynSolValue;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::{hex::FromHex, FixedBytes, Keccak256, B256, U256};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::compiler::{DatalakeResult, Derivable};

use super::task::ComputationalTask;

use hdp_primitives::{
    datalake::{datalake_type::DatalakeType, envelope::DatalakeEnvelope},
    format::{
        split_big_endian_hex_into_parts, Account, AccountFormatted, Header, HeaderFormatted,
        MMRMeta, ProcessedResult, ProcessedResultFormatted, Storage, StorageFormatted, Task,
        TaskFormatted,
    },
};

use hdp_provider::evm::AbstractProvider;

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluationResult {
    /// task_commitment -> fetched datalake relevant data
    pub fetched_datalake_results: HashMap<String, DatalakeResult>,
    /// task_commitment -> compiled_result
    pub compiled_results: HashMap<String, String>,
    /// ordered task_commitment
    pub ordered_tasks: Vec<String>,
    /// encoded tasks task_commitment -> encoded task
    pub encoded_tasks: HashMap<String, String>,
    /// encoded datalakes task_commitment -> evaluated datalake
    pub encoded_datalakes: HashMap<String, EvaluatedDatalake>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluatedDatalake {
    /// encoded datalake
    pub encoded_datalake: String,
    /// ex. dynamic datalake / block sampled datalake
    pub datalake_type: DatalakeType,
    /// ex. "header", "account", "storage"
    pub property_type: u8,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            ordered_tasks: Vec::new(),
            compiled_results: HashMap::new(),
            fetched_datalake_results: HashMap::new(),
            encoded_tasks: HashMap::new(),
            encoded_datalakes: HashMap::new(),
        }
    }

    pub fn build_merkle_tree(&self) -> Result<(StandardMerkleTree, StandardMerkleTree)> {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();

        for task_commitment in &self.ordered_tasks {
            let compiled_result = match self.compiled_results.get(task_commitment) {
                Some(result) => result,
                None => bail!("Task commitment not found in compiled results"),
            };

            let typed_task_commitment = FixedBytes::from_hex(task_commitment)?;
            tasks_leaves.push(DynSolValue::FixedBytes(typed_task_commitment, 32));

            let result_commitment =
                evaluation_result_to_result_commitment(task_commitment, compiled_result);
            results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
        }
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);
        let results_merkle_tree = StandardMerkleTree::of(results_leaves);

        Ok((tasks_merkle_tree, results_merkle_tree))
    }

    pub fn save_to_file(&self, file_path: &str, is_cairo_format: bool) -> Result<()> {
        let json = if is_cairo_format {
            self.to_cairo_formatted_json()?
        } else {
            self.to_general_json()?
        };
        std::fs::write(file_path, json)?;
        Ok(())
    }

    pub fn to_general_json(&self) -> Result<String> {
        // 1. build merkle tree
        let (tasks_merkle_tree, results_merkle_tree) = self.build_merkle_tree()?;

        // 2. get roots of merkle tree
        let task_merkle_root = tasks_merkle_tree.root();
        let result_merkle_root = results_merkle_tree.root();

        // 3. flatten the datalake result for all tasks
        let mut flattened_headers: HashSet<Header> = HashSet::new();
        let mut flattened_accounts: HashSet<Account> = HashSet::new();
        let mut flattened_storages: HashSet<Storage> = HashSet::new();
        let mut assume_mmr_meta: Option<MMRMeta> = None;

        let mut procesed_tasks: Vec<Task> = vec![];

        for task_commitment in &self.ordered_tasks {
            let datalake_result = match self.fetched_datalake_results.get(task_commitment) {
                Some(result) => result,
                None => bail!("Task commitment not found in fetched datalake results"),
            };
            let header_set: HashSet<Header> = datalake_result.headers.iter().cloned().collect();
            let account_set: HashSet<Account> = datalake_result.accounts.iter().cloned().collect();
            let storage_set: HashSet<Storage> = datalake_result.storages.iter().cloned().collect();
            flattened_headers.extend(header_set);
            flattened_accounts.extend(account_set);
            flattened_storages.extend(storage_set);
            assume_mmr_meta = Some(datalake_result.mmr_meta.clone());

            let result = match self.compiled_results.get(task_commitment) {
                Some(result) => result,
                None => bail!("Task commitment not found in compiled results"),
            };
            let typed_task_commitment = FixedBytes::from_hex(task_commitment)?;
            let task_proof =
                tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(typed_task_commitment, 32));
            let result_commitment = evaluation_result_to_result_commitment(task_commitment, result);
            let result_proof =
                results_merkle_tree.get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
            let encoded_task = match self.encoded_tasks.get(task_commitment) {
                Some(encoded_task) => encoded_task.to_string(),
                None => bail!("Task commitment not found in encoded tasks"),
            };
            let datalake = match self.encoded_datalakes.get(task_commitment) {
                Some(datalake) => datalake,
                None => bail!("Task commitment not found in encoded datalakes"),
            };

            procesed_tasks.push(Task {
                encoded_task,
                task_commitment: task_commitment.to_string(),
                task_proof,
                compiled_result: result.to_string(),
                result_commitment: result_commitment.to_string(),
                result_proof,
                encoded_datalake: datalake.encoded_datalake.clone(),
                datalake_type: datalake.datalake_type.into(),
                property_type: datalake.property_type,
            });
        }

        let processed_result = ProcessedResult {
            results_root: result_merkle_root.to_string(),
            tasks_root: task_merkle_root.to_string(),
            headers: flattened_headers.into_iter().collect(),
            accounts: flattened_accounts.into_iter().collect(),
            mmr: assume_mmr_meta.unwrap(),
            storages: flattened_storages.into_iter().collect(),
            tasks: procesed_tasks,
        };

        Ok(serde_json::to_string(&processed_result)?)
    }

    pub fn to_cairo_formatted_json(&self) -> Result<String> {
        // 1. build merkle tree
        let (tasks_merkle_tree, results_merkle_tree) = self.build_merkle_tree()?;
        // 2. get roots
        let task_merkle_root = tasks_merkle_tree.root();
        let result_merkle_root = results_merkle_tree.root();

        // 3. flatten the datalake result for all tasks
        let mut flattened_deaders: HashSet<HeaderFormatted> = HashSet::new();
        let mut flattened_accounts: HashSet<AccountFormatted> = HashSet::new();
        let mut flattened_storages: HashSet<StorageFormatted> = HashSet::new();
        let mut assume_mmr_meta: Option<MMRMeta> = None;

        let mut procesed_tasks: Vec<TaskFormatted> = vec![];

        for task_commitment in &self.ordered_tasks {
            let datalake_result = self.fetched_datalake_results.get(task_commitment).unwrap();
            let header_set: HashSet<HeaderFormatted> = datalake_result
                .headers
                .iter()
                .cloned()
                .map(|h| h.to_cairo_format())
                .collect();
            let account_set: HashSet<AccountFormatted> = datalake_result
                .accounts
                .iter()
                .cloned()
                .map(|a| a.to_cairo_format())
                .collect();
            let storage_set: HashSet<StorageFormatted> = datalake_result
                .storages
                .iter()
                .cloned()
                .map(|s| s.to_cairo_format())
                .collect();
            flattened_deaders.extend(header_set);
            flattened_accounts.extend(account_set);
            flattened_storages.extend(storage_set);
            assume_mmr_meta = Some(datalake_result.mmr_meta.clone());

            let result = self.compiled_results.get(task_commitment).unwrap();
            let typed_task_commitment = FixedBytes::from_hex(task_commitment).unwrap();
            let task_proof =
                tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(typed_task_commitment, 32));
            let result_commitment = evaluation_result_to_result_commitment(task_commitment, result);
            let result_proof =
                results_merkle_tree.get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
            let encoded_task = self.encoded_tasks.get(task_commitment).unwrap().to_string();
            let evaluated_datalake = self.encoded_datalakes.get(task_commitment).unwrap();
            let task = Task {
                encoded_task,
                task_commitment: task_commitment.to_string(),
                task_proof,
                compiled_result: result.to_string(),
                result_commitment: result_commitment.to_string(),
                result_proof,
                encoded_datalake: evaluated_datalake.encoded_datalake.clone(),
                datalake_type: evaluated_datalake.datalake_type.into(),
                property_type: evaluated_datalake.property_type,
            };

            procesed_tasks.push(task.to_cairo_format());
        }

        let processed_result = ProcessedResultFormatted {
            results_root: split_big_endian_hex_into_parts(&result_merkle_root.to_string()),
            tasks_root: split_big_endian_hex_into_parts(&task_merkle_root.to_string()),
            headers: flattened_deaders.into_iter().collect(),
            accounts: flattened_accounts.into_iter().collect(),
            mmr: assume_mmr_meta.unwrap(),
            storages: flattened_storages.into_iter().collect(),
            tasks: procesed_tasks,
        };

        Ok(serde_json::to_string(&processed_result)?)
    }
}

pub fn evaluation_result_to_result_commitment(
    task_commitment: &str,
    compiled_result: &str,
) -> FixedBytes<32> {
    let mut hasher = Keccak256::new();
    hasher.update(Vec::from_hex(task_commitment).unwrap());
    hasher.update(B256::from(U256::from_str(compiled_result).unwrap()));
    hasher.finalize()
}

impl Default for EvaluationResult {
    fn default() -> Self {
        EvaluationResult::new()
    }
}

pub async fn evaluator(
    mut computational_tasks: Vec<ComputationalTask>,
    datalake_for_tasks: Option<Vec<DatalakeEnvelope>>,
    provider: Arc<RwLock<AbstractProvider>>,
) -> Result<EvaluationResult> {
    let mut results = EvaluationResult::new();

    // If optional datalake_for_tasks is provided, need to assign the datalake to the corresponding task
    if let Some(datalake) = datalake_for_tasks {
        for (datalake_idx, datalake) in datalake.iter().enumerate() {
            let task = &mut computational_tasks[datalake_idx];

            task.datalake = Some(datalake.derive());
        }
    }

    // Evaulate the compute expressions
    for task in computational_tasks {
        // task_commitment is the unique identifier for the task
        let task_commitment = task.to_string();
        // Encode the task
        let encoded_task = task.encode()?;
        let mut datalake_base = match task.datalake {
            Some(datalake) => datalake,
            None => bail!("Task is not filled with datalake"),
        };

        let datalake_result = datalake_base.compile(&provider).await?;
        match datalake_base.datalake {
            Some(datalake) => {
                let encoded_datalake = datalake.encode()?;
                let aggregation_fn = AggregationFunction::from_str(&task.aggregate_fn_id)?;
                let aggregation_fn_ctx = task.aggregate_fn_ctx;
                // Compute datalake over specified aggregation function
                let result = aggregation_fn
                    .operation(&datalake_result.compiled_results, aggregation_fn_ctx)?;
                // Save the datalake results
                results
                    .compiled_results
                    .insert(task_commitment.to_string(), result);
                // Save order of tasks
                results.ordered_tasks.push(task_commitment.to_string());
                // Save the fetched datalake results
                results
                    .fetched_datalake_results
                    .insert(task_commitment.to_string(), datalake_result);
                // Save the task data
                results
                    .encoded_tasks
                    .insert(task_commitment.to_string(), encoded_task);
                // Save the datalake data
                results.encoded_datalakes.insert(
                    task_commitment,
                    EvaluatedDatalake {
                        encoded_datalake,
                        datalake_type: datalake.get_datalake_type(),
                        property_type: datalake.get_collection_type().to_index(),
                    },
                );
            }
            None => bail!("Datalake base is not filled with specific datalake"),
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {

    use super::*;
    use hdp_primitives::format::{Account, Header, HeaderProof, MMRMeta, MPTProof, Storage};

    fn setup() -> EvaluationResult {
        let mut init_eval_result = EvaluationResult::new();
        init_eval_result.fetched_datalake_results.insert(
            "0x242fe0d1fa98c743f84a168ff10abbcca83cb9e0424f4541fab5041cd63d3387".to_string(),
            DatalakeResult {
                compiled_results: vec!["0x9184e72a000".to_string()],
                headers: vec![Header {
                    rlp: "f90253a008a4f6a7d5055ce465e285415779bc338134600b750c06396531ce6a29d09f4ba01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347941268ad189526ac0b386faf06effc46779c340ee6a0fa23637d8a5d4a624479b33410895951995bae67f7c16b00859f9ac630b9e020a0792c487bc3176e482c995a9a1a16041d456db8d52e0db6fb73b540a64e96feaca04406def0dad7a6c6ef8c41a59be6b5b89124391a5b0491c8a5339e859e24d7acb901001a820024432050a200d1bc129162042984e09002002806340a14630c0aca5060c140a0608e043199e90280a1418cb89f1020085394a48f412d00d05041ad00a09002801a30b50d10c008522a2203284384841e055052404040710462e48103580026004a4e6842518210c2060c0729944118e4d0801936d020008811bb0c0464028a0008219056543b1111890cac50c04805000a400040401089904927409ec6720b8001c80a204628d8400064b402a1220480c21418480c24d00446a743000180a880128245028010a00103a8036b06c119a20124c32482280cc14021b430082a9408840030d46c062010f0b290c194040888189e081100c1070280304c0a01808352229a8401c9c38084017f9a188465df90188a4e65746865726d696e64a0178bae25662326acf0824d8441db8493865a53b8c627dc8aea5eb50ed2102fdc8800000000000000008401d76098a06eb2bc6208c3733aa1158ff8a100cb5c7ad1706ac6c3fb95d28f28007a770403808404c20000a0195eac87285a920cb37eb2b2dcf6eb9853efa2547c386bfe58ca2ff0fe167eb5".to_string(),
                    proof: HeaderProof {
                        leaf_idx: 660751,
                        mmr_path: vec![
                            "0x50b38c27a0e12585ae387c5fd218ccaea57f7ff72cd4739ed9ff0a29ba6fe7a".to_string(),
                            "0x072bda07a8e5d5e8ad4b09bdc39c842d2ae64c7e27da33030f7ae9ad0f67295".to_string(),
                            "0x4a52af19f212ccf13b343003c02169115c68af56a0ed3f8efe8f24fc6e7be30".to_string(),
                            "0x5b6148cde08614b2c03dca0ff0cc89ab6f68f42ed3a7039e8fb4bb96d18dab4".to_string(),
                            "0x123a65b944f582989bed238cfbc60a4b05b06a279e2618941075abb075262ae".to_string(),
                        ],
                    },
                }],
              accounts: vec![
                Account {
                    address: "0x75cec1db9dceb703200eaa6595f66885c962b920".to_string(),
                    account_key: "0x962f445fc8476432660877b666f653759ea69189b60d2f4a7008e70555746ad1".to_string(),
                    proofs: vec![MPTProof {
                        block_number: 5382810,
                        proof: vec![
                        "0xf90211a004a07b0ced9c4e49cf3574d029b4aca46893aaead889508635b766b8bd9ff49aa035557e7ab5adda1f7876e96caf874a825a03267c9bcbd85e14f3578f7b80980ba05190d1fdc6e8506a5cc08e7291498d62aafc44913c4b47dc998d3cff5a7fee29a0cc16f65cc93a89251834e9e703f7bca425ad644dcb8d7502870439a47e7377c3a014623f34ab8b17adca3cf7648bac3f59b67fccf9082cf8bfd1a5f58a3cc5483da07f046112f9c54206ecf2379d2c75c6a343e19f19563a615163d7f032e54b70baa0869bb928152f852a8fb130ba8b95597a49a7c4b53cb6ab7af56f0f2e0a9d22f9a0effce50b7901262428133a0829fc11baf319f1a0a5388ddb0546a55d26ddb01ca096bedf7371a32ebbfbf159c3688efb85b675fc9968274d30f436025735633fd9a09ae43877fde992c6b39eed307abcea8a922918ceb672cc5257f3a2f1d23210d3a0c496cfc0e6c6d082ad5d80d827ad3fd748da5fef22e321f98d110f55166c0104a012155dffd30839241dad6bdba97b30b3b0368ceaf46069aba61cf63c7735ef8aa02b78b3f87d1c29a10c6b584e47f2df8b2f6d333b6c03f62413db4fc732d8b543a05bd948e3417c7e63795702d6d8b249d6bad8f5b47e05058ad83f869e719c76d9a037f216a3e0a186c53c6216a6990f422bf38c28e68e4d29f65feca1d9d518acdca0e56801140a1beffc0a88cad0b104d40024b5bf9586d175496821aff4a649cb3380".to_string(),
                        "0xf90211a03f3580144cee82b6906ca3c934415e6b729e301bc1d2267f115e55ae0bd863b0a07a99440f7094ed5913f8b84f4adfed5f7c5a6fb472aca8c6c121f21c4c2568bea071880cb0a7afa1ff0ef3e4925dd813108ce7ba0ab307e6360d686f05f6c3ad64a07beefec05c82c8047f804931cc7c4641f6336b89b26f133afd7d4e02e58c32eea04c75cf284cd38baff74dcd2af18f1826aa7324d0d71fedbf686bfb9cd75c49e1a02c46c881f9ee6f9b938848e291cf72a18b622380e7033c191562cafddf528b2ca0acf9ae3071a0c5e58391f4b89c6e0a208c7b59bccb1b924dc969ced2c63d8105a01161e667fed4e9e05cdc6fd2fd7e684088ac4ab7f53ca3f1d78a34672651de83a0b6fb07d86d8868648d9a590bd28b7729143a42bca173002a599ca97935beb5caa060c35a900422f89e0d198859069c318255a6a3bbde2de3c2a82c4616febe648ba05fa9f9f55d1b1df4727a469f8c5fed99f5fa03c41e85e5910fd862edebb223a7a0f0ccedb4844f906a3f625945e8b2e1f8c89427612449e93d5c5bd98405bcc7aaa0b19a8af79ddf60dc01378271193e1dc2eda727c10cd76981924457ab8c420543a04961963e7ecd9f398dc7fdebfe8430e5a177c994d3a174a549f622190f5513c6a027c3e382e26b35145fe33fce2342fbd99e2ee6c84da7d4e2bffb228b7d11d3c2a02f082b9dabf679dabe90c6846a161b86e05ed5ceed7d0a90df02cd5bd7f2d51a80".to_string(),
                        "0xf90211a0a6da5c364983d1e2e687865457299aa8c13081d115a3d51f35662dfa7eac6256a06bb734ec57173f79b02c57330bef05932888d62ad7fc82741db70e5de9bd61ada0f89e110878f6c10309a06f3dbca32dda58d6d2b486388641a19b928eb1b92421a0b9cac6354d99f071864ff640d89808b5750410e160a4c5b58de993b33229c552a097ad71a3972c6ce83fbe80e5c9b47afe3ba4e6adf960b496418c4b89197534a0a06266177614cce023fe7930547821cf882abd5ac9a3b4ca603cbd0f947d892261a02258a7be0550675c2c7367a365a9af7c5cbd08bdcc305bb36002b7da420c30a8a0a91e6bb57b51a7013223f6f1dfba9d12eecf493c53d455676b705baa5136dedda0b9b8694642db41a63b34184bcbf5de6fc4991edf2a0443ec46423e517a6d86e4a049facc47454f6ba7873c4895e8e6f50d7f3d1d1b2d4786f7736d425be3479455a088aac371d4bbe7d3e5bc347b1d2880510788344255ba94ef91abf01c035f0ed5a02603ae41fec37271217afae503906ba86bd4061fce8432c3a5e37eb033a6df55a0009eb7c64ce5f55c615f00c1e627a91a029bccc4e4d01a45f03ca9645d1b4d21a0c993a83a7a5ddd4cccfbdc5594b2302ca2d0502a0355d8e8cb21df916b0a2ce4a0368450b4c14c0b24756db1a6d49a0c442cfba70b5c1c52d0a09b39345d8a4375a0e113f47236545878f0bfdf607ab2b981e7de092b493ce2844aa6770ee37bddcb80".to_string(),
                        "0xf90211a0b177b8f619430bc90376aae7049633893351c83434be732535d151c4c4fc6c9da0d58aeb638a0025a352f5b1918dc8813fe8d1263d014a08fcfa6ab18ef2ea3c54a069fc4557f533512e960ad4368e59bfc120a45e4208cb281a216c8785dd12eb09a029077a02afab0f25435cc5d45073b8040e08b240f94be2e8ece71448acc6c43ea0e232165d9892eaefbde628466c4632dbd222a3004b44c51074d599d38238b7b3a0203f833ce8ee72688f9624a8c221f5751ed4299d4f1b5d2a61ee3873fda6252ca0e451721694b4709cd9d2943ea9c5b456b25ce27f5ad02f27651306af1cdd106ea0aae712f99614bd0e461d39d2a69dc3c11ba1cd105c0f03e1970bcc3c1a821738a078c74f82f76a85da907851c7416ba8f70468f019ee44d42c083ab2d2b727d819a0fa1b0278b6a8e9e4b39ec8feae00a3e327fad7a55b1d7f7234470d0f731967baa095570c2a2a5baef7263820ea08e8c6d612f94ff93a5c884f72d78ad0c687a149a0ec02904efceee3647640953fe1e9b895beb240993a8da4b9eb8f17a77ad54258a0dd3df430eead5f118197fa38f4f9847068d452c17871d817af9214091d710ec3a0877c37633ca01e4032496811068cb14d0fe64aea330f583c355ef222bb04125ca000dbb8ae704b67bd38551538260e29601961c0ce53769e55246b290d5c00509ba05bffed67919c8cce7e9752face925963512290eb66118b7503bbe8df74e7253180".to_string(),
                        "0xf90211a0821aeff02ca301eb7e092e275fa6b8c115e72ec56ed690a06bbeb2f5647f0d69a0e8a97331932d7037d648a31c2b4d888221ac650520d37e2add9b96c537098c27a0b0dec54e94587c350f4f7dff5f03d0e9cd9745619a60ff4dcc7279ecf104a158a063892f2952a504af354873f1cccc86dc32a909db3755a37d14fcf07ffae7f784a0bce4c1d4e9fde0d96db53a73de689f53022c61e743b4fee13a2bda25eac1c26fa02d579a264ff31036f717769ed3f6ee42710abba3ef9c6208fff18e906f941baaa01aa910553b488a0ca451158abdbf2ca69f29d3358d4eb54bef355f8762e7d2b2a0f78e2963f6d8ccd226c3a92688b32795ec34a5ea0d486707dd038904ddd45ec2a0df5ec404a1c485dfe5a34e39a62381e175a02017be80d7aa42d86771a5522c82a056a3f06aecdfaa0c55db6a8c792431bfba2c3b3c2bf7a41122c293351c9f5220a0145bb193f9c192bd370b654e5c7b3e723332f622e791ff9a26228a55b66ae1f5a02a954bd07252e904225825a28dfd869aa3c52ad5519e4a76e3f0c8ff3dd719afa073c266d70e9614bbb1eac3a95e15239388cb47cac7a5acea7ccd40a036c65c64a0499ea61f4f8a09c34cc8ab7b72b934878c0bfc1bd918e9593bda3cccd20a5a84a074bc941cd484458b298c0117c1585835040b6b65384ccaeda58771c4155f1e8ca0aa91a0b89bba3017911a95315e0e2a7d33927d4d29e3b4a5c35e4e7fff5bafd780".to_string(),
                        "0xf901b1a06d6223af2401971b5d3667a3a58a872ea759e76582fb375e4f2de0e420df275ea0f158252d20b99f3aa36e5b97c87644eaabc50f582e769ea41cf18f7ae3602227a0a4faeacc33284fdd0eafce32e0776543e3ac349de68dfcb7abcc40b0ae82df5fa0245f6fda91c1d0dd6036c8799a9338cbf85cbbca8a3a45f35a27bb076d10cb65a080d306d21c5efccfa655b98f48b2811000fe6f60a9aebd8fdcbde7589c748e96a077499f3ba879737a73d6628cbe5d5b8ad598430031ca879cdcb3a2509d3f7d5fa0c91ebaef1a0e560845ba673efd813a313e8b9e870524adc4aa4cb6ce4eb47358a0d099e0247546af785a392f5799fb7a3eb712ca9e32dde36f49d65a61d57426e2a02aaaa42933c19eec648bef646eda705a1e84cffbe4ecd62a31862aee16e05241a06e516cdf1f81d33ffae52ca5bf785219501710b5302738b2c405127406ef3c9480a0e412c32035edec4058b02f8137c18a403f5d0274e1ca1f0beff3257f61788af8a0be49c166207007fd651f379fdd6a16bea5854e57e6fcf0109551e5d7f28f883680a04086d5b652c856858cefec39b45e2601955efa89cfcfc8d42583f954f97bcf1e8080".to_string(),
                        "0xf8518080808080a01922ad14def89076bde0011d514a50cae7632d617136bb83c1b2fcbed3383c7380808080808080a0e81a4320e846af94db949f1a5298f425864e8eecbe8b72342b0aea33c0ea6e3c808080".to_string(),
                        "0xf86c9d3fc8476432660877b666f653759ea69189b60d2f4a7008e70555746ad1b84cf84a018612309ce54000a069bbf0407f9d5438512c6218768a9581f377fa5dc119ea1409b917b75c242e1ca0eab3448e22d0f75e09ed849b2e87ac6739db4104db4eaeeffcc66cfa819755fd".to_string(),
                    ]}
              ]}],
              storages: vec![
                Storage {
                    address: "0x75cec1db9dceb703200eaa6595f66885c962b920".to_string(),
                    slot: "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
                    storage_key: "0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace".to_string(),
                    proofs: vec![MPTProof {
                        block_number: 5382810,
                        proof: vec![
                            "0xf8918080a0b7a7c859e6ddbad6c18adb60b9f48842e652021b4f8b875894b8b879568629f880a0e7f9c6d331c7d110c992550a7baa3e051adc1e26a53d928dbd517a313d221863808080808080a0e40cf9c20b1e8e4aaf3201dd3cb84ab06d2bac34e8dc3e918626e5c44c4f0707808080a0c01a2f302bfc71151daac60eeb4c1b73470845d4fe219e71644752abaafb02ab80".to_string(),"0xe9a0305787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace878609184e72a000".to_string(),
                        ],
                    }],
                }
              ],
              mmr_meta: MMRMeta {
                root: "0x7e956408569267d909a31fa404a972db1360a7e02bc3858e97c7c21ff394057".to_string(),
                size: 660813,
                id:19,
                peaks: vec!["0x06a2bfcd354f679b547aa151e4462b6bae75fd80a2a92e3767b24eab609d1d4".to_string(), "0x5967364928f2fee43c8244dc6290cd9d3ea8e9dcb4e072ef6a099e9605f241d".to_string(), "0x30d5538138ec908e6f3b6429ae49702607432c224ef10be72d23c11556f06a0".to_string(), "0x308f10140fbc6043127353ee21fab20d6c12f00ac7a8928911611b71ce5b1ab".to_string(), "0x122a500639912a0a918dc32a73b1268f3417abf6b72a6a0dc814f0986f5124d".to_string(), "0x3b2087462ad3d5c84593fdfeb72f7972695a35097e184d017470b5f99c411fd".to_string(), "0x75c56dd4e70cac0dd54944d78632700da4329824239eb1be974e3b66b56c8b9".to_string(), "0x00225132138a053a102fab30cdd9e04cdcb25ded860d7d93c2a288c7532273e".to_string(), "0x6e5d1c234047cd531f2a1406ab894f4c9487dbef207cf870cca897dea3cf5ee".to_string()],
              },
            },

        );
        init_eval_result.compiled_results.insert(
            "0x242fe0d1fa98c743f84a168ff10abbcca83cb9e0424f4541fab5041cd63d3387".to_string(),
            "10000000000000".to_string(),
        );
        init_eval_result.ordered_tasks =
            vec!["0x242fe0d1fa98c743f84a168ff10abbcca83cb9e0424f4541fab5041cd63d3387".to_string()];
        init_eval_result.encoded_tasks.insert(
            "0x242fe0d1fa98c743f84a168ff10abbcca83cb9e0424f4541fab5041cd63d3387".to_string(),
            "".to_string(),
        );
        init_eval_result.encoded_datalakes.insert(
            "0x242fe0d1fa98c743f84a168ff10abbcca83cb9e0424f4541fab5041cd63d3387".to_string(),
            EvaluatedDatalake {
                encoded_datalake: "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000052229a000000000000000000000000000000000000000000000000000000000052229a000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000350375cec1db9dceb703200eaa6595f66885c962b92000000000000000000000000000000000000000000000000000000000000000020000000000000000000000".to_string(),
                datalake_type:DatalakeType::BlockSampled,
                property_type:3,
            }
        );

        init_eval_result
    }

    #[test]
    fn test_build_merkle_tree() {
        let evaluatio_result = setup();

        let (task_merkle_tree, results_merkle_tree) = evaluatio_result.build_merkle_tree().unwrap();
        assert_eq!(
            task_merkle_tree.root().to_string(),
            "0x663d096802271660f33286d812ee13f3cda273bdf1d183d06a0119b9421151e7".to_string()
        );
        assert_eq!(
            results_merkle_tree.root().to_string(),
            "0xb540014ad1d08106489adb9d8c893947841c505f1f5794525f4cc8e5d3a92395".to_string()
        );
    }
}
