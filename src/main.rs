use winreg::enums::*;
use winreg::RegKey;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io;


#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Print,
    PrintCal, // New token for printcal keyword
    Text(String),
    EndOfInput,
}

struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Token {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                self.input.next();
                continue;
            }
            if c.is_digit(10) || c == '.' {
                let mut num = 0.0;
                let mut after_point = false;
                let mut factor = 10.0;

                while let Some(&next) = self.input.peek() {
                    if next.is_digit(10) {
                        if after_point {
                            num += next.to_digit(10).unwrap() as f64 / factor;
                            factor *= 10.0;
                        } else {
                            num = num * 10.0 + next.to_digit(10).unwrap() as f64;
                        }
                        self.input.next(); // Consume the digit
                    } else if next == '.' && !after_point {
                        self.input.next(); // Consume the dot
                        after_point = true;
                    } else {
                        break;
                    }
                }
                return Token::Number(num);
            }
            if let Some(&'p') = self.input.peek() {
                let mut keyword = String::new();
                for _ in 0..8 { // Increase the loop count to 8 to capture "printcal"
                    if let Some(&c) = self.input.peek() {
                        if c.is_alphabetic() || c == '_' {
                            keyword.push(self.input.next().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                match keyword.as_str() {
                    "printcal" => return Token::PrintCal,
                    "print" => return Token::Print,
                    _ => panic!("Unexpected keyword: {}", keyword),
                }
            }            
            if let Some(&'"') = self.input.peek() {
                self.input.next(); // Consume the opening quote
                let mut text = String::new();
                while let Some(&next) = self.input.peek() {
                    if next == '"' {
                        self.input.next(); // Consume the closing quote
                        break;
                    }
                    text.push(self.input.next().unwrap());
                }
                return Token::Text(text);
            }
            match c {
                '+' => {
                    self.input.next();
                    return Token::Plus;
                }
                '-' => {
                    self.input.next();
                    return Token::Minus;
                }
                '*' => {
                    self.input.next();
                    return Token::Multiply;
                }
                '/' => {
                    self.input.next();
                    return Token::Divide;
                }
                _ => panic!("Unexpected character: {}", c),
            }
        }
        Token::EndOfInput
    }
}

fn parse_expression(lexer: &mut Lexer) -> f64 {
    let mut value = match lexer.next_token() {
        Token::Number(num) => num,
        _ => panic!("Expected a number"),
    };

    loop {
        match lexer.next_token() {
            Token::Plus => {
                value += match lexer.next_token() {
                    Token::Number(num) => num,
                    _ => panic!("Expected a number"),
                };
            }
            Token::Minus => {
                value -= match lexer.next_token() {
                    Token::Number(num) => num,
                    _ => panic!("Expected a number"),
                };
            }
            Token::Multiply => {
                value *= match lexer.next_token() {
                    Token::Number(num) => num,
                    _ => panic!("Expected a number"),
                };
            }
            Token::Divide => {
                let divisor = match lexer.next_token() {
                    Token::Number(num) => num,
                    _ => panic!("Expected a number"),
                };
                if divisor == 0.0 {
                    panic!("Division by zero");
                }                
                value /= divisor;
            }
            Token::EndOfInput => break,
            _ => panic!("Unexpected token"),
        }
    }

    value
}

fn interpret(code: &str) {
    let mut lexer = Lexer::new(code);
    let mut line_number = 1;

    loop {
        match lexer.next_token() {
            Token::Print => {
                match lexer.next_token() {
                    Token::Text(text) => {
                        println!("{}", text);
                    }
                    _ => {
                        eprintln!("ERROR[0002] Unexpected token in line {}", line_number);
                        return;
                    },
                }
            }
            Token::PrintCal => {
                let next_token = lexer.next_token();
                match next_token {
                    Token::Number(num) => {
                        let remaining_code = code.split_at(lexer.input.clone().count()).1;
                        let value = parse_expression(&mut Lexer::new(&format!("{} {}", num, remaining_code)));
                        println!("{}", value);
                    }
                    _ => {
                        eprintln!("ERROR[0001] Use print instead of printcal in line {}", line_number);
                        return;
                    },
                }
            }
            Token::EndOfInput => break,
            _ => {
                eprintln!("ERROR[0003] Unexpected token in line {}", line_number);
                return;
            },
        }

        line_number += 1;
    }
}

fn associate_exe_and_ico(exe_path: &str, ico_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let (mly, _) = hkcr.create_subkey(".mly")?;
    mly.set_value("", &"mly_file")?;

    let (mly_file, _) = hkcr.create_subkey("mly_file")?;
    let (default_icon, _) = mly_file.create_subkey("DefaultIcon")?;
    default_icon.set_value("", &ico_path)?;

    let (shell, _) = mly_file.create_subkey("shell")?;
    let (open, _) = shell.create_subkey("open")?;
    let (command, _) = open.create_subkey("command")?;
    command.set_value("", &(exe_path.to_string() + " \"%1\""))?;

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let mut file = File::open(filename)?;
        let mut code = String::new();
        file.read_to_string(&mut code)?;
        interpret(&code);
    } else {
        let ico_path = "C://Users/raz/desktop/mainly_lang/public/mainly.ico";
        let exe_path = env::current_exe().unwrap().to_str().unwrap().to_string();
        match associate_exe_and_ico(&exe_path, ico_path) {
            Ok(_) => println!("EXE and icon successfully associated with .mly files."),
            Err(e) => eprintln!("Error associating EXE and icon: {}", e),
        }
    }

    println!("Press Enter to exit...");
    io::stdin().read_line(&mut String::new())?;

    Ok(())
}