use std::fs;
use std::env;

enum OpCodes {

}

fn bytes_to_opcodes(bytes: Vec<u16>) -> Vec<OpCodes> {
  let mut opcodes = Vec::new();

  opcodes
}

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();

  let file_name = &args[1];

  // Todo
  let bytes = fs::read(file_name)
    .map_err(|err| format!("Read failed from {}", file_name))?;

  // TODO: handle odd number of bytes
  let u16_vec: Vec<u16> = bytes.chunks_exact(2)
    .map(|a| u16::from_be_bytes([a[0], a[1]]))
    .collect();

  println!("{:04x?}", u16_vec);

  Ok(())
}
