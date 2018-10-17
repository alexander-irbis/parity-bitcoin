use std::fmt::LowerHex;

use serde::{self, Serializer, Deserializer, Deserialize};


pub fn as_hex<V, S>(v: V, serializer: S) -> Result<S::Ok, S::Error>
	where
		V: LowerHex,
		S: Serializer,
{
	// FIXME avoid string allocation
	serializer.serialize_str(&format!("{:x}", v))
}

pub fn u32_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
	where
		D: Deserializer<'de>,
{
	String::deserialize(deserializer)
		.and_then(|string| {
			u32::from_str_radix(&string, 16)
				.map_err(|err| serde::de::Error::custom(err.to_string()))
		})
}
