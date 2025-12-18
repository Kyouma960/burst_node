use crate::command_handler::RpcCommandHandler;
use burst_rpc_messages::{AccountArg, CountResponse};
use burst_types::PublicKey;

impl RpcCommandHandler {
    pub(crate) fn delegators_count(&self, args: AccountArg) -> CountResponse {
        let representative: PublicKey = args.account.into();

        let count = self
            .node
            .ledger
            .any()
            .iter_accounts()
            .filter(|(_, info)| info.representative == representative)
            .count();

        CountResponse::new(count as u64)
    }
}
