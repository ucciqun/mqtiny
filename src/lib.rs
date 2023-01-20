use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum MqttPacket {
    Publish(MqttPublishPacket),
    Puback(MqttPubackPacket),
    Subscribe(MqttSubscribePacket),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MqttPublishPacket {
    pub topic_name: u16,
    pub qos: QoS,
    pub payload: Vec<u8>,
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MqttPubackPacket {
    pub payload: Vec<u8>,
}
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct MqttSubscribePacket {
    pub topic_name: u16,
}

pub fn parse_publish_packet(flags: u8, data: &[u8]) -> MqttPublishPacket {
    let mut cursor = Cursor::new(data);
    cursor.advance(2); // fixed header

    let qos = QoS::from_usize(((flags & 0x06) >> 1).into()).unwrap();
    let topic_name = cursor.get_u16();
    let payload = cursor.into_inner();

    MqttPublishPacket {
        topic_name,
        qos,
        payload: payload.to_vec(),
    }
}

pub fn parse_subscribe_packet(_flags: u8, data: &[u8]) -> MqttSubscribePacket {
    let mut cursor = Cursor::new(data);
    cursor.advance(2); // fixed header

    let topic_name = cursor.get_u16();

    MqttSubscribePacket { topic_name }
}

pub struct MQTinyCodec {}
impl Decoder for MQTinyCodec {
    type Item = MqttPacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }
        // println!("src.len: {}", src.len());

        let packet_type = src[0] >> 4;
        let packet_flags = src[0] & 0x0F;
        let remaining_length = src[1] as usize;

        if src.len() < remaining_length + 2 {
            src.reserve(remaining_length + 2 - src.len());

            return Ok(None);
        }

        let packet_data = src.split_to(remaining_length + 2).freeze();
        // println!("packet_type: {}", packet_type);
        // println!("packet_flags: {}", packet_flags);
        // println!("remaining_length: {}", remaining_length);
        let packet = match packet_type {
            3 => MqttPacket::Publish(parse_publish_packet(packet_flags, &packet_data)),
            8 => MqttPacket::Subscribe(parse_subscribe_packet(packet_flags, &packet_data)),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "undefined packet",
                ))
            }
        };

        return Ok(Some(packet));
    }
}
impl Encoder<MqttPacket> for MQTinyCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: MqttPacket, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match item {
            MqttPacket::Publish(publish) => {
                dst.put(&publish.payload[..]);
                Ok(())
            }
            MqttPacket::Subscribe(_) => todo!(),
            MqttPacket::Puback(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum PacketType {
    Unknown = 0,
    Connect = 1,
    Connack = 2,
    Publish = 3,
    Puback = 4,
    Pubrec = 5,
    Pubrel = 6,
    Pubcomp = 7,
    Subscribe = 8,
    Suback = 9,
    Unsubscribe = 10,
    Unsuback = 11,
    Pingreq = 12,
    Pingresp = 13,
    Disconnect = 14,
    Reserved = 15,
}

impl PacketType {
    pub fn from_usize(n: usize) -> Option<PacketType> {
        match n {
            0 => Some(PacketType::Unknown),
            1 => Some(PacketType::Connect),
            2 => Some(PacketType::Connack),
            3 => Some(PacketType::Publish),
            4 => Some(PacketType::Puback),
            5 => Some(PacketType::Pubrec),
            6 => Some(PacketType::Pubrel),
            7 => Some(PacketType::Pubcomp),
            8 => Some(PacketType::Subscribe),
            9 => Some(PacketType::Suback),
            10 => Some(PacketType::Unsubscribe),
            11 => Some(PacketType::Unsuback),
            12 => Some(PacketType::Pingreq),
            13 => Some(PacketType::Pingresp),
            14 => Some(PacketType::Disconnect),
            15 => Some(PacketType::Reserved),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl QoS {
    pub fn from_usize(n: usize) -> Option<QoS> {
        match n {
            0 => Some(QoS::AtMostOnce),
            1 => Some(QoS::AtLeastOnce),
            2 => Some(QoS::ExactlyOnce),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct Packet {
    pub fixed_header: FixedHeader,
    pub variable_header: VariableHeader,
    pub payload: Option<Bytes>,
}

#[derive(Debug)]
#[allow(unused)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub qos: QoS,
    pub remaining_length: u8,
}

#[derive(Debug)]
#[allow(unused)]
pub struct VariableHeader {
    pub topic_name: u16,
}
