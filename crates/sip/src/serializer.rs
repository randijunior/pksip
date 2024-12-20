use std::io::{self, Write};

use arrayvec::ArrayVec;

use crate::{msg::SipResponse, transport::MAX_PACKET_SIZE};

// packetize or encode or serialize
pub trait Serialize<'a> {
    fn serialize(&self) -> io::Result<ArrayVec<u8, MAX_PACKET_SIZE>>;
}

impl<'a> Serialize<'a> for SipResponse<'a> {
    fn serialize(&self) -> io::Result<ArrayVec<u8, MAX_PACKET_SIZE>> {
        let mut buf = ArrayVec::<u8, MAX_PACKET_SIZE>::new();

        write!(buf, "{}\r\n", self.st_line)?;
        for hdr in self.headers.iter() {
            write!(buf, "{hdr}")?;
        }
        write!(buf, "\r\n")?;

        if let Some(body) = self.body {
            if let Err(err) = buf.try_extend_from_slice(body) {
                return Err(io::Error::other(err));
            }
        }

        Ok(buf)
    }
}
