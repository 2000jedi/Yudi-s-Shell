use super::reader;

#[derive(Debug)]
pub enum Token {
    Opr(char),
    Name(String),
    Error(char),
    Empty,
    EOF,
}

const SEPS : [char; 3] = [' ', '\r', '\n'];
const OPERATORS : [char; 4] = [',', '|', '>', '&'];
    
pub fn next_token(r : &mut reader::Reader) -> Token {
    match r.peek() {
        Some(val) => {
            if SEPS.contains(& val){ // whitespace
                r.consume(val);
                return next_token(r);
            }

            if ! OPERATORS.contains(& val) {
                // name(id)
                let mut string : Vec<u8> = Vec::new();
                while match r.peek() {
                    Some(val2) => (! OPERATORS.contains(& val2)) && (! SEPS.contains(& val2)),
                    None => false,
                } {
                    let cur_val : char = r.peek().unwrap();
                    string.push(cur_val as u8);
                    r.consume(cur_val);
                }

                let token_val : String = match String::from_utf8(string) {
                    Ok(v) => v,
                    Err(_) => panic!("file not encoded in utf-8")
                };
                return Token::Name(token_val);
            }

            if val == '"' {
                // name(id)
                r.consume(val);

                let mut string : Vec<u8> = Vec::new();
                while match r.peek() {
                    Some(val2) => val2 != '"',
                    None => false,
                } {
                    let cur_val : char = r.peek().unwrap();
                    string.push(cur_val as u8);
                    r.consume(cur_val);
                }

                let token_val : String = match String::from_utf8(string) {
                    Ok(v) => v,
                    Err(_) => panic!("file not encoded in utf-8")
                };

                r.consume('"');

                return Token::Name(token_val);
            }

            if OPERATORS.contains(&val) {
                r.consume(val);
                return Token::Opr(val);
            }
            r.consume(val);
            return Token::Error(val);
        },
        None => {
            return Token::EOF;
        },
    };
}
