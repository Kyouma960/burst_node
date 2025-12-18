use crate::config::NetworkConstants;
use once_cell::sync::Lazy;
use burst_ledger::LedgerConstants;
use burst_types::Networks;
use burst_work::WorkThresholds;

pub static DEV_NETWORK_PARAMS: Lazy<NetworkParams> =
    Lazy::new(|| NetworkParams::new(Networks::NanoDevNetwork));

#[derive(Clone)]
pub struct NetworkParams {
    pub work: WorkThresholds,
    pub network: NetworkConstants,
    pub ledger: LedgerConstants,
}

impl NetworkParams {
    pub fn new(network: Networks) -> Self {
        let work = WorkThresholds::default_for(network);
        let network_constants = NetworkConstants::new(work.clone(), network);
        Self {
            work: work.clone(),
            ledger: LedgerConstants::new(work.clone(), network),
            network: network_constants,
        }
    }
}
