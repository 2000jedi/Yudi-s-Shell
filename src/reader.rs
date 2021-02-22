use std::fs;

pub struct Reader {
    f : String,
    index : usize,
}

impl Reader {
    pub fn from_string(things: String) -> Reader {
        Reader {f: things, index: 0}
    }

    pub fn from_file(path: String) -> Reader {
        let f = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_e) => panic!("File Not Found"),
        };
        Reader {f: f, index: 0}
    }

    pub fn has_next(&mut self) -> bool {
        match self.f.bytes().nth(self.index) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        match self.f.bytes().nth(self.index) {
            Some(val) => Some(val as char),
            None => None,
        }
    }

    pub fn consume(&mut self, curr : char) -> bool {
        match self.f.bytes().nth(self.index) {
        Some(val) => {
            if val == curr as u8 {
            self.index += 1;
            return true;
            } else {
            return false;
            }
        },
        None => return false,
        };
    }
}
