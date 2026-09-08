mod core;
mod emitter;
mod ir;
mod lexer;
mod parser;

use std::fs;
use emitter::Emitter;
use ir::IRGenerator;
use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("---------------------------------------------------------");
    println!("  Axiom Compiler v0.2.0 (Freestanding Windows x86-64)");
    println!("---------------------------------------------------------");

    let source_path = "source.txt";
    let output_asm = "output.asm";

    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("Failed to read source file: {}", source_path));

    println!("[1/4] Lexing tokens...");
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("[2/4] Parsing into AST...");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    println!("[3/4] Generating 3-address IR...");
    let mut ir_gen = IRGenerator::new();
    let ir = ir_gen.generate_program(ast);

    println!("[4/4] Emitting freestanding NASM assembly to '{}'...", output_asm);
    let emitter = Emitter::new();
    emitter.emit_to_file(&ir, output_asm).expect("Failed to write assembly output");

    println!("---------------------------------------------------------");
    println!("Success! Emitted pure Win64 assembly to '{}'", output_asm);
    println!();
    println!("To assemble and link without CRT:");
    println!("  nasm -f win64 output.asm -o output.obj");
    println!("  x86_64-w64-mingw32-gcc -nostdlib -e mainCRTStartup output.obj -lkernel32 -o program.exe");
    println!("  ./program.exe");
    println!("---------------------------------------------------------");
}