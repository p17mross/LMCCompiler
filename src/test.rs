use std::{collections::{HashMap, HashSet}};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenType<'a> {
    Identifier(&'a str),
    Number(i32),
    NewLine,
    If,
    EndIf,
    Else,
    While,
    EndWhile,
    Break,
    Input,
    Output,
    True,
    OperatorAdd,
    OperatorSub,
    OperatorAssignment,
    OperatorInequality,
    OperatorEquality,
    OperatorGreaterThan,
    OperatorLessThan,
    OperatorGreaterThanInclusive,
    OperatorLessThanInclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
    line: usize,
    token_type: TokenType<'a>
}

use TokenType::*;

fn tokenise(src: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let split_by_comment: Vec<&str> = line.splitn(2, "//").collect();

        for token_str in split_by_comment[0].split_whitespace() {
            if let Ok(n) = str::parse::<i32>(token_str){
                if n > 999 || n < -999 {
                    println!("Warning: number {} on line {} is outside the bounds of LMC numbers", n, i);
                }
                tokens.push(Token { line: i, token_type: Number(n) })
            }
            else {
                let token = match token_str {
                    "if" => If,
                    "endif" => EndIf,
                    "else" => Else,
                    "while" => While,
                    "endwhile" => EndWhile,
                    "break" => Break,
                    "input" => Input,
                    "output" | "print" => Output,
                    "true" => True,
                    "+" => OperatorAdd,
                    "-" => OperatorSub,
                    "=" => OperatorAssignment,
                    "==" => OperatorEquality,
                    "!=" => OperatorInequality,
                    ">" => OperatorGreaterThan,
                    "<" => OperatorLessThan,
                    ">=" => OperatorGreaterThanInclusive,
                    "<=" => OperatorLessThanInclusive,
                    s => Identifier(s)
                };
                tokens.push(Token { line: i, token_type: token })
            }

        }

        tokens.push(Token { line: i, token_type: NewLine });
    }

    tokens
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    While{start_line: usize},
    If{else_start_line: usize, if_start_line: usize},
}

fn parse_tokens(src: Vec<Token>) -> Result<String, String> {
    let mut vars: HashMap<&str, i32> = HashMap::new();
    let mut consts: HashSet<i32> = HashSet::new();
    let mut program: String = String::new();

    let mut scope_stack: Vec<Scope> = Vec::new();
    
    let lines: Vec<&[Token]> = src.split(|t| t.token_type == NewLine).collect();

    for line in lines {
        if line.len() == 0 {
            continue;
        }
        let line_no = line[0].line;
        match line[0].token_type {
            //Variable assignment
            Identifier(assigned_to) => {
                if line.len() == 1 || line[1].token_type != OperatorAssignment {
                    return Err(format!("Error on line {line_no}: Identifer at the beginning of a line must be followed by '='"));
                }

                match line.get(2) {
                    None => return Err(format!("Error on line {line_no}: Expected identifier or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            program += &format!("LDA var_{s}\n");
                        },
                        Number(n) => {
                            consts.insert(n);
                            program += &format!("LDA const_{n}\n");
                        }
                        _ => return Err(format!("Error on line {line_no} token 2: Expected identifier or number"))
                    }
                }

                match line.get(3) {
                    None => {
                        program += &format!("STA var_{assigned_to}\n");
                        continue
                    },
                    Some(t) => match t.token_type {
                        OperatorAdd => program += "ADD ",
                        OperatorSub => program += "SUB ",
                        _ => return Err(format!("Error on line {line_no} token 3: Expected '+' or '1'"))
                    }
                }

                match line.get(4) {
                    None => return Err(format!("Error on line {line_no}: Expected identifer or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => program += &format!("var_{s}\n"),
                        Number(n) => {
                            consts.insert(n);
                            program += &format!("const_{n}\n")
                        },
                        _ => return Err(format!("Error on line {line_no} token 4: Expected identifer or number"))
                    }
                }

                program += &format!("STA var_{assigned_to}\n");
                
            }
            //Input
            Input => {
                match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected identifier")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            if !vars.contains_key(s) {
                                vars.insert(s, 0);
                            }
                            program += &format!("INP\nSTA var_{s}\n")
                        },
                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier"))
                    }
                }
            }
            //Output
            Output => {
                match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected identifier or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => program += &format!("LDA var_{s}\nOUT\n"),
                        Number(n) => {
                            consts.insert(n);
                            program += &format!("LDA const_{n}\nOUT\n");
                        }
                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier or number"))
                    }
                }
            }
            //While
            While => {
                program += &format!("while_{line_no} ");
                scope_stack.push(Scope::While { start_line: line_no });


                let label_if_true = format!("while_{line_no}_body");
                let label_if_false = format!("while_{line_no}_end");



                let lhs = match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                    Some(t) => match t.token_type {
                        Identifier(s) => format!("var_{s}"),
                        Number(n) => {
                            consts.insert(n);
                            format!("const_{n}")
                        },
                        True => continue,
                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier or number"))
                    }
                };

                let rhs = match line.get(3) {
                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                    Some(t) => match t.token_type {
                        Identifier(s) => format!("var_{s}"),
                        Number(n) => {
                            consts.insert(n);
                            format!("const_{n}")
                        },
                        _ => return Err(format!("Error on line {line_no} token 3: Expected identifier or number"))
                    }
                };

                match line[2].token_type {
                    OperatorEquality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_true}\nBRA {label_if_false}\n"),
                    OperatorInequality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_false}\nBRA {label_if_true}\n"),

                    OperatorGreaterThan => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),
                    OperatorLessThan => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),

                    OperatorGreaterThanInclusive => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                    OperatorLessThanInclusive => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                
                    _ => return Err(format!("Error on line {line_no} token 2: Expected comparison operator"))
                }

                program += &format!("{label_if_true} ");
            }
            //End while
            EndWhile => {
                match scope_stack.pop() {
                    None => return Err(format!("Error on line {line_no}: 'endwhile' found while 'while' loop was not inner most control flow construct")),
                    Some(Scope::While { start_line })=>  program += &format!("BRA while_{start_line}\nwhile_{start_line}_end "),
                    _ => return Err(format!("Error on line {line_no}: 'endwhile' found while 'while' loop was not inner most control flow construct"))
                }
            }
            //If
            If => {
                scope_stack.push(Scope::If { if_start_line: line_no, else_start_line: line_no });

                let lhs = match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                    Some(t) => match t.token_type {
                        Identifier(s) => format!("var_{s}"),
                        Number(n) => {
                            consts.insert(n);
                            format!("const_{n}")
                        },
                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier or number"))
                    }
                };

                let rhs = match line.get(3) {
                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                    Some(t) => match t.token_type {
                        Identifier(s) => format!("var_{s}"),
                        Number(n) => {
                            consts.insert(n);
                            format!("const_{n}")
                        },
                        _ => return Err(format!("Error on line {line_no} token 3: Expected identifier or number"))
                    }
                };

                let label_if_true = format!("if_{line_no}_body");
                let label_if_false = format!("if_{line_no}_else");

                match line[2].token_type {
                    OperatorEquality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_true}\nBRA {label_if_false}\n"),
                    OperatorInequality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_false}\nBRA {label_if_true}\n"),

                    OperatorGreaterThan => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),
                    OperatorLessThan => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),

                    OperatorGreaterThanInclusive => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                    OperatorLessThanInclusive => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                
                    _ => return Err(format!("Error on line {line_no} token 2: Expected comparison operator"))
                }

                program += &format!("{label_if_true} ");
            }
            //Else
            Else => {
                match scope_stack.pop() {
                    None => return Err(format!("Error on line {line_no}: 'else' found while 'if' statement was not inner most control flow construct")),
                    Some(Scope::If { if_start_line, else_start_line }) => match line.get(1) {
                        None => {
                            scope_stack.push(Scope::If { if_start_line: if_start_line, else_start_line: line_no });
                            program += &format!("BRA if_{if_start_line}_end\nif_{else_start_line}_else ");
                        }
                        Some(t) => match t.token_type {
                            If => {
                                scope_stack.push(Scope::If { if_start_line: if_start_line, else_start_line: line_no });

                                let lhs = match line.get(2) {
                                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                                    Some(t) => match t.token_type {
                                        Identifier(s) => format!("var_{s}"),
                                        Number(n) => {
                                            consts.insert(n);
                                            format!("const_{n}")
                                        },
                                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier or number"))
                                    }
                                };

                                let rhs = match line.get(4) {
                                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                                    Some(t) => match t.token_type {
                                        Identifier(s) => format!("var_{s}"),
                                        Number(n) => {
                                            consts.insert(n);
                                            format!("const_{n}")
                                        },
                                        _ => return Err(format!("Error on line {line_no} token 3: Expected identifier or number"))
                                    }
                                };

                                let label_if_true = format!("if_{line_no}_body");
                                let label_if_false = format!("if_{line_no}_else");

                                program += &format!("BRA if_{if_start_line}_end\nif_{else_start_line}_else ");

                                match line[3].token_type {
                                    OperatorEquality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_true}\nBRA {label_if_false}\n"),
                                    OperatorInequality => program += &format!("LDA {lhs}\nSUB {rhs}\nBRZ {label_if_false}\nBRA {label_if_true}\n"),

                                    OperatorGreaterThan => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),
                                    OperatorLessThan => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_false}\nBRA {label_if_true}\n"),

                                    OperatorGreaterThanInclusive => program += &format!("LDA {lhs}\nSUB {rhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                                    OperatorLessThanInclusive => program += &format!("LDA {rhs}\nSUB {lhs}\nBRP {label_if_true}\nBRA {label_if_false}\n"),
                                
                                    _ => return Err(format!("Error on line {line_no} token 2: Expected comparison operator"))
                                }

                                program += &format!("{label_if_true} ");
                            },
                            _ => return Err(format!("Error on line {line_no}: 'else' found while 'if' statement was not inner most control flow construct"))
                        }
                    }
                    _ => return Err(format!("Error on line {line_no}: expected 'else if' or just 'else'"))
                }
            }
            //End if
            EndIf => {
                match scope_stack.pop() {
                    None => return Err(format!("Error on line {line_no}: 'endif' found while 'if' statement was not inner most control flow construct")),
                    Some(Scope::If { if_start_line, else_start_line: _ }) =>  program += &format!("if_{if_start_line}_end "),
                    _ => return Err(format!("Error on line {line_no}: 'endif' found while 'if' statement was not inner most control flow construct"))
                }
            }
            
            _ => return Err(format!("Error on line {line_no}: Expected assignment, input, output, or start or end of if statement or while loop"))
        }
    }

    program += "HLT\n";
    for (s, n) in vars {
        program += &format!("var_{s} DAT {n}\n");
    }

    program += "\n";
    for n in consts {
        program += &format!("const_{n} DAT {n}\n");
    }

    Ok(program)
}

pub fn compile(src: &str) -> Result<String, String> {

    let tokens = tokenise(src);

    parse_tokens(tokens)

}