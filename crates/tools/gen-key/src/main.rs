use anyhow::Result;
use lib_utils::b64::b64u_encode;
use rand::RngCore;

fn main() -> Result<()> {
	let mut key = [0u8; 64]; // 512 bits = 64 bytes
	rand::thread_rng().fill_bytes(&mut key);
	println!("\nGenerated key from rand::thread_rng():\n{key:?}");

	let b64u = b64u_encode(key);
	println!("\nKey b64u encoded:\n{b64u}");

	Ok(())
}
