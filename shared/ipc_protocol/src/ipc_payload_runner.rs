use std::io;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

use crate::ipc_header::{Frame, MsgType};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloOkPayload {
    pub name: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallPayload {
    pub fn_name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultPayload {
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: u32,
    pub message: String,
}

#[derive(Debug)]
pub enum Message {
    Hello,
    HelloOk(HelloOkPayload),

    Call {
        request_id: u32,
        data: CallPayload,
    },

    Result {
        request_id: u32,
        data: ResultPayload,
    },

    Error {
        request_id: u32,
        data: ErrorPayload,
    },

    Heartbeat,
}

impl Message {
    pub fn into_frame(self) -> io::Result<Frame> {
        match self {
            Message::Hello => Ok(Frame::new(MsgType::Hello, 0, Vec::new())),
            Message::Heartbeat => Ok(Frame::new(MsgType::Heartbeat, 0, Vec::new())),

            Message::HelloOk(p) => Ok(Frame::new(MsgType::HelloOk, 0, to_cbor(&p)?)),

            Message::Call { request_id, data } => {
                Ok(Frame::new(MsgType::Call, request_id, to_cbor(&data)?))
            }

            Message::Result { request_id, data } => {
                Ok(Frame::new(MsgType::Result, request_id, to_cbor(&data)?))
            }

            Message::Error { request_id, data } => {
                Ok(Frame::new(MsgType::Error, request_id, to_cbor(&data)?))
            }
        }
    }
}

pub fn send_message<W: Write>(w: &mut W, msg: Message) -> io::Result<()> {
    let frame = msg.into_frame()?;
    frame.write_to(w)
}

pub fn recv_message<R: Read>(r: &mut R) -> io::Result<Message> {
    let frame = Frame::read_from(r)?;
    decode_frame(frame)
}

pub fn decode_frame(frame: Frame) -> io::Result<Message> {
    match frame.mtype {
        MsgType::Hello => Ok(Message::Hello),
        MsgType::Heartbeat => Ok(Message::Heartbeat),

        MsgType::HelloOk => {
            let p: HelloOkPayload = from_cbor(&frame.payload)?;
            Ok(Message::HelloOk(p))
        }

        MsgType::Call => {
            let p: CallPayload = from_cbor(&frame.payload)?;
            Ok(Message::Call {
                request_id: frame.request_id,
                data: p,
            })
        }

        MsgType::Result => {
            let p: ResultPayload = from_cbor(&frame.payload)?;
            Ok(Message::Result {
                request_id: frame.request_id,
                data: p,
            })
        }

        MsgType::Error => {
            let p: ErrorPayload = from_cbor(&frame.payload)?;
            Ok(Message::Error {
                request_id: frame.request_id,
                data: p,
            })
        }

        // You can implement Log later:
        // MsgType::Log => ...
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported message type",
        )),
    }
}


fn to_cbor<T: Serialize>(v: &T) -> io::Result<Vec<u8>> {
    serde_cbor::to_vec(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn from_cbor<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> io::Result<T> {
    serde_cbor::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
