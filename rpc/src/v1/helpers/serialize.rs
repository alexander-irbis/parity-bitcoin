use std::fmt::LowerHex;

use hex;
use serde::{self, Serializer, Deserializer, Deserialize};


pub fn as_hex<V, S>(v: V, serializer: S) -> Result<S::Ok, S::Error>
	where
		V: LowerHex,
		S: Serializer,
{
	// FIXME avoid string allocation
	serializer.serialize_str(&format!("{:x}", v))
}

fn u32_from_hex<'de, D>(deserializer: D) -> Result<T, D::Error>
	where
		D: Deserializer<'de>,
		T: Deserialize<'static>,
{

	String::deserialize(deserializer)
		.and_then(|string| {
			let x: [u8; 8] = hex::FromHex::from_hex(&string)
				.map_err(|err| serde::Error::custom(err.to_string()))
				.and_then();
			serde::de::from_slice(&x)
		})
		.map(|bytes| PublicKey::from_slice(&bytes))
		.and_then(|opt| opt.ok_or_else(|| serde::Error::custom("failed to deserialize public key")))
}
