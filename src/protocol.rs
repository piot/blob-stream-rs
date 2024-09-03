/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/blob-stream-rs
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use flood_rs::{ReadOctetStream, WriteOctetStream};
use std::io;
use std::io::ErrorKind;

#[repr(u8)]
enum SenderToReceiverCommand {
    SetChunk = 0x01,
}

impl TryFrom<u8> for SenderToReceiverCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x01 => Ok(Self::SetChunk),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SetChunkData {
    pub chunk_index: u32,
    pub payload: Vec<u8>,
}

impl SetChunkData {
    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.chunk_index)?;
        stream.write_u16(self.payload.len() as u16)?;
        stream.write(&self.payload[..])?;
        Ok(())
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let chunk_index = stream.read_u32()?;
        let octet_length = stream.read_u16()?;
        let mut payload = vec![0u8; octet_length as usize];
        stream.read(&mut payload)?;

        Ok(Self {
            chunk_index,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub enum SenderToReceiverCommands {
    SetChunk(SetChunkData),
}

impl SenderToReceiverCommands {
    #[must_use]
    pub const fn to_octet(&self) -> u8 {
        match self {
            Self::SetChunk(_) => SenderToReceiverCommand::SetChunk as u8,
        }
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.

    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> std::io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            Self::SetChunk(set_chunk_header) => set_chunk_header.to_stream(stream),
        }
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> std::io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = SenderToReceiverCommand::try_from(command_value)?;
        let x = match command {
            SenderToReceiverCommand::SetChunk => Self::SetChunk(SetChunkData::from_stream(stream)?),
        };
        Ok(x)
    }
}

// ---------- Receiver

#[repr(u8)]
enum ReceiverToSenderCommand {
    AckChunk = 0x02,
}

impl TryFrom<u8> for ReceiverToSenderCommand {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            0x02 => Ok(Self::AckChunk),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Unknown command {value}"),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AckChunkData {
    pub waiting_for_chunk_index: u32, // first chunk index that remote has not received fully in sequence. (first gap in chunks from the start).
    pub receive_mask_after_last: u64, // receive bit mask for chunks after the `waiting_for_chunk_index`
}

impl AckChunkData {
    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u32(self.waiting_for_chunk_index)?;
        stream.write_u64(self.receive_mask_after_last)?;
        Ok(())
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        Ok(Self {
            waiting_for_chunk_index: stream.read_u32()?,
            receive_mask_after_last: stream.read_u64()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ReceiverToSenderCommands {
    AckChunk(AckChunkData),
}

impl ReceiverToSenderCommands {
    #[must_use]
    pub const fn to_octet(&self) -> u8 {
        match self {
            Self::AckChunk(_) => ReceiverToSenderCommand::AckChunk as u8,
        }
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn to_stream(&self, stream: &mut dyn WriteOctetStream) -> io::Result<()> {
        stream.write_u8(self.to_octet())?;
        match self {
            Self::AckChunk(set_chunk_header) => set_chunk_header.to_stream(stream),
        }
    }

    /// # Errors
    ///
    /// This function will return an `io::Error` if there is an issue with writing to the stream.
    /// This could happen if the stream is closed or if there are underlying I/O errors during the write operation.
    pub fn from_stream(stream: &mut dyn ReadOctetStream) -> io::Result<Self> {
        let command_value = stream.read_u8()?;
        let command = ReceiverToSenderCommand::try_from(command_value)?;
        let x = match command {
            ReceiverToSenderCommand::AckChunk => Self::AckChunk(AckChunkData::from_stream(stream)?),
        };
        Ok(x)
    }
}
