use scan::Scanner;
use virtual_machine::{chunk::Chunk, compiler::compile, values::Value, vm::Vm};

fn main() {
    let source = "var a = 1;";
    let mut vm = Vm::default();
    let scanner = Scanner::new(source);
    println!("scanner {:?}", &scanner);
    let mut chunk = Chunk::default();

    // Compile enrich the chunk with the data
    compile::<Value>(scanner, &mut chunk);

    match vm.interpret(chunk) {
        Ok(_) => println!("Program executed successfully"),
        Err(e) => eprintln!("Error during execution: {:?}", e),
    }
    println!("Virtual machine state: {:?}", vm);
}
