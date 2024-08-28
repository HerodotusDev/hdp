use alloy::primitives::{address, B256};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use hdp::provider::{evm::provider::EvmProvider, ProofProvider};
use tokio::runtime::Runtime;

fn benchmark_header(b: &mut Bencher) {
    let provider = EvmProvider::default();
    let rt = Runtime::new().unwrap();

    b.iter(|| {
        rt.block_on(async {
            provider
                .get_range_of_header_proofs(6127485, 6127485 + 10, 1)
                .await
                .unwrap();
        });
    });
}

fn benchmark_account(b: &mut Bencher) {
    let provider = EvmProvider::default();
    let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");
    let rt = Runtime::new().unwrap();

    b.iter(|| {
        rt.block_on(async {
            provider
                .get_range_of_account_proofs(6127485, 6127485 + 10, 1, target_address)
                .await
                .unwrap();
        });
    });
}

fn benchmark_storage(b: &mut Bencher) {
    let provider = EvmProvider::default();
    let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
    let storage_key = B256::ZERO;
    let rt = Runtime::new().unwrap();

    b.iter(|| {
        rt.block_on(async {
            provider
                .get_range_of_storage_proofs(6127485, 6127485 + 10, 1, target_address, storage_key)
                .await
                .unwrap();
        });
    });
}

fn benchmark_transaction(b: &mut Bencher) {
    let provider = EvmProvider::default();
    let rt = Runtime::new().unwrap();

    b.iter(|| {
        rt.block_on(async {
            provider
                .get_tx_with_proof_from_block(6127485, 0, 23, 1)
                .await
                .unwrap();
        });
    });
}

fn benchmark_transaction_receipt(b: &mut Bencher) {
    let provider = EvmProvider::default();
    let rt = Runtime::new().unwrap();

    b.iter(|| {
        rt.block_on(async {
            provider
                .get_tx_receipt_with_proof_from_block(6127485, 0, 23, 1)
                .await
                .unwrap();
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("get_10_header_proofs", benchmark_header);
    c.bench_function("get_10_account_proofs", benchmark_account);
    c.bench_function("get_10_storage_proofs", benchmark_storage);
    c.bench_function("get_tx_with_proof_from_block", benchmark_transaction);
    c.bench_function(
        "get_tx_receipt_with_proof_from_block",
        benchmark_transaction_receipt,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::new(10, 0));
    targets = criterion_benchmark
}

criterion_main!(benches);
