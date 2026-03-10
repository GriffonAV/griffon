use std::io;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::ipc_header::{Frame, MsgType};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfoDto {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterfaceRequest {
    Ping,
    ListPlugins,
    RefreshPlugins,
    RestartPlugin { pid: u32 },
    KillPlugin { pid: u32 },
    CallPlugin {
        pid: u32,
        fn_name: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterfaceResponse {
    Pong,
    Ok,
    Error { message: String },
    Plugins(Vec<PluginInfoDto>),
    CallAccepted { request_id: u32 },
    CallResult {
        request_id: u32,
        ok: bool,
        output: String,
    },
}

pub fn send_interface_request<W: Write>(w: &mut W, req: &InterfaceRequest) -> io::Result<()> {
    let payload = to_cbor(req)?;
    let frame = Frame::new(MsgType::InterfaceRequest, 0, payload);
    frame.write_to(w)
}

pub fn recv_interface_request<R: Read>(r: &mut R) -> io::Result<InterfaceRequest> {
    let frame = Frame::read_from(r)?;

    match frame.mtype {
        MsgType::InterfaceRequest => from_cbor(&frame.payload),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected InterfaceRequest",
        )),
    }
}

pub fn send_interface_response<W: Write>(w: &mut W, resp: &InterfaceResponse) -> io::Result<()> {
    let payload = to_cbor(resp)?;
    let frame = Frame::new(MsgType::InterfaceResponse, 0, payload);
    frame.write_to(w)
}

pub fn recv_interface_response<R: Read>(r: &mut R) -> io::Result<InterfaceResponse> {
    let frame = Frame::read_from(r)?;

    match frame.mtype {
        MsgType::InterfaceResponse => from_cbor(&frame.payload),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected InterfaceResponse",
        )),
    }
}

fn to_cbor<T: Serialize>(v: &T) -> io::Result<Vec<u8>> {
    serde_cbor::to_vec(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn from_cbor<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> io::Result<T> {
    serde_cbor::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}