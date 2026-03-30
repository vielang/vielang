use bytes::{BufMut, BytesMut};
use mqttbytes::v5::{ConnAck, Packet, SubAck, SubscribeReasonCode};
use mqttbytes::QoS;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::TransportError;

const MAX_PACKET_SIZE: usize = 10 * 1024 * 1024; // 10 MB

/// Đọc một MQTT packet từ async reader.
/// Tự buffer bytes cho đến khi đủ một packet hoàn chỉnh.
/// Dùng cho mọi packet kể cả CONNECT (mqttbytes 0.6 có Login fields public).
pub async fn read_packet<R: AsyncRead + Unpin>(
    reader: &mut R,
    buf: &mut BytesMut,
) -> Result<Packet, TransportError> {
    loop {
        match mqttbytes::v5::read(buf, MAX_PACKET_SIZE) {
            Ok(packet) => return Ok(packet),
            Err(mqttbytes::Error::InsufficientBytes(_)) => {
                let n = reader.read_buf(buf).await?;
                if n == 0 {
                    return Err(TransportError::Io(std::io::Error::from(
                        std::io::ErrorKind::ConnectionReset,
                    )));
                }
            }
            Err(e) => return Err(TransportError::Protocol(e.to_string())),
        }
    }
}

pub async fn write_connack<W: AsyncWrite + Unpin>(
    writer: &mut W,
    connack: ConnAck,
) -> Result<(), TransportError> {
    let mut buf = BytesMut::new();
    connack
        .write(&mut buf)
        .map_err(|e| TransportError::Protocol(e.to_string()))?;
    writer.write_all(&buf).await?;
    Ok(())
}

/// MQTT PINGRESP: fixed header 0xD0 + remaining length 0x00
pub async fn write_pingresp<W: AsyncWrite + Unpin>(
    writer: &mut W,
) -> Result<(), TransportError> {
    writer.write_all(&[0xD0, 0x00]).await?;
    Ok(())
}

/// MQTT PUBACK cho QoS 1
pub async fn write_puback<W: AsyncWrite + Unpin>(
    writer: &mut W,
    pkid: u16,
) -> Result<(), TransportError> {
    let buf = [0x40, 0x02, (pkid >> 8) as u8, (pkid & 0xFF) as u8];
    writer.write_all(&buf).await?;
    Ok(())
}

/// MQTT SUBACK — grant QoS 0 cho tất cả filters
pub async fn write_suback<W: AsyncWrite + Unpin>(
    writer: &mut W,
    pkid: u16,
    n_filters: usize,
) -> Result<(), TransportError> {
    let mut buf = BytesMut::new();
    let suback = SubAck {
        pkid,
        return_codes: vec![SubscribeReasonCode::QoS0; n_filters],
        properties: None,
    };
    suback
        .write(&mut buf)
        .map_err(|e| TransportError::Protocol(e.to_string()))?;
    writer.write_all(&buf).await?;
    Ok(())
}

/// Gửi PUBLISH từ server → device (dùng cho attribute response)
pub async fn write_publish<W: AsyncWrite + Unpin>(
    writer: &mut W,
    topic: &str,
    payload: &[u8],
) -> Result<(), TransportError> {
    use mqttbytes::{v5::Publish, QoS};
    let mut buf = BytesMut::new();
    let publish = Publish::new(topic, QoS::AtMostOnce, payload.to_vec());
    publish
        .write(&mut buf)
        .map_err(|e| TransportError::Protocol(e.to_string()))?;
    writer.write_all(&buf).await?;
    Ok(())
}

// ── Synchronous encode functions (for write task pattern) ────────────────────

/// Encode CONNACK to bytes (synchronous, for write task pattern)
pub fn encode_connack(connack: ConnAck) -> bytes::Bytes {
    let mut buf = BytesMut::new();
    connack.write(&mut buf).unwrap_or_default();
    buf.freeze()
}

/// Encode PINGRESP to bytes
pub fn encode_pingresp() -> bytes::Bytes {
    bytes::Bytes::from_static(&[0xD0, 0x00])
}

/// Encode PUBACK to bytes (QoS 1 ack)
pub fn encode_puback(pkid: u16) -> bytes::Bytes {
    let mut buf = BytesMut::with_capacity(4);
    buf.put_u8(0x40);
    buf.put_u8(0x02);
    buf.put_u16(pkid);
    buf.freeze()
}

/// Encode SUBACK to bytes
pub fn encode_suback(pkid: u16, n_filters: usize) -> bytes::Bytes {
    let mut buf = BytesMut::new();
    let suback = SubAck {
        pkid,
        return_codes: vec![SubscribeReasonCode::QoS0; n_filters],
        properties: None,
    };
    suback.write(&mut buf).unwrap_or_default();
    buf.freeze()
}

/// Encode PUBLISH to bytes (server→device)
pub fn encode_publish(topic: &str, payload: &[u8]) -> bytes::Bytes {
    use mqttbytes::v4::Publish;
    let mut buf = BytesMut::new();
    let publish = Publish::new(topic, QoS::AtMostOnce, payload.to_vec());
    publish.write(&mut buf).unwrap_or_default();
    buf.freeze()
}

/// Encode PUBREC to bytes (QoS 2 step 2)
pub fn encode_pubrec(pkid: u16) -> bytes::Bytes {
    let mut buf = BytesMut::with_capacity(4);
    buf.put_u8(0x50);
    buf.put_u8(0x02);
    buf.put_u16(pkid);
    buf.freeze()
}

/// Encode PUBCOMP to bytes (QoS 2 step 4)
pub fn encode_pubcomp(pkid: u16) -> bytes::Bytes {
    let mut buf = BytesMut::with_capacity(4);
    buf.put_u8(0x70);
    buf.put_u8(0x02);
    buf.put_u16(pkid);
    buf.freeze()
}
