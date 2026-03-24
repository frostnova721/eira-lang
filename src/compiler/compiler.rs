use std::path::PathBuf;

use crate::{
    CodeGen, Parser, Value, WeaveAnalyzer,
    assembler::Assembler,
    compiler::{
        Stmt, WovenStmt,
        scanner::{Scanner, Token},
        scroll_reader::ScrollReader,
    },
    print_ast, print_byte_code, print_woven_ast,
    runtime::Instruction,
};

type Result<T> = std::result::Result<T, CompileError>;

pub struct CompileError {
    pub msg: String,
}

pub struct CompilerOptions {
    pub print_tokens: bool,
    pub print_ast: Option<u8>,
    pub print_woven_ast: Option<u8>,
    pub print_instructions: bool,
    pub print_bytecode: bool,
}

pub struct Compiler {
    pub source_path: String,
    pub options: CompilerOptions,
    // pub
}

pub struct CompiledCode {
    pub bytecode: Vec<u8>,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}

impl Compiler {
    pub fn new(source_path: String, options: CompilerOptions) -> Self {
        Compiler {
            source_path,
            options,
        }
    }

    pub fn compile(&self) -> Result<CompiledCode> {
        let tokens = self.scan()?;

        if self.options.print_tokens {
            println!("Tokens:");
            for token in &tokens {
                println!("{}", token);
            }
        }

        let ast = self.parse(tokens)?;

        if self.options.print_ast.is_some() {
            println!("AST:");
            print_ast(&ast, self.options.print_ast.unwrap());
        }

        let woven_ast = self.analyze_weaves(ast)?;

        if self.options.print_woven_ast.is_some() {
            println!("Woven AST:");
            print_woven_ast(&woven_ast, self.options.print_woven_ast.unwrap());
        }

        let instructions = self.gen_instructions(woven_ast)?;

        Ok(instructions)
    }

    pub fn compile_to_bytecode(&self) -> Result<CompiledCode> {
        let mut compiled_code = self.compile()?;
        compiled_code.bytecode = self.gen_bytecode(&compiled_code.instructions);

        if self.options.print_bytecode {
            println!("Bytecode:");
            print_byte_code(&compiled_code.bytecode);
        }

        Ok(compiled_code)
    }

    fn scan(&self) -> Result<Vec<Token>> {
        let scroll_reader = ScrollReader::new(PathBuf::from(&self.source_path), Vec::new());

        let content = scroll_reader.read_scroll(&PathBuf::from(&self.source_path));

        if content.is_err() {
            return Err(CompileError {
                msg: content.err().unwrap().msg,
            });
        }

        Ok(Scanner::init(&content.ok().unwrap()).tokenize())
    }

    fn parse(&self, tokens: Vec<Token>) -> Result<Vec<Stmt>> {
        let ast = Parser::new(tokens, self.source_path.clone()).parse();
        match ast {
            Err(parse_error) => {
                // println!("Parse Error: {:?}", parse_error.0);
                return Err(CompileError {
                    msg: format!("Parse Error: {:?}", parse_error.0),
                });
            }
            Ok(ast) => Ok(ast),
        }
    }

    fn analyze_weaves(&self, ast: Vec<Stmt>) -> Result<Vec<WovenStmt>> {
        let mut weave_analyzer = WeaveAnalyzer::new();
        match weave_analyzer.analyze(ast) {
            Err(no_no) => {
                let errstr = format!(
                    "Weave Error: {}\nat '{}' in {}:{}:{}",
                    no_no.msg,
                    no_no.token.lexeme,
                    self.source_path,
                    no_no.token.line,
                    no_no.token.column,
                );
                return Err(CompileError { msg: errstr });
            }
            Ok(woven_ast) => Ok(woven_ast),
        }
    }

    fn gen_bytecode(&self, instructions: &Vec<Instruction>) -> Vec<u8> {
        Assembler::convert_to_byte_code(instructions)
    }

    fn gen_instructions(&self, woven_ast: Vec<WovenStmt>) -> Result<CompiledCode> {
        let mut cg = CodeGen::new(woven_ast, self.options.print_instructions);
        match cg.summon_instructions() {
            Err(gen_error) => {
                // println!("CodeGen Error: {}", gen_error.msg);
                return Err(CompileError {
                    msg: format!("CodeGen Error: {}", gen_error.msg),
                });
            }
            Ok(instructions) => Ok(CompiledCode {
                bytecode: vec![],
                instructions,
                constants: cg.get_constants(),
            }),
        }
    }
}
