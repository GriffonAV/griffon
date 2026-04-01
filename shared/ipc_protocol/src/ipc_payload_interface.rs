use std::io;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::ipc_header::{Frame, MsgType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginInfoDto {
    pub pid: u32,
    pub status: bool,
    pub plugin_uuid: [u8; 16],
    pub name: String,
    pub path: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterfaceRequest {
    RefreshPlugins,
    StopPlugin {
        plugin_uuid: [u8; 16],
    },
    StartPlugin {
        plugin_uuid: [u8; 16],
    },
    KillPlugin {
        plugin_uuid: [u8; 16],
    },
    CallPlugin {
        plugin_uuid: [u8; 16],
        fn_name: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterfaceResponse {
    Ok,
    Error {
        request_id: u32,
        message: String,
    },
    Plugins {
        plugins: Vec<PluginInfoDto>,
    },
    CallAccepted {
        request_id: u32,
    },
    CallResult {
        request_id: u32,
        ok: bool,
        output: String,
    },
}

pub fn format_uuid_bytes(uuid: &[u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        uuid[0],
        uuid[1],
        uuid[2],
        uuid[3],
        uuid[4],
        uuid[5],
        uuid[6],
        uuid[7],
        uuid[8],
        uuid[9],
        uuid[10],
        uuid[11],
        uuid[12],
        uuid[13],
        uuid[14],
        uuid[15],
    )
}

pub fn send_interface_request<W: Write>(
    w: &mut W,
    req: &InterfaceRequest,
    request_id: u32,
) -> io::Result<()> {
    let payload = to_cbor(req)?;
    let frame = Frame::new(MsgType::InterfaceRequest, request_id, payload);
    frame.write_to(w)
}

pub fn recv_interface_request<R: Read>(r: &mut R) -> io::Result<(Frame, InterfaceRequest)> {
    let frame = Frame::read_from(r)?;

    let req = match frame.mtype {
        MsgType::InterfaceRequest => from_cbor(&frame.payload)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected InterfaceRequest",
            ));
        }
    };

    Ok((frame, req))
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
