/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/blob-stream-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use blob_stream::prelude::*;

#[test]
fn start_transfer() {
    let start_transfer = SenderToReceiverFrontCommands::StartTransfer(StartTransferData {
        transfer_id: 1,
        total_octet_size: 8,
        chunk_size: 2,
    });

    let mut logic = FrontLogic::new();

    let answer = logic
        .update(&start_transfer)
        .expect("start transfer should work");

    let expected_answer = ReceiverToSenderFrontCommands::AckStart(1);

    assert_eq!(answer, expected_answer);
}

#[test]
fn drop_previous_transfer() {
    let start_transfer = SenderToReceiverFrontCommands::StartTransfer(StartTransferData {
        transfer_id: 1,
        total_octet_size: 8,
        chunk_size: 2,
    });

    let mut logic = FrontLogic::new();

    {
        let answer = logic
            .update(&start_transfer)
            .expect("start transfer should work");

        let expected_answer = ReceiverToSenderFrontCommands::AckStart(1);

        assert_eq!(answer, expected_answer);
    }

    {
        let new_transfer = SenderToReceiverFrontCommands::StartTransfer(StartTransferData {
            transfer_id: 2,
            total_octet_size: 8,
            chunk_size: 2,
        });

        let answer = logic
            .update(&new_transfer)
            .expect("it should accept new transfer");

        let expected_answer = ReceiverToSenderFrontCommands::AckStart(2);

        assert_eq!(answer, expected_answer);
    }
}

fn set_chunk_and_check(
    logic: &mut FrontLogic,
    transfer_id: u16,
    chunk_index: u32,
    payload: &[u8],
    waiting: u32,
    receive_mask: u64,
) {
    let set_chunk_data = SetChunkData {
        chunk_index,
        payload: payload.to_vec(),
    };
    let set_chunk_front = SetChunkFrontData {
        transfer_id: TransferId(transfer_id),
        data: set_chunk_data,
    };
    let set_chunk_command = SenderToReceiverFrontCommands::SetChunk(set_chunk_front);

    let ack = logic
        .update(&set_chunk_command)
        .expect("update should work");
    match ack {
        ReceiverToSenderFrontCommands::AckChunk(ack) => {
            assert_eq!(ack.data.waiting_for_chunk_index, waiting);
            assert_eq!(ack.data.receive_mask_after_last, receive_mask);
        }
        _ => panic!("unexpected response"),
    }
}

#[test]
fn complete_transfer() {
    const TRANSFER_ID_VALUE: u16 = 0x3211;
    const TRANSFER_ID: TransferId = TransferId(TRANSFER_ID_VALUE);
    let start_transfer = SenderToReceiverFrontCommands::StartTransfer(StartTransferData {
        transfer_id: TRANSFER_ID.0,
        total_octet_size: 9,
        chunk_size: 4,
    });

    let mut logic = FrontLogic::new();

    {
        let answer = logic
            .update(&start_transfer)
            .expect("start transfer should work");

        let expected_answer = ReceiverToSenderFrontCommands::AckStart(TRANSFER_ID_VALUE);

        assert_eq!(answer, expected_answer);
    }

    set_chunk_and_check(
        &mut logic,
        TRANSFER_ID_VALUE,
        1,
        &[0xff, 0x11, 0xfe, 0x22],
        0,
        0b1,
    );
    set_chunk_and_check(
        &mut logic,
        TRANSFER_ID_VALUE,
        0,
        &[0xba, 0xbc, 0xbd, 0xbe],
        2,
        0b0,
    );
    set_chunk_and_check(&mut logic, TRANSFER_ID_VALUE, 2, &[0x42], 3, 0b0);

    assert_eq!(
        logic
            .blob()
            .expect("blob should be ready after receiving three chunks"),
        &[0xba, 0xbc, 0xbd, 0xbe, 0xff, 0x11, 0xfe, 0x22, 0x42]
    );
}
