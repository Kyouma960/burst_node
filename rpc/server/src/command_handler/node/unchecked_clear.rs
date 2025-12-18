use crate::command_handler::RpcCommandHandler;
use burst_rpc_messages::SuccessResponse;

impl RpcCommandHandler {
    pub(crate) fn unchecked_clear(&self) -> SuccessResponse {
        self.node.unchecked.lock().unwrap().clear();
        SuccessResponse::new()
    }
}
