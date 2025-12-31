use std::fs::File;
use unity_rs::streams::endian_reader::{BinaryReader, Endian, StreamReader};

fn main() -> std::io::Result<()> {
    // Create a test file
    std::fs::write(
        "test.bin",
        [
            3, 0, 0, 0, // i32: 3
            10, 0, 0, 0, // i32: 10
            20, 0, 0, 0, // i32: 20
            30, 0, 0, 0, // i32: 30
        ],
    )?;

    // Open with StreamReader
    let file = File::open("test.bin")?;
    let mut reader = StreamReader::new(file, Endian::Little, 0)?;

    // Read array
    let array = reader.read_i32_array(None)?;
    println!("Read i32 array from file: {:?}", array);
    println!("Position: {}", reader.position());

    Ok(())
}
