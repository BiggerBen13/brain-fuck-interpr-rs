use std::io::Read;

use brainfuck_rs;
fn main() {
    let path: Vec<String> = std::env::args().collect();
    let mut file = std::fs::File::open(&path[1]).unwrap();
    let mut source: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut source);
    let mut executor = brainfuck_rs::brainfuck::Executor::from_bytes(&source);
    let _ = executor.run();
}
