use std::{fmt::Error, collections::HashMap};

pub fn build(file : String) -> Result<String, Error> {
    let token_stream = parse(file).expect("Syntax error!");

    Ok(compile(token_stream).expect("Compilation failure."))
}

#[derive(Debug, Clone)]
enum Token {
    Let,
    Identifier(String),
    Assignment,
    SizedLiteral(usize),
    Free
}

fn parse(file : String) -> Result<Vec<Token>, Error> {
    let mut iter = file.chars().into_iter();

    let mut vec : Vec<Token> = vec![];

    let mut current = String::new();

    while let Some(c) = iter.next() {
        match c {
            '=' => {
                if (&current).len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }
                vec.push(Token::Assignment);
            },
            _ if c.is_numeric() => {
                if (&current).len() > 0 {
                    vec.push(process_identifier(current.clone()));
                    current.clear();
                }
                current.push(c);
                while let Some(next) = iter.next() {
                    if !next.is_numeric() {
                        vec.push(Token::SizedLiteral(current.parse::<usize>().expect("failed to parse sized literal")));
                        current.clear();
                        break;
                    }
                    current.push(next);
                }
            },
            ';' => {
                vec.push(process_identifier(current.clone()));
                current.clear();
            } 
            _ if c.is_whitespace() => {
                if current.len() == 0 {
                    continue;
                }
                vec.push(process_identifier(current.clone()));
                current.clear();
            },
            _ => current.push(c)
        }
    };

    return Ok(vec);
}

fn process_identifier(token : String) -> Token {
    match token.as_str() {
        "var" => Token::Let,
        "free" => Token::Free,
        _ => Token::Identifier(token)
    }
}

fn compile(tokens : Vec<Token>) -> Result<String, Error> {
    let mut final_val : String = String::new();
    let mut t = tokens.into_iter();

    let mut var_mem_addr_table : HashMap<String, String> = HashMap::new();
    let mut malloc : Vec<bool> = Vec::new();
    
    while let Some(token) = t.next() {
        match token {
            Token::Let => {
                let index = allocate(&mut malloc);

                let nt = t.next().expect("EOF after var statement");
                let name = match nt {
                    Token::Identifier(x) => x,
                    _ => panic!("expected variable name after var statement.")
                };

                if var_mem_addr_table.contains_key(&name) {
                    panic!("Redefinition of variable.");
                }

                let eq = t.next().expect("EOF after variable name");
                match eq {
                    Token::Assignment => {},
                    _ => panic!("expected = after var statement")
                };

                let cnst_val = t.next().expect("EOF after var statement");
                match cnst_val {
                    Token::SizedLiteral(x) => {
                        final_val.push_str(format!("memset {} {} // VARIABLE : {} // \n", index, x, name).as_str());
                    },
                    Token::Identifier(x) => {
                        final_val.push_str(
                            format!("mov rax {} // MEMORY VALUE PULL // \nmemset {} rax // ASSIGN MEMORY TO VARIABLE //\n",
                                var_mem_addr_table.get(&x).expect("Undefined variable!"),
                                index
                            ).as_str()
                        );
                    }
                    _ => panic!("Expected usize token after var statement.")
                };
                
                var_mem_addr_table.insert(name, index.to_string());
            },
            Token::Free => {
                let varname = t.next().expect("Compiler error - EOF after free statement");

                match varname {
                    Token::Identifier(name) => {
                        if !var_mem_addr_table.contains_key(&name) {
                            panic!("Compiler error - freeing undefined variable {}", name);
                        }

                        malloc[var_mem_addr_table.get(&name).unwrap().parse::<usize>().unwrap()] = false;
                        var_mem_addr_table.remove(&name);
                    },
                    _ => panic!("Compiler error - Non-identifier token after free statement.")
                }
            }
            _ => panic!("Unrecognized token.")
        }
    }

    Ok(final_val)
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