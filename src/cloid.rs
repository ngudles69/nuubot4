//! CLOID encoding.

use crate::{NuuError, Result};

/// Carry the canonical decoded 128-bit Order identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloidFields {
    pub botcycle_id: u32,
    pub symbol_id: u16,
    pub exchange: u8,
    pub network: u8,
    pub side: u8,
    pub reduce_only: bool,
    pub purpose: u8,
    pub trade_no: u32,
    pub batch_no: u16,
    pub order_pos: u16,
    pub timestamp_s: u32,
}

/// Validate and pack one Order identity into Hyperliquid CLOID format.
pub fn encode_cloid(fields: CloidFields) -> Result<String> {
    // Validate fixed ranges.
    if fields.botcycle_id > 0x00ff_ffff
        || fields.exchange > 0x0f
        || fields.network > 0x03
        || fields.side > 1
        || fields.trade_no == 0
        || fields.trade_no > 0x001f_ffff
        || fields.batch_no == 0
        || fields.batch_no > 1000
        || fields.order_pos == 0
        || fields.order_pos > 1000
        || fields.timestamp_s > 0x7fff_ffff
    {
        return Err(NuuError::Cloid(
            "one field is outside its fixed range".into(),
        ));
    }

    // Pack fixed fields.
    let raw = ((fields.botcycle_id as u128) << 104)
        | ((fields.symbol_id as u128) << 88)
        | ((fields.exchange as u128) << 84)
        | ((fields.network as u128) << 82)
        | ((fields.side as u128) << 81)
        | ((fields.reduce_only as u128) << 80)
        | ((fields.purpose as u128) << 72)
        | ((fields.trade_no as u128) << 51)
        | ((fields.batch_no as u128) << 41)
        | ((fields.order_pos as u128) << 31)
        | fields.timestamp_s as u128;
    Ok(format!("0x{raw:032x}"))
}

/// Validate and unpack one Hyperliquid CLOID.
pub fn decode_cloid(cloid: &str) -> Result<CloidFields> {
    // Validate exchange shape.
    if cloid.len() != 34
        || !cloid.starts_with("0x")
        || !cloid[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(NuuError::Cloid(
            "cloid must be 0x followed by exactly 32 hex characters".into(),
        ));
    }
    let raw = u128::from_str_radix(&cloid[2..], 16)
        .map_err(|error| NuuError::Cloid(error.to_string()))?;

    // Unpack fixed fields.
    let fields = CloidFields {
        botcycle_id: ((raw >> 104) & 0x00ff_ffff) as u32,
        symbol_id: ((raw >> 88) & 0xffff) as u16,
        exchange: ((raw >> 84) & 0x0f) as u8,
        network: ((raw >> 82) & 0x03) as u8,
        side: ((raw >> 81) & 0x01) as u8,
        reduce_only: ((raw >> 80) & 0x01) == 1,
        purpose: ((raw >> 72) & 0xff) as u8,
        trade_no: ((raw >> 51) & 0x001f_ffff) as u32,
        batch_no: ((raw >> 41) & 0x03ff) as u16,
        order_pos: ((raw >> 31) & 0x03ff) as u16,
        timestamp_s: (raw & 0x7fff_ffff) as u32,
    };
    if fields.trade_no == 0
        || fields.batch_no == 0
        || fields.batch_no > 1000
        || fields.order_pos == 0
        || fields.order_pos > 1000
    {
        return Err(NuuError::Cloid(
            "decoded Order identity is outside its fixed range".into(),
        ));
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloid_round_trip() {
        let fields = CloidFields {
            botcycle_id: 16_777_215,
            symbol_id: 65_535,
            exchange: 15,
            network: 3,
            side: 1,
            reduce_only: true,
            purpose: 255,
            trade_no: 2_097_151,
            batch_no: 1000,
            order_pos: 1000,
            timestamp_s: 2_147_483_647,
        };
        let encoded = encode_cloid(fields).unwrap();
        assert_eq!(encoded.len(), 34);
        assert_eq!(decode_cloid(&encoded).unwrap(), fields);
    }
}
