use std::io::{self, Read, Write};

pub const MAGIC: u16 = 0xBEEF;
pub const VERSION: u8 = 1;

pub const HEADER_LEN: usize = 12; // 2 + 1 + 1 + 4 + 4
pub const MAX_PAYLOAD: u32 = 1024 * 1024; // 1MB cap

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum MsgType {
    Hello = 1,
    HelloOk = 2,
    Call = 3,
    Result = 4,
    Log = 5,
    Heartbeat = 6,
    Error = 7,

    InterfaceRequest = 8,
    InterfaceResponse = 9,
    InterfaceEvent = 10,
}

impl MsgType {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            1 => MsgType::Hello,
            2 => MsgType::HelloOk,
            3 => MsgType::Call,
            4 => MsgType::Result,
            5 => MsgType::Log,
            6 => MsgType::Heartbeat,
            7 => MsgType::Error,

            8 => MsgType::InterfaceRequest,
            9 => MsgType::InterfaceResponse,
            10 => MsgType::InterfaceEvent,

            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub version: u8,
    pub mtype: MsgType,
    pub request_id: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(mtype: MsgType, request_id: u32, payload: Vec<u8>) -> Self {
        Self {
            version: VERSION,
            mtype,
            request_id,
            payload,
        }
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        if self.payload.len() as u32 > MAX_PAYLOAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "payload too large",
            ));
        }

        let mut header = [0u8; HEADER_LEN];
        header[0..2].copy_from_slice(&MAGIC.to_be_bytes());
        header[2] = self.version;
        header[3] = self.mtype as u8;
        header[4..8].copy_from_slice(&self.request_id.to_be_bytes());
        header[8..12].copy_from_slice(&(self.payload.len() as u32).to_be_bytes());

        w.write_all(&header)?;
        w.write_all(&self.payload)?;
        Ok(())
    }

    pub fn read_from<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut header = [0u8; HEADER_LEN];
        r.read_exact(&mut header)?;

        let magic = u16::from_be_bytes([header[0], header[1]]);
        if magic != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic"));
        }

        let version = header[2];
        if version != VERSION {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad version"));
        }

        let mtype = MsgType::from_u8(header[3])
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad type"))?;

        let request_id = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let len = u32::from_be_bytes(header[8..12].try_into().unwrap());

        if len > MAX_PAYLOAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "payload too large",
            ));
        }

        let mut payload = vec![0u8; len as usize];
        r.read_exact(&mut payload)?;

        Ok(Frame {
            version,
            mtype,
            request_id,
            payload,
        })
    }
}
