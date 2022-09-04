use std::{fmt::Error, collections::HashMap};

pub fn build(file : String) -> Result<String, Error> {
    let token_stream = parse(file).expect("Syntax error!");

    println!("{:#?}", token_stream);

    Ok(compile(token_stream).expect("Compilation failure."))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Let,
    Identifier(String),
    Assignment,
    SizedLiteral(usize),
    Free,
    BlockStart(usize),
    BlockEnd,
    PreprocessorDirective(String),
    Conditional,
    ExprStart(usize),
    ExprEnd,
    None,
    Equality
}

fn parse(file : String) -> Result<Vec<Token>, Error> {
    let mut iter = file.chars().into_iter();

    let mut vec : Vec<Token> = vec![];

    let mut current = String::new();

    while let Some(c) = iter.next() {
        match c {
            '=' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                let next = iter.next().expect("EOF after assignment.");
                if next == '=' {
                    vec.push(Token::Equality);
                } else {
                    if !next.is_whitespace() {
                        current.push(next);
                    }
                    vec.push(Token::Assignment);
                }
            },
            _ if c.is_numeric() => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }
                current.push(c);
                while let Some(next) = iter.next() {
                    if !next.is_numeric() {
                        vec.push(Token::SizedLiteral(current.parse::<usize>().expect("failed to parse sized literal")));
                        current.clear();
                        if !next.is_whitespace() && next != ';' { 
                            current.push(next);
                        }
                        break;
                    }
                    current.push(next);
                }
            },
            ';' => {
                vec.push(process_identifier(current.clone()));
                current.clear();
            } 
            '{' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                vec.push(Token::BlockStart(0));
            },
            '}' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                vec.push(Token::BlockEnd);
            },
            '(' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                vec.push(Token::ExprStart(0));
            },
            ')' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                vec.push(Token::ExprEnd);
            }
            '#' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                while let Some(c) = iter.next() {
                    if c == '\n' || c == '\r' { break }
                    current.push(c);
                }

                eprintln!("Preprocessor directives are not yet complete and will not be evaluated.");

                vec.push(Token::PreprocessorDirective(current.clone()));
                current.clear();
            },
            '\\' => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }

                while let Some(c) = iter.next() {
                    if c == '\n' || c == '\r' { break }
                }
            },
            _ if c.is_whitespace() => {
                if current.len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }
            },
            _ if c.is_alphabetic() => {
                current.push(c);
            },
            _ => panic!("Unrecognized character {}", c)
        }
    };

    preprocessor_firstpass(&mut vec);

    return Ok(vec);
}

fn process_identifier(token : String) -> Token {    
    match token.as_str().trim().trim_end() {
        "var" => Token::Let,
        "free" => Token::Free,
        "if" => Token::Conditional,
        "" => Token::None,
        "(" => Token::ExprStart(0),
        ")" => Token::ExprEnd,
        "{" => Token::BlockStart(0),
        "}" => Token::BlockEnd,
        _ => Token::Identifier(token)
    }
}

fn preprocessor_firstpass(tokens : &mut Vec<Token>) {
    let mut block_stack : Vec<usize> = Vec::new();
    let mut expr_stack : Vec<usize> = Vec::new();

    let len = tokens.len();

    for index in 0..len {
        match tokens[index] {
            Token::BlockStart(_) => {
                block_stack.push(index);
            },
            Token::BlockEnd => {
                let i = block_stack.pop().expect("Preprocessor error - unpaired block end bracket.");
                tokens[i] = Token::BlockStart(index);
            },
            Token::ExprStart(_) => {
                expr_stack.push(index);
            }
            Token::ExprEnd => {
                let i = expr_stack.pop().expect("Preprocessor error - unpaired expression end bracket.");
                tokens[i] = Token::ExprStart(index);
            },
            _ => { continue }
        }
    }

    if block_stack.len() > 0 {
        panic!("Incomplete block bracket pair!");
    }
}

fn compile(mut tokens : Vec<Token>) -> Result<String, Error> {
    let mut final_val : String = String::new();
    // let mut t = tokens.into_iter();

    let mut var_mem_addr_table : HashMap<String, String> = HashMap::new();
    let mut malloc : Vec<bool> = Vec::new();

    let mut index : usize = 0;
    let mut index_lables : Vec<usize> = vec![];
    
    while index < tokens.len() {
        match tokens[index] {
            Token::Let => {
                let var_index = allocate(&mut malloc);

                index += 1;
                let nt = tokens[index].clone();
                let name = match nt {
                    Token::Identifier(x) => x,
                    _ => panic!("expected variable name after var statement.")
                };

                if var_mem_addr_table.contains_key(&name) {
                    panic!("Redefinition of variable.");
                }

                index += 1;
                let eq = tokens[index].clone();
                match eq {
                    Token::Assignment => {},
                    _ => panic!("expected = after var statement")
                };

                index += 1;
                let cnst_val = tokens[index].clone();
                match cnst_val {
                    Token::SizedLiteral(x) => {
                        final_val.push_str(format!("memset {} {} // VARIABLE : {} // \n", var_index, x, name).as_str());
                    },
                    Token::Identifier(x) => {
                        final_val.push_str(
                            format!("mov rax {} // MEMORY VALUE PULL // \nmemset {} rax // ASSIGN MEMORY TO VARIABLE //\n",
                                var_mem_addr_table.get(&x).expect("Undefined variable!"),
                                var_index
                            ).as_str()
                        );
                    }
                    _ => panic!("Expected usize token after var statement.")
                };
                
                var_mem_addr_table.insert(name, var_index.to_string());
            },
            Token::Free => {
                index += 1;
                let varname = tokens[index].clone();

                match varname {
                    Token::Identifier(name) => {
                        free(&mut var_mem_addr_table, &mut malloc, name);
                    },
                    Token::SizedLiteral(_) => panic!("Compiler error - Attempted to free a constant."),
                    _ => panic!("Compiler error - Non-identifier token after free statement.")
                }
            },
            Token::Conditional => {
                final_val.push_str(compile_expression_recursive(&mut malloc, &mut tokens, &mut index).as_str());

                let val = match tokens[index] {
                    Token::BlockStart(u) => u,
                    _ => panic!("Compiler Error - No block after conditional expression.")
                };

                final_val.push_str(
                    format!(
                        "cgt rcx {} // IF // \n", val
                    ).as_str()
                );
            },
            Token::PreprocessorDirective(_) => { continue },
            Token::BlockStart(u) => { index_lables.push(u) },
            Token::BlockEnd => { 
                final_val.push_str(format!("label {} // IF END // \n", index).as_str());
            },
            _ => panic!("Unrecognized token {:#?}", tokens[index])
        }

        index += 1;
    }

    Ok(final_val)
}

fn compile_expression_recursive(malloc: &mut Vec<bool>, tokens : &mut Vec<Token>, index : &mut usize) -> String {
    *index += 1;
    let end_index = match tokens[*index] {
        Token::ExprStart(u) => u,
        _ => panic!("Internal error.")
    };

    *index += 1;

    let lhs_tmp_ptr = allocate(malloc);

    let lhs = match tokens[*index] {
        Token::SizedLiteral(s) => {
            format!(
                "memset {} {} // COMPILER INTERNAL // \nmov rax {} // COMPILER INTERNAL //\n", 
                lhs_tmp_ptr, 
                s,
                lhs_tmp_ptr
            )
        }
        _ => panic!("Invalid token in expression.")
    };

    *index += 1;
    
    let operator = match tokens[*index] {
        Token::Equality => {
            format!(
                "eq rcx rax rbx // COMPILER INTERNAL // \ninv rcx // EQUALITY // \n"
            )
        },
        _ => panic!("Invalid expression operator.")
    };
    
    *index += 1;

    let rhs_tmp_ptr = allocate(malloc);

    let rhs = match tokens[*index] {
        Token::SizedLiteral(s) => {
            format!(
                "memset {} {} // COMPILER INTERNAL // \nmov rbx {}\n",
                rhs_tmp_ptr,
                s,
                rhs_tmp_ptr
            )
        },
        _ => panic!("Invalid token in right hand side of expression.")
    };

    let asm = format!(
        "{}{}{}",
        lhs,
        rhs,
        operator
    );

    free_index(malloc, lhs_tmp_ptr);
    free_index(malloc, rhs_tmp_ptr);

    *index += 2;

    return asm;
}

fn free(var_mem_addr_table: &mut HashMap<String, String>, malloc: &mut Vec<bool>, name: String) {
    if !var_mem_addr_table.contains_key(&name) {
        panic!("Compiler error - freeing undefined variable {}", name);
    }
    malloc[var_mem_addr_table.get(&name).unwrap().parse::<usize>().unwrap()] = false;
    var_mem_addr_table.remove(&name);
}

fn free_index(malloc: &mut Vec<bool>, index : usize) {
    malloc[index] = false;
}

fn allocate(alloc : &mut Vec<bool> ) -> usize {
    for (index, value) in alloc.into_iter().enumerate() {
        if !*value {
            return index;
        }
    }

    alloc.push(true);
    alloc.len() - 1
}