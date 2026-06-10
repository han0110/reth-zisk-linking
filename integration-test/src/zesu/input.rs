//! Conversion of a reth stateless fixture into the schema StatelessInput.

use anyhow::{Result, anyhow};
use libssz_types::SszList;
use stateless_validator_reth::guest::StatelessValidatorRethInput;

use crate::stateless_ssz::{
    BlobSchedule, ChainConfig, ExecutionWitness, ForkActivation, ForkConfig, NewPayloadRequest,
    StatelessInput,
};

pub fn from_fixture(
    stateless_input: &stateless::StatelessInput,
    valid_block: bool,
) -> Result<StatelessInput> {
    let reth_input = StatelessValidatorRethInput::new(stateless_input, valid_block)?;
    let header = &stateless_input.block.header;
    Ok(StatelessInput {
        new_payload_request: NewPayloadRequest(reth_input.new_payload_request),
        witness: ExecutionWitness {
            state: bytes_list(&stateless_input.witness.state, "witness state")?,
            codes: bytes_list(&stateless_input.witness.codes, "witness codes")?,
            headers: bytes_list(&stateless_input.witness.headers, "witness headers")?,
        },
        chain_config: ChainConfig {
            chain_id: stateless_input.chain_config.chain_id,
            active_fork: active_fork(
                &stateless_input.chain_config,
                header.number,
                header.timestamp,
            )?,
        },
        public_keys: ssz_list(
            reth_input.public_keys.iter().map(|key| key.0).collect(),
            "public keys",
        )?,
    })
}

fn active_fork(
    config: &alloy_genesis::ChainConfig,
    block_number: u64,
    timestamp: u64,
) -> Result<ForkConfig> {
    let by_time = |at: Option<u64>| -> (Option<u64>, Option<u64>) { (None, at) };
    let by_block = |at: Option<u64>| -> (Option<u64>, Option<u64>) { (at, None) };
    let forks = [
        (by_time(config.amsterdam_time), 24, "amsterdam"),
        (by_time(config.bpo5_time), 23, "bpo5"),
        (by_time(config.bpo4_time), 22, "bpo4"),
        (by_time(config.bpo3_time), 21, "bpo3"),
        (by_time(config.bpo2_time), 20, "bpo2"),
        (by_time(config.bpo1_time), 19, "bpo1"),
        (by_time(config.osaka_time), 18, "osaka"),
        (by_time(config.prague_time), 17, "prague"),
        (by_time(config.cancun_time), 16, "cancun"),
        (by_time(config.shanghai_time), 15, ""),
        (by_block(config.merge_netsplit_block), 14, ""),
        (by_block(config.gray_glacier_block), 13, ""),
        (by_block(config.arrow_glacier_block), 12, ""),
        (by_block(config.london_block), 11, ""),
        (by_block(config.berlin_block), 10, ""),
        (by_block(config.muir_glacier_block), 9, ""),
        (by_block(config.istanbul_block), 8, ""),
        (by_block(config.petersburg_block), 7, ""),
        (by_block(config.constantinople_block), 6, ""),
        (by_block(config.byzantium_block), 5, ""),
        (by_block(config.eip158_block.or(config.eip155_block)), 4, ""),
        (by_block(config.eip150_block), 3, ""),
        (by_block(config.dao_fork_block), 2, ""),
        (by_block(config.homestead_block), 1, ""),
    ];
    let ((at_block, at_time), fork, schedule_key) = forks
        .into_iter()
        .find(|((at_block, at_time), _, _)| {
            at_block.is_some_and(|at| block_number >= at)
                || at_time.is_some_and(|at| timestamp >= at)
        })
        .unwrap_or(((Some(0), None), 0, ""));
    let blob_schedule = config
        .blob_schedule
        .get(schedule_key)
        .map(|params| {
            Ok::<_, anyhow::Error>(BlobSchedule {
                target: params.target_blob_count,
                max: params.max_blob_count,
                base_fee_update_fraction: u64::try_from(params.update_fraction)
                    .map_err(|_| anyhow!("update fraction exceeds u64"))?,
            })
        })
        .transpose()?;
    Ok(ForkConfig {
        fork,
        activation: ForkActivation {
            block_number: ssz_list(at_block.into_iter().collect(), "activation block number")?,
            timestamp: ssz_list(at_time.into_iter().collect(), "activation timestamp")?,
        },
        blob_schedule: ssz_list(blob_schedule.into_iter().collect(), "blob schedule")?,
    })
}

fn bytes_list<const N: usize, const M: usize>(
    items: &[alloy_primitives::Bytes],
    name: &str,
) -> Result<SszList<SszList<u8, N>, M>> {
    let items = items
        .iter()
        .map(|item| ssz_list(item.to_vec(), name))
        .collect::<Result<_>>()?;
    ssz_list(items, name)
}

fn ssz_list<T, const N: usize>(items: Vec<T>, name: &str) -> Result<SszList<T, N>> {
    SszList::try_from(items).map_err(|error| anyhow!("{name} exceeds the schema limit, {error:?}"))
}
