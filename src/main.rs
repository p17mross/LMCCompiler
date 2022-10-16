use std::env;
use std::fs;

mod test;

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    let program = fs::read_to_string(args[1].clone())
    .expect("Should have been able to read the file");

    match test::compile(&program) {
        Ok(s) => {
            print!("{s}"); 
            return Ok(())
        },
        Err(s) => {
            println!("{s}");
            return Err(())
        }
    }
}
