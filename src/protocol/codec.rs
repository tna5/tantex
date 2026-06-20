use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{de::DeserializeOwned, Serialize};

/// Encode a message with binary framing: [4-byte LE length][1-byte type][payload]
/// The length field covers type byte + payload.
pub fn encode_message(msg_type: u8, payload: &[u8]) -> Vec<u8> {
    let frame_len = 1 + payload.len(); // type byte + payload
    let mut buf = Vec::with_capacity(4 + frame_len);
    buf.put_u32_le(frame_len as u32);
    buf.put_u8(msg_type);
    buf.put_slice(payload);
    buf
}

/// Decode one frame from the buffer.
/// Returns Some((msg_type, payload)) if a complete frame is available.
/// Consumes the frame bytes from the buffer.
pub fn decode_frame(buf: &mut BytesMut) -> Option<(u8, Bytes)> {
    if buf.len() < 4 {
        return None;
    }

    // Peek at the length without consuming
    let len_bytes = [buf[0], buf[1], buf[2], buf[3]];
    let frame_len = u32::from_le_bytes(len_bytes) as usize;

    if buf.len() < 4 + frame_len {
        return None; // not enough data yet
    }

    // Consume the length prefix
    buf.advance(4);

    // Read type byte
    let msg_type = buf[0];
    buf.advance(1);

    // Read payload
    let payload = buf.split_to(frame_len - 1).freeze();

    Some((msg_type, payload))
}

/// Encode a request: serialize to MessagePack then frame it.
pub fn encode_request<T: Serialize>(msg_type: u8, request: &T) -> Vec<u8> {
    let payload = rmp_serde::to_vec(request).expect("Failed to serialize request");
    encode_message(msg_type, &payload)
}

/// Decode a response from MessagePack payload bytes.
pub fn decode_response<T: DeserializeOwned>(payload: &[u8]) -> Result<T, String> {
    rmp_serde::from_slice(payload).map_err(|e| format!("Failed to deserialize response: {}", e))
}
