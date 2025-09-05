use std::io;

use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

use super::Payload;
use crate::header::{ContentLength, HeaderParser};

//stream_oriented
#[derive(Default)]
pub(crate) struct StreamingDecoder;

impl Decoder for StreamingDecoder {
    type Error = io::Error;
    type Item = Payload;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if is keep-alive.
        if is_keep_alive(src) {
            src.advance(src.len());
            return Ok(None);
        }

        // Find header end.
        let hdr_end = b"\n\r\n";
        let pos = find_subslice(src, hdr_end);
        let Some(pos) = pos else {
            return Ok(None);
        };
        let body_start = pos + 3;
        let hdr_end = pos + 1;

        // Find "Content-Length" header
        let mut content_length = None;

        let lines = src[..hdr_end].split(|&b| b == b'\n');
        for line in lines {
            let mut split = line.splitn(2, |&c| c == b':');
            let Some(name) = split.next() else {
                continue;
            };
            if ContentLength::matches_name(name) {
                let Some(value) = split.next() else {
                    continue;
                };
                let Ok(value_str) = std::str::from_utf8(value) else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid UTF-8 in Content-Length header",
                    ));
                };
                if let Ok(parsed_value) = value_str.trim().parse::<usize>() {
                    content_length = Some(parsed_value);
                }
            }
        }

        if let Some(c_len) = content_length {
            let expected_msg_size = body_start + c_len;
            if src.len() < expected_msg_size {
                src.reserve(expected_msg_size - src.len());
                return Ok(None);
            }
            let src_bytes = src.split_to(expected_msg_size);
            let src_bytes = src_bytes.freeze();

            Ok(Some(Payload::new(src_bytes)))
        } else {
            // Return Error
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Content-Length not found",
            ))
        }
    }
}

fn find_subslice(src: &[u8], buf: &[u8]) -> Option<usize> {
    src.windows(buf.len()).position(|w| w == buf)
}

fn is_keep_alive(buf: &[u8]) -> bool {
    matches!(buf, b"\r\n\r\n" | b"\r\n")
}
