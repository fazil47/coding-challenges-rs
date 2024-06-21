use std::{collections::HashMap, fmt::Error};

#[derive(Debug)]
enum JsonValue {
    String(String),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken(usize),
    UnexpectedEndOfInput,
    TrailingComma,
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            None => Err(ParseError::UnexpectedEndOfInput),
            _ => Err(ParseError::UnexpectedToken(self.position)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        let mut object: HashMap<String, JsonValue> = HashMap::new();
        self.consume(); // consume '{'
        loop {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.consume();
                break;
            }

            if let JsonValue::String(key) = self.parse_string()? {
                self.skip_whitespace();

                if self.peek() != Some(':') {
                    match self.peek() {
                        Some(_) => return Err(ParseError::UnexpectedToken(self.position)),
                        None => return Err(ParseError::UnexpectedEndOfInput),
                    }
                }
                self.consume(); // consume ':'

                let value = self.parse_value()?;
                object.insert(key, value);

                self.skip_whitespace();
                match self.consume() {
                    Some('}') => break,
                    Some(',') => {
                        self.skip_whitespace();
                        if self.peek() == Some('}') {
                            return Err(ParseError::TrailingComma);
                        }
                        continue;
                    }
                    _ => return Err(ParseError::UnexpectedEndOfInput),
                }
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let c = self.peek().ok_or(ParseError::UnexpectedEndOfInput)?;
        match c {
            '{' => self.parse_object(),
            '"' => self.parse_string(),
            _ => Err(ParseError::UnexpectedToken(self.position)),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        let mut s = String::new();
        self.consume(); // consume '"'
        loop {
            match self.consume() {
                Some('"') => break,
                Some(c) => s.push(c),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::String(s))
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.position += c.len_utf8();
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.consume();
        }
    }
}

fn parse_json(file_path: &str) -> Result<(), Error> {
    let file =
        std::fs::read(file_path).expect(format!("Invalid file path: {}", file_path).as_str());
    let json = std::str::from_utf8(file.as_slice())
        .expect(format!("Invalid content for file: {}", file_path).as_str());

    if json.len() == 0 {
        eprintln!("Empty file: {}", file_path);
        return Err(Error);
    }

    let mut parser = Parser::new(json);
    match parser.parse() {
        Ok(json) => {
            match json {
                JsonValue::Object(o) => println!("{:?}", o),
                _ => eprintln!("Invalid JSON object"),
            }

            Ok(())
        }
        Err(err) => {
            match err {
                ParseError::UnexpectedToken(pos) => {
                    eprintln!("Unexpected token at position {}", pos);
                }
                ParseError::UnexpectedEndOfInput => {
                    eprintln!("Unexpected end of input");
                }
                ParseError::TrailingComma => {
                    eprintln!("Trailing comma");
                }
            }
            Err(Error)
        }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        return Err(Error);
    }

    parse_json(args[1].as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_step(dir: &str) {
        std::fs::read_dir(dir).unwrap().for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let valid = entry
                .file_name()
                .into_string()
                .unwrap()
                .starts_with("valid");

            if valid {
                assert!(parse_json(path_str).is_ok());
            } else {
                assert!(parse_json(path_str).is_err());
            }
        });
    }

    #[test]
    fn test_step_1() {
        test_step("./tests/step1");
    }

    #[test]
    fn test_step_2() {
        test_step("./tests/step2");
    }

    #[test]
    fn test_step_3() {
        test_step("./tests/step3");
    }
}
