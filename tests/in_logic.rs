/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/blob-stream-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::in_logic::InLogic;
use blob_stream::protocol::{ReceiverToSenderCommands, SenderToReceiverCommands, SetChunkData};

#[test]
fn check_receive() {
    let mut logic = InLogic::new(10, 5);

    let set_chunk_data = SetChunkData {
        chunk_index: 1,
        payload: [0x8f, 0x23, 0x98, 0xfa, 0x99].into(),
    };
    let command = SenderToReceiverCommands::SetChunk(set_chunk_data);
    logic
        .receive(command)
        .expect("should be able to receive valid SetChunk");

    let answer = logic.send();
    match answer {
        ReceiverToSenderCommands::AckChunk(ack) => {
            assert_eq!(ack.waiting_for_chunk_index, 0);
            assert_eq!(ack.receive_mask_after_last, 0b1); // Indicates that chunk_index 1 was received
        }
    }
}

#[test]
fn multiple_not_received() {
    let mut logic = InLogic::new(11, 5);

    let set_chunk_data = SetChunkData {
        chunk_index: 2,
        payload: [0x8f].into(),
    };
    let command = SenderToReceiverCommands::SetChunk(set_chunk_data);
    logic
        .receive(command)
        .expect("should be able to receive valid SetChunk");

    let answer = logic.send();
    match answer {
        ReceiverToSenderCommands::AckChunk(ack) => {
            assert_eq!(ack.waiting_for_chunk_index, 0);
            assert_eq!(ack.receive_mask_after_last, 0b10); // Verifies that chunk_index 2 was received (bit 1 = index 2, bit 0 = index 1).
        }
    }
}

fn check_ack(logic: &mut InLogic, waiting_for_chunk_index: u32, receive_mask: u64) {
    let answer = logic.send();
    match answer {
        ReceiverToSenderCommands::AckChunk(ack) => {
            assert_eq!(ack.waiting_for_chunk_index, waiting_for_chunk_index);
            assert_eq!(ack.receive_mask_after_last, receive_mask); // Nothing can have been received after the chunk index 2
        }
    }
}

fn set_chunk(logic: &mut InLogic, chunk_index: u32, payload: &[u8]) {
    let set_chunk_data_2 = SetChunkData {
        chunk_index,
        payload: payload.to_vec(),
    };
    let command = SenderToReceiverCommands::SetChunk(set_chunk_data_2);
    logic
        .receive(command)
        .expect("should be able to receive valid SetChunk");
}

fn set_chunk_and_check(
    logic: &mut InLogic,
    chunk_index: u32,
    payload: &[u8],
    waiting: u32,
    receive_mask: u64,
) {
    set_chunk(logic, chunk_index, payload);
    check_ack(logic, waiting, receive_mask);
}

#[test]
fn all_received() {
    let mut logic = InLogic::new(11, 5);

    set_chunk_and_check(&mut logic, 2, &[0x8f], 0, 0b10); // Verifies that chunk_index 2 was received (bit 1 = index 2, bit 0 = index 1)
    set_chunk_and_check(&mut logic, 0, &[0x33; 5], 1, 0b1); // Verifies that chunk_index 2 was received (bit 0 = index 2)
    set_chunk_and_check(&mut logic, 1, &[0xff; 5], 3, 0b0);

    assert_eq!(
        logic.blob().expect("Blob slice should be complete"),
        &[0x33, 0x33, 0x33, 0x33, 0x33, 0xff, 0xff, 0xff, 0xff, 0xff, 0x8f]
    )
}
