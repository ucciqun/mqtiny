use bytes::Bytes;

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
