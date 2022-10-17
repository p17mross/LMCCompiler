use std::{collections::{HashMap, HashSet}};

/// Types of token output by the tokeniser
#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenType<'a> {
    /// Any token not matched by another token
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

/// A token output by the tokeniser
#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
    line: usize,
    token_type: TokenType<'a>
}

use TokenType::*;

/// Takes a string and returns Vec<Token>.
/// Does not error - any syntax errors will be caught in the parser.
/// Any string that does not match another token will become an identifier, which means that any string can become an identifier.
fn tokenise(src: &str) -> Vec<Token> {
    // Final list of tokens
    let mut tokens: Vec<Token> = Vec::new();
    // Loop over lines of string
    for (i, line) in src.lines().enumerate() {
        // Ignore anything after a comment
        let split_by_comment: Vec<&str> = line.splitn(2, "//").collect();

        // Separate tokens by whitespace
        for token_str in split_by_comment[0].split_whitespace() {
            // If the token is a number, add a Number token
            if let Ok(n) = str::parse::<i32>(token_str) {
                // Check bounds of LMC ints
                if n > 999 || n < -999 {
                    println!("Warning: number {} on line {} is outside the bounds of LMC numbers", n, i);
                }
                tokens.push(Token { line: i, token_type: Number(n) })
            }
            else {
                // Match specific keywords
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
                    // Anything else is an identifier
                    s => Identifier(s)
                };
                tokens.push(Token { line: i, token_type: token })
            }
        }
        // Add newline after every line
        tokens.push(Token { line: i, token_type: NewLine });
    }
    // Return tokens
    tokens
}

/// A scope for an if statement or while loop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// While loop
    While {
        /// Used so that the 'endwhile' can emit the correct label
        start_line: usize
    },
    If {
        /// The line of the 'if' statement
        if_start_line: usize,
        /// The line of the 'if' or 'else if' statement
        else_start_line: usize,
        /// Whether there is an 'else' to the if.
        /// Controls whether the label emitted by the 'endif' is if_{line}_else or if_{line}_end
        /// 'else if's don't count for this as they emit the correct symbol anyway
        has_else: bool
    },
}

/// Parses a Vec<Token> into LMC assembly
fn parse_tokens(src: Vec<Token>) -> Result<String, String> {
    // Definded variables
    let mut vars: HashMap<&str, i32> = HashMap::new();
    // Constants used in expressions, as the LMC instruction set has no immediates
    let mut consts: HashSet<i32> = HashSet::new();
    // 0 is always a constant as a fix for having multiple labels on one line
    consts.insert(0);

    // The program
    let mut program: String = String::new();

    // A stack of Scopes to store line numbers of constructs that need end labels
    let mut scope_stack: Vec<Scope> = Vec::new();
    
    // Loop line by line
    let lines: Vec<&[Token]> = src.split(|t| t.token_type == NewLine).collect();

    'lines: for line in lines {
        // Ignore empty lines
        if line.len() == 0 {
            continue;
        }
        // Get line number in original text file of this line
        let line_no = line[0].line;
        // Type of construct on line is determined by the first token
        match line[0].token_type {
            //Variable assignment
            Identifier(assigned_to) => {
                // Check for correct formatting
                if line.len() == 1 || line[1].token_type != OperatorAssignment {
                    return Err(format!("Error on line {line_no}: Identifer at the beginning of a line must be followed by '='"));
                }

                // Get left hand side of expression
                match line.get(2) {
                    // Error if line ends here
                    None => return Err(format!("Error on line {line_no}: Expected identifier or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            program += &format!("LDA var_{s}\n");
                        },
                        Number(n) => {
                            // Optimisation for if a variable is initialised with a constant value
                            if line.len() == 3 && !vars.contains_key(assigned_to){
                                vars.insert(assigned_to, n);
                                continue;
                            }
                            // Add const to set
                            consts.insert(n);
                            // Emit code to load const
                            program += &format!("LDA const_{n}\n");
                        }
                        // If token is neither a variable or a number, error
                        _ => return Err(format!("Error on line {line_no} token 2: Expected identifier or number"))
                    }
                }

                // Get operator
                match line.get(3) {
                    // If line ends here, just store data
                    None => {
                        vars.insert(assigned_to, 0);
                        program += &format!("STA var_{assigned_to}\n");
                        continue
                    },
                    // Else, emit partial code to perform calculation
                    Some(t) => match t.token_type {
                        OperatorAdd => program += "ADD ",
                        OperatorSub => program += "SUB ",
                        _ => return Err(format!("Error on line {line_no} token 3: Expected '+' or '-'"))
                    }
                }

                // Emit address of right hand side of expression
                match line.get(4) {
                    None => return Err(format!("Error on line {line_no}: Expected identifer or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            program += &format!("var_{s}\n");
                        },
                        Number(n) => {
                            // Emit code to load const
                            consts.insert(n);
                            program += &format!("const_{n}\n")
                        },
                        _ => return Err(format!("Error on line {line_no} token 4: Expected identifer or number"))
                    }
                }
                // Emit code to store value
                program += &format!("STA var_{assigned_to}\n");

                // Error if too many tokens
                if line.get(5).is_some() {
                    return Err(format!("Error on line {line_no} token 5: Unexpected token"))
                }
                
                // Create variable if it does not already exist
                if !vars.contains_key(assigned_to) {
                    vars.insert(assigned_to, 0);
                }

            }
            //Input
            Input => {
                // Find where to put inputted value
                match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected identifier")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            // Create variable if it does not exist
                            if !vars.contains_key(s) {
                                vars.insert(s, 0);
                            }
                            // Emit code to input to variable
                            program += &format!("INP\nSTA var_{s}\n")
                        },
                        _ => return Err(format!("Error on line {line_no} token 1: Expected identifier"))
                    }
                }

                // Error if too many
                if line.get(2).is_some() {
                    return Err(format!("Error on line {line_no} token 2: Unexpected token"))
                }
            }
            //Output
            Output => {
                match line.get(1) {
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

                match line.get(2) {
                    None => {
                        program += &format!("OUT\n");
                        continue
                    },
                    Some(t) => match t.token_type {
                        OperatorAdd => program += "ADD ",
                        OperatorSub => program += "SUB ",
                        _ => return Err(format!("Error on line {line_no} token 3: Expected '+' or '-'"))
                    }
                }

                match line.get(3) {
                    None => return Err(format!("Error on line {line_no}: Expected identifer or number")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            program += &format!("var_{s}\n");
                        },
                        Number(n) => {
                            consts.insert(n);
                            program += &format!("const_{n}\n")
                        },
                        _ => return Err(format!("Error on line {line_no} token 4: Expected identifer or number"))
                    }
                }
                program += &format!("OUT\n");
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
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit variable name
                            format!("var_{s}\n")
                        },
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
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            format!("var_{s}\n")
                        },
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
            //Break
            Break => {
                for frame in scope_stack.iter().rev() {
                    if let Scope::While{start_line} = frame {
                        program += &format!("BRA while_{start_line}_end\n");
                        continue 'lines;
                    }
                }

                return Err(format!("Error on line {line_no}: 'break' while not in loop"));
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
                scope_stack.push(Scope::If { if_start_line: line_no, else_start_line: line_no , has_else: false});

                let lhs = match line.get(1) {
                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                    Some(t) => match t.token_type {
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            format!("var_{s}\n")
                        },
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
                        Identifier(s) => {
                            // Error if variable is not defined
                            if !vars.contains_key(s) {
                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                            }
                            // Emit code to load variable
                            format!("var_{s}\n")
                        },
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
                    Some(Scope::If { if_start_line, else_start_line, has_else: _ }) => match line.get(1) {
                        None => {
                            scope_stack.push(Scope::If { if_start_line: if_start_line, else_start_line: line_no, has_else: true });
                            program += &format!("BRA if_{if_start_line}_end\nif_{else_start_line}_else ");
                        },
                        Some(t) => match t.token_type {
                            If => {
                                scope_stack.push(Scope::If { if_start_line: if_start_line, else_start_line: line_no, has_else: true });

                                let lhs = match line.get(2) {
                                    None => return Err(format!("Error on line {line_no}: Expected condition formed of two arguments and a comparison operator")),
                                    Some(t) => match t.token_type {
                                        Identifier(s) => {
                                            // Error if variable is not defined
                                            if !vars.contains_key(s) {
                                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                                            }
                                            // Emit code to load variable
                                            format!("var_{s}\n")
                                        },
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
                                        Identifier(s) => {
                                            // Error if variable is not defined
                                            if !vars.contains_key(s) {
                                                return Err(format!("Error on line {line_no} token 2: Variable unknown identifier '{s}'"))
                                            }
                                            // Emit code to load variable
                                            format!("var_{s}\n")
                                        },
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
                    },
                    s => {
                        println!("{:?}", s);
                        println!("{:?}", scope_stack);
                        return Err(format!("Error on line {line_no}: expected 'else if' or just 'else'"))}
                }
            }
            //End if
            EndIf => {
                match scope_stack.pop() {
                    None => return Err(format!("Error on line {line_no}: 'endif' found while 'if' statement was not inner most control flow construct")),
                    Some(Scope::If { if_start_line, else_start_line: _, has_else }) => {
                        if has_else {
                            program += &format!("if_{if_start_line}_end ADD const_0\n")
                        }
                        else {
                            program += &format!("if_{if_start_line}_else ADD const_0\n")
                        }
                    }
                    _ => return Err(format!("Error on line {line_no}: 'endif' found while 'if' statement was not inner most control flow construct"))
                }
            }
            
            _ => return Err(format!("Error on line {line_no}: Expected assignment, input, output, or start or end of if statement or while loop"))
        }
    }

    program += "HLT\n\n";
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