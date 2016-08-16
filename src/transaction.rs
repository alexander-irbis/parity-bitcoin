
//! Bitcoin trainsaction.
//! https://en.bitcoin.it/wiki/Protocol_documentation#tx

use reader::{Deserializable, Reader, Error as ReaderError};
use crypto::dhash;
use hash::H256;
use stream::{Serializable, Stream, serialize};
use compact_integer::CompactInteger;

#[derive(Debug)]
pub struct OutPoint {
	hash: H256,
	index: u32,
}

impl Serializable for OutPoint {
	fn serialize(&self, stream: &mut Stream) {
		stream
			.append_bytes(&self.hash)
			.append(&self.index);
	}
}

impl Deserializable for OutPoint {
	fn deserialize(reader: &mut Reader) -> Result<Self, ReaderError> where Self: Sized {
		let mut hash = [0u8; 32];
		hash.copy_from_slice(try!(reader.read_bytes(32)));
		let index = try!(reader.read());
		let result = OutPoint {
			hash: hash,
			index: index,
		};

		Ok(result)
	}
}

#[derive(Debug)]
pub struct TransactionInput {
	previous_output: OutPoint,
	signature_script: Vec<u8>,
	sequence: u32,
}

impl Serializable for TransactionInput {
	fn serialize(&self, stream: &mut Stream) {
		stream
			.append(&self.previous_output)
			.append(&CompactInteger::from(self.signature_script.len()))
			.append_bytes(&self.signature_script)
			.append(&self.sequence);
	}
}

impl Deserializable for TransactionInput {
	fn deserialize(reader: &mut Reader) -> Result<Self, ReaderError> where Self: Sized {
		let previous_output = try!(reader.read());
		let signature_script_len = try!(reader.read::<CompactInteger>());
		let signature_script = try!(reader.read_bytes(signature_script_len.into())).to_vec();
		let sequence = try!(reader.read());

		let result = TransactionInput {
			previous_output: previous_output,
			signature_script: signature_script,
			sequence: sequence,
		};

		Ok(result)
	}
}

#[derive(Debug)]
pub struct TransactionOutput {
	value: u64,
	pk_script: Vec<u8>,
}

impl Serializable for TransactionOutput {
	fn serialize(&self, stream: &mut Stream) {
		stream
			.append(&self.value)
			.append(&CompactInteger::from(self.pk_script.len()))
			.append_bytes(&self.pk_script);
	}
}

impl Deserializable for TransactionOutput {
	fn deserialize(reader: &mut Reader) -> Result<Self, ReaderError> where Self: Sized {
		let value = try!(reader.read());
		let pk_script_len = try!(reader.read::<CompactInteger>());
		let pk_script = try!(reader.read_bytes(pk_script_len.into())).to_vec();

		let result = TransactionOutput {
			value: value,
			pk_script: pk_script,
		};

		Ok(result)
	}
}

#[derive(Debug)]
pub struct Transaction {
	version: i32,
	transaction_inputs: Vec<TransactionInput>,
	transaction_outputs: Vec<TransactionOutput>,
	lock_time: u32,
}

impl Serializable for Transaction {
	fn serialize(&self, stream: &mut Stream) {
		stream
			.append(&self.version)
			.append(&CompactInteger::from(self.transaction_inputs.len()))
			.append_list(&self.transaction_inputs)
			.append(&CompactInteger::from(self.transaction_outputs.len()))
			.append_list(&self.transaction_outputs)
			.append(&self.lock_time);
	}
}

impl Deserializable for Transaction {
	fn deserialize(reader: &mut Reader) -> Result<Self, ReaderError> where Self: Sized {
		let version = try!(reader.read());
		let tx_inputs_len= try!(reader.read::<CompactInteger>());
		let tx_inputs = try!(reader.read_list(tx_inputs_len.into()));
		let tx_outputs_len= try!(reader.read::<CompactInteger>());
		let tx_outputs = try!(reader.read_list(tx_outputs_len.into()));
		let lock_time = try!(reader.read());

		let result = Transaction {
			version: version,
			transaction_inputs: tx_inputs,
			transaction_outputs: tx_outputs,
			lock_time: lock_time,
		};

		Ok(result)
	}
}

impl Transaction {
	pub fn hash(&self) -> H256 {
		dhash(&serialize(self))
	}
}

#[cfg(test)]
mod tests {
	use rustc_serialize::hex::FromHex;
	use reader::deserialize;
	use super::Transaction;

	// real transaction from block 80000
	// https://blockchain.info/rawtx/5a4ebf66822b0b2d56bd9dc64ece0bc38ee7844a23ff1d7320a88c5fdb2ad3e2
	// https://blockchain.info/rawtx/5a4ebf66822b0b2d56bd9dc64ece0bc38ee7844a23ff1d7320a88c5fdb2ad3e2?format=hex
	#[test]
	fn test_transaction_reader() {
		let encoded_tx = "0100000001a6b97044d03da79c005b20ea9c0e1a6d9dc12d9f7b91a5911c9030a439eed8f5000000004948304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501ffffffff0100f2052a010000001976a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac00000000".from_hex().unwrap();
		let t: Transaction = deserialize(&encoded_tx).unwrap();
		assert_eq!(t.version, 1);
		assert_eq!(t.lock_time, 0);
		assert_eq!(t.transaction_inputs.len(), 1);
		assert_eq!(t.transaction_outputs.len(), 1);
		let tx_input = &t.transaction_inputs[0];
		assert_eq!(tx_input.sequence, 4294967295);
		assert_eq!(tx_input.signature_script, "48304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501".from_hex().unwrap());
		let tx_output = &t.transaction_outputs[0];
		assert_eq!(tx_output.value, 5000000000);
		assert_eq!(tx_output.pk_script, "76a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac".from_hex().unwrap());
	}
	
	#[test]
	fn test_transaction_hash() {
		let encoded_tx = "0100000001a6b97044d03da79c005b20ea9c0e1a6d9dc12d9f7b91a5911c9030a439eed8f5000000004948304502206e21798a42fae0e854281abd38bacd1aeed3ee3738d9e1446618c4571d1090db022100e2ac980643b0b82c0e88ffdfec6b64e3e6ba35e7ba5fdd7d5d6cc8d25c6b241501ffffffff0100f2052a010000001976a914404371705fa9bd789a2fcd52d2c580b65d35549d88ac00000000".from_hex().unwrap();
		let mut hash = "5a4ebf66822b0b2d56bd9dc64ece0bc38ee7844a23ff1d7320a88c5fdb2ad3e2".from_hex().unwrap();
		hash.reverse();
		let t: Transaction = deserialize(&encoded_tx).unwrap();
		assert_eq!(t.hash().to_vec(), hash);
	}
}
