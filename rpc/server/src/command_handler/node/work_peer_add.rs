use crate::command_handler::RpcCommandHandler;
use burst_rpc_messages::{AddressWithPortArgs, SuccessResponse};
use burst_types::Peer;

impl RpcCommandHandler {
    pub(crate) fn work_peer_add(&self, args: AddressWithPortArgs) -> SuccessResponse {
        self.node
            .work_factory
            .add_peer(Peer::new(args.address, args.port.into()));
        SuccessResponse::new()
    }
}
