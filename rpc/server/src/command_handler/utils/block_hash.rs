use burst_rpc_messages::{BlockHashArgs, HashRpcMessage};
use burst_types::Block;

pub fn block_hash(args: BlockHashArgs) -> HashRpcMessage {
    let block_enum: Block = args.block.into();
    HashRpcMessage::new(block_enum.hash())
}
