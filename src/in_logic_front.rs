/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/blob-stream-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use crate::in_logic::InLogic;
use crate::protocol::{ReceiverToSenderCommands, SenderToReceiverCommands, TransferId};
use crate::protocol_front::{
    AckChunkFrontData, ReceiverToSenderFrontCommands, SenderToReceiverFrontCommands,
};
use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;

/// `InLogicFront` handles the logic for receiving and processing chunks of data
/// in a streaming context. It manages the internal state and interactions
/// between the sender and receiver commands.
#[derive(Debug, Default)]
pub struct InLogicFront {
    transfers: HashMap<u16, InLogic>,
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
        Self {
            transfers: HashMap::default(),
        }
    }

    /// Processes a `SenderToReceiverCommands` command, applying it to the internal stream.
    ///
    /// Currently, this function only handles the `SetChunk` command, which updates the
    /// stream with a new chunk of data.
    ///
    /// # Arguments
    ///
    /// * `command` - The command sent by the sender, containing the chunk data.
    ///
    /// # Errors
    ///
    /// Returns an `io::Result<()>` if the chunk cannot be set due to an I/O error.
    ///
    /// # Example
    pub fn receive(&mut self, command: SenderToReceiverFrontCommands) -> io::Result<()> {
        match command {
            SenderToReceiverFrontCommands::StartTransfer(start_transfer_data) => {
                self.transfers
                    .entry(start_transfer_data.transfer_id)
                    .or_insert_with(|| {
                        InLogic::new(
                            start_transfer_data.total_octet_size as usize,
                            start_transfer_data.chunk_size as usize,
                        )
                    });
                Ok(())
            }
            SenderToReceiverFrontCommands::SetChunk(chunk_data) => {
                if let Some(found) = self.transfers.get_mut(&chunk_data.transfer_id.0) {
                    found.receive(SenderToReceiverCommands::SetChunk(chunk_data.data))
                } else {
                    Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Unknown transfer_id {}", chunk_data.transfer_id.0),
                    ))
                }
            }
        }
    }

    /// Generates a `ReceiverToSenderCommands` command to acknowledge the received chunks.
    ///
    /// This function determines the next chunk index expected by the receiver and
    /// generates a receive-mask indicating which chunks have been received.
    ///
    /// # Returns
    ///
    /// A `ReceiverToSenderCommands::AckChunk` containing the next expected chunk index
    /// and the receive-mask for the subsequent chunks.
    ///
    /// # Example
    ///
    /// ```
    /// use blob_stream::in_logic_front::InLogicFront;
    /// let mut in_logic = InLogicFront::new();
    /// let ack_command = in_logic.send();
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn send(&self) -> Vec<ReceiverToSenderFrontCommands> {
        let mut commands = Vec::new();
        for (transfer_id, in_logic) in &self.transfers {
            let receiver_to_front = in_logic.send();
            match receiver_to_front {
                ReceiverToSenderCommands::AckChunk(ack_chunk) => {
                    let ack_chunk_front = AckChunkFrontData {
                        transfer_id: TransferId(*transfer_id),
                        data: ack_chunk,
                    };
                    commands.push(ReceiverToSenderFrontCommands::AckChunk(ack_chunk_front));
                } // Handle other variants of ReceiverToSenderCommands if needed
            }
        }
        commands
    }
}
