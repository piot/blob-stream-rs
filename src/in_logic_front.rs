/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/blob-stream-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::in_logic::InLogic;
use crate::protocol::TransferId;
use crate::protocol_front::{
    AckChunkFrontData, ReceiverToSenderFrontCommands, SenderToReceiverFrontCommands,
};
use std::io;
use std::io::ErrorKind;

#[derive(Debug)]
pub struct InLogicFrontState {
    transfer_id: TransferId,
    logic: InLogic,
}

/// `InLogicFront` handles the logic for receiving and processing chunks of data
/// in a streaming context. It manages the internal state and interactions
/// between the sender and receiver commands.
#[derive(Debug, Default)]
pub struct InLogicFront {
    state: Option<InLogicFrontState>,
}

impl InLogicFront {
    /// Creates a new `InLogicFront` instance with the specified `octet_count` and `chunk_size`.
    ///
    /// # Arguments
    ///
    /// * `octet_count` - The total number of octets (bytes) expected in the stream.
    /// * `chunk_size` - The size of each chunk in the stream.
    ///
    /// # Returns
    ///
    /// A new `InLogicFront` instance.
    ///
    #[must_use]
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn update(
        &mut self,
        command: SenderToReceiverFrontCommands,
    ) -> io::Result<ReceiverToSenderFrontCommands> {
        match command {
            SenderToReceiverFrontCommands::StartTransfer(start_transfer_data) => {
                if self
                    .state
                    .as_ref()
                    .map_or(true, |s| s.transfer_id.0 != start_transfer_data.transfer_id)
                {
                    // Either logic is not set or the transfer_id is different, so we start with a fresh InLogic.
                    self.state = Some(InLogicFrontState {
                        transfer_id: TransferId(start_transfer_data.transfer_id),
                        logic: InLogic::new(
                            start_transfer_data.total_octet_size as usize,
                            start_transfer_data.chunk_size as usize,
                        ),
                    });
                }
                Ok(ReceiverToSenderFrontCommands::AckStart(
                    start_transfer_data.transfer_id,
                ))
            }
            SenderToReceiverFrontCommands::SetChunk(chunk_data) => {
                if let Some(ref mut state) = self.state {
                    let ack = state.logic.update(&chunk_data.data)?;
                    Ok(ReceiverToSenderFrontCommands::AckChunk(AckChunkFrontData {
                        transfer_id: chunk_data.transfer_id,
                        data: ack,
                    }))
                } else {
                    Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Unknown transfer_id {}", chunk_data.transfer_id.0),
                    ))
                }
            }
        }
    }
}
