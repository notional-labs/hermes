use core::str::FromStr;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use tendermint_rpc::abci::tag::Tag;
use tendermint_rpc::abci::Event as AbciEvent;

use super::error::Error;
use crate::applications::transfer::coin::RawCoin;
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::events::IbcEventType;
use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct IncentivizedPacket {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
    pub total_recv_fee: Vec<RawCoin>,
    pub total_ack_fee: Vec<RawCoin>,
    pub total_timeout_fee: Vec<RawCoin>,
}

fn find_value(key: &str, entries: &[Tag]) -> Result<String, Error> {
    entries
        .iter()
        .find_map(|entry| {
            if entry.key.as_ref() == key {
                Some(entry.value.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| Error::event_attribute_not_found(key.to_string()))
}

fn new_tag(key: &str, value: &str) -> Tag {
    Tag {
        key: key.parse().unwrap(),
        value: value.parse().unwrap(),
    }
}

impl From<IncentivizedPacket> for AbciEvent {
    fn from(event: IncentivizedPacket) -> AbciEvent {
        let attributes: Vec<Tag> = vec![
            new_tag("port_id", event.port_id.as_str()),
            new_tag("channel_id", event.channel_id.as_ref()),
            new_tag("packet_sequence", &event.sequence.to_string()),
            new_tag("recv_fee", &event.total_recv_fee.iter().join(",")),
            new_tag("ack_fee", &event.total_ack_fee.iter().join(",")),
            new_tag("timeout_fee", &event.total_timeout_fee.iter().join(",")),
        ];

        AbciEvent {
            type_str: IbcEventType::IncentivizedPacket.as_str().to_string(),
            attributes,
        }
    }
}

impl<'a> TryFrom<&'a Vec<Tag>> for IncentivizedPacket {
    type Error = Error;

    fn try_from(entries: &'a Vec<Tag>) -> Result<Self, Error> {
        let port_id_str = find_value("port_id", entries)?;
        let channel_id_str = find_value("channel_id", entries)?;
        let sequence_str = find_value("packet_sequence", entries)?;
        let recv_fee_str = find_value("recv_fee", entries)?;
        let ack_fee_str = find_value("ack_fee", entries)?;
        let timeout_fee_str = find_value("timeout_fee", entries)?;

        let port_id = PortId::from_str(&port_id_str).map_err(Error::ics24)?;

        let channel_id = ChannelId::from_str(&channel_id_str).map_err(Error::ics24)?;

        let sequence = Sequence::from_str(&sequence_str).map_err(Error::channel)?;

        let total_recv_fee = RawCoin::from_string_list(&recv_fee_str).map_err(Error::transfer)?;

        let total_ack_fee = RawCoin::from_string_list(&ack_fee_str).map_err(Error::transfer)?;

        let total_timeout_fee =
            RawCoin::from_string_list(&timeout_fee_str).map_err(Error::transfer)?;

        Ok(IncentivizedPacket {
            port_id,
            channel_id,
            sequence,
            total_recv_fee,
            total_ack_fee,
            total_timeout_fee,
        })
    }
}
