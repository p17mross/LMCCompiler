#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Argument<'a> {
    Constant(i64),
    Label(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextToken {
    LabelOrInstruction,
    Instruction,
    None,
    ArgOrNone,
    Arg,
}

fn assemble(src: &str) -> [i32; 100] {
    //let mut bin: [i64; 100] = [0; 100];
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let mut ip: usize = 0;
    let mut references: [(i64, Argument); 100] = [(0, Argument::Constant(0)); 100];

    for (i, line) in program.lines().enumerate() {
        let tokens: Vec<&str> = line.split("//").collect::<Vec<&str>>()[0].split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }
        if tokens.len() > 3 {
            println!("Too many tokens on line {}", i);
            return Err(());
        }

        let mut next_token = NextToken::LabelOrInstruction;

        for token in tokens {

            if next_token == NextToken::None {
                println!("Unexpected token '{}' on line {}", token, i);
                return Err(())
            }

            if vec![NextToken::Arg, NextToken::ArgOrNone].contains(&next_token) {
                match str::parse(token) {
                    Ok(n) => references[ip].1 = Argument::Constant(n),
                    Err(_) => references[ip].1 = Argument::Label(token)
                }
            }

            match token {
                "dat" => {references[ip].0 = 0; next_token = NextToken::ArgOrNone},
                "inp" => {references[ip].0 = 901; next_token = NextToken::None},
                "out" => {references[ip].0 = 902; next_token = NextToken::None},
                "lda" => {references[ip].0 = 500; next_token = NextToken::Arg},
                "add" => {references[ip].0 = 100; next_token = NextToken::Arg},
                "sub" => {references[ip].0 = 200; next_token = NextToken::Arg},
                "brp" => {references[ip].0 = 800; next_token = NextToken::Arg},
                "brz" => {references[ip].0 = 700; next_token = NextToken::Arg},
                "bra" => {references[ip].0 = 600; next_token = NextToken::Arg},
                "hlt" => {references[ip].0 = 0; next_token = NextToken::None},
                s => {labels.insert(s, ip);}
            }
            ip += 1
        }
    } 

    todo!();
}