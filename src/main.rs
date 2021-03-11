use std::fs;
use std::env;

enum OpCodes {

}

fn bytes_to_opcodes(bytes: Vec<u8>) {

}

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();

  let file_name = &args[1];

  let bytes = fs::read(file_name)
    .map_err(|err| format!("Read failed from {}", file_name))?;

  println!("{:02x?}", bytes);

  Ok(())
}
