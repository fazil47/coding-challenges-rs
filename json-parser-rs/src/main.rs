use indexmap::IndexMap;
use std::fmt::Error;

enum JsonValue {
    Null,
    Boolean(bool),
    Number(f32),
    String(String),
    Array(Vec<JsonValue>),
    Object(IndexMap<String, JsonValue>),
}

impl ToString for JsonValue {
    fn to_string(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Array(a) => {
                let mut s = "[".to_string();
                for value in a.iter() {
                    s.push_str(&format!("{}, ", value.to_string()));
                }
                s.pop(); // Pop final comma
                s.pop(); // Pop final space
                s.push(']');
                s
            }
            JsonValue::Object(o) => {
                let mut s = "{".to_string();
                for (key, value) in o.iter() {
                    s.push_str(&format!("\"{}\": {}, ", key, value.to_string()));
                }
                s.pop(); // Pop final comma
                s.pop(); // Pop final space
                s.push('}');
                s
            }
        }
    }
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken(usize),
    UnexpectedEndOfInput,
    TrailingComma(usize),
    MaxDepthExceeded(usize),
    LeadingZero(usize),
}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

const MAX_DEPTH: u32 = 20;

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let res = match self.peek() {
            Some('{') => self.parse_object(0),
            Some('[') => self.parse_array(0),
            None => Err(ParseError::UnexpectedEndOfInput),
            _ => Err(ParseError::UnexpectedToken(self.position)),
        };

        self.skip_whitespace();
        match self.peek() {
            Some(_) => Err(ParseError::UnexpectedToken(self.position)),
            None => res,
        }
    }

    fn parse_value(&mut self, depth: u32) -> Result<JsonValue, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(ParseError::MaxDepthExceeded(self.position));
        }

        self.skip_whitespace();
        let c = self.peek().ok_or(ParseError::UnexpectedEndOfInput)?;
        // Match object, string, boolean, null and number
        match c {
            '{' => self.parse_object(depth),
            '[' => self.parse_array(depth),
            '"' => self.parse_string(),
            't' | 'f' => self.parse_boolean(),
            'n' => self.parse_null(),
            c if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError::UnexpectedToken(self.position)),
        }
    }

    fn parse_object(&mut self, depth: u32) -> Result<JsonValue, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(ParseError::MaxDepthExceeded(self.position));
        }

        let mut object: IndexMap<String, JsonValue> = IndexMap::new();
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

                let value = self.parse_value(depth + 1)?;
                object.insert(key, value);

                self.skip_whitespace();
                match self.consume() {
                    Some('}') => break,
                    Some(',') => {
                        self.skip_whitespace();
                        if self.peek() == Some('}') {
                            return Err(ParseError::TrailingComma(self.position));
                        }
                        continue;
                    }
                    _ => return Err(ParseError::UnexpectedEndOfInput),
                }
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse_array(&mut self, depth: u32) -> Result<JsonValue, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(ParseError::MaxDepthExceeded(self.position));
        }

        let mut array: Vec<JsonValue> = Vec::new();
        self.consume(); // consume '['

        loop {
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.consume();
                break;
            }

            let value = self.parse_value(depth + 1)?;
            array.push(value);

            self.skip_whitespace();
            match self.consume() {
                Some(']') => break,
                Some(',') => {
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        return Err(ParseError::TrailingComma(self.position));
                    }
                    continue;
                }
                _ => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        let mut s = String::new();
        self.consume(); // consume '"'

        loop {
            match self.consume() {
                Some('"') => break,

                // Error if tab, newline, or carriage return is not escaped
                Some('\t') | Some('\n') | Some('\r') => {
                    return Err(ParseError::UnexpectedToken(self.position))
                }

                // Handle escape characters
                Some('\\') => match self.consume() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\u{0008}'),
                    Some('f') => s.push('\u{000C}'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            match self.consume() {
                                Some(c) if c.is_digit(16) => hex.push(c),
                                Some(_) => return Err(ParseError::UnexpectedToken(self.position)),
                                None => return Err(ParseError::UnexpectedEndOfInput),
                            }
                        }
                        let codepoint = u32::from_str_radix(&hex, 16).unwrap();
                        match std::char::from_u32(codepoint) {
                            Some(c) => s.push(c),
                            None => return Err(ParseError::UnexpectedToken(self.position)),
                        }
                    }
                    Some(_) => return Err(ParseError::UnexpectedToken(self.position)),
                    None => return Err(ParseError::UnexpectedEndOfInput),
                },

                Some(c) => s.push(c),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::String(s))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, ParseError> {
        let mut s = String::new();

        loop {
            match self.peek() {
                Some(c) if c.is_alphabetic() => {
                    s.push(c);
                    self.consume();
                }
                _ => break,
            }
        }

        match s.as_str() {
            "true" => Ok(JsonValue::Boolean(true)),
            "false" => Ok(JsonValue::Boolean(false)),
            _ => Err(ParseError::UnexpectedToken(self.position)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        let mut s = String::new();

        loop {
            match self.peek() {
                Some(c) if c.is_alphabetic() => {
                    s.push(c);
                    self.consume();
                }
                _ => break,
            }
        }

        if s == "null" {
            Ok(JsonValue::Null)
        } else {
            Err(ParseError::UnexpectedToken(self.position))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let mut s = String::new();
        let mut is_negative = false;
        let mut is_float = false;

        if self.peek() == Some('-') {
            s.push('-');
            self.consume();
            is_negative = true;
        }

        loop {
            match self.peek() {
                Some(c) if c.is_digit(10) => {
                    s.push(c);
                    self.consume();

                    match self.peek() {
                        Some('.') => {
                            self.consume();
                            s.push(c);
                            is_float = true;
                        }
                        Some('e') | Some('E') => {
                            self.consume();
                            s.push(c);

                            match self.peek() {
                                Some('+') | Some('-') => {
                                    s.push(c);
                                    self.consume();
                                }
                                _ => (),
                            }

                            match self.peek() {
                                Some(c) if c.is_digit(10) => (),
                                Some(_) => return Err(ParseError::UnexpectedToken(self.position)),
                                None => return Err(ParseError::UnexpectedEndOfInput),
                            }
                        }
                        _ => (),
                    }
                }
                _ => break,
            }
        }

        if is_negative && s.len() == 1 {
            return Err(ParseError::UnexpectedToken(self.position));
        }

        if !is_float && s.len() > 1 && (s.starts_with('0') || s.starts_with("-0")) {
            return Err(ParseError::LeadingZero(self.position));
        }

        let number: f32 = s.parse().unwrap();
        Ok(JsonValue::Number(number))
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
    let file: Vec<u8> =
        std::fs::read(file_path).expect(format!("Invalid file path: {}", file_path).as_str());
    let json: &str = std::str::from_utf8(file.as_slice())
        .expect(format!("Invalid content for file: {}", file_path).as_str());

    if json.len() == 0 {
        eprintln!("Empty file: {}", file_path);
        return Err(Error);
    }

    let mut parser = Parser::new(json);
    match parser.parse() {
        Ok(json) => {
            match json {
                JsonValue::Object(_) => println!("Valid JSON: {}", json.to_string()),
                JsonValue::Array(_) => println!("Valid JSON: {}", json.to_string()),
                _ => eprintln!("Invalid JSON"),
            }

            Ok(())
        }
        Err(err) => {
            match err {
                ParseError::UnexpectedToken(pos) => {
                    eprintln!(
                        "\n\
                        Unexpected token at position {}:\n\
                        {}\n\
                        {}^\n",
                        pos,
                        &json,
                        " ".repeat(pos),
                    );
                }
                ParseError::UnexpectedEndOfInput => {
                    eprintln!(
                        "\n\
                        Unexpected end of input:\n\
                        {}\n\
                        {}^\n",
                        &json,
                        " ".repeat(json.len() - 1),
                    );
                }
                ParseError::TrailingComma(pos) => {
                    eprintln!(
                        "\n\
                        Trailing comma at position {}:\n\
                        {}\n\
                        {}^\n",
                        pos,
                        &json,
                        " ".repeat(pos),
                    );
                }
                ParseError::MaxDepthExceeded(pos) => {
                    eprintln!(
                        "\n\
                        Max depth exceeded at position {}:\n\
                        {}\n\
                        {}^\n",
                        pos,
                        &json,
                        " ".repeat(pos),
                    );
                }
                ParseError::LeadingZero(pos) => {
                    eprintln!(
                        "\n\
                        Leading zero at position {}:\n\
                        {}\n\
                        {}^\n",
                        pos,
                        &json,
                        " ".repeat(pos),
                    );
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
            let filename = entry.file_name().into_string().unwrap();
            let valid = filename.starts_with("valid");

            if valid {
                assert!(
                    parse_json(path_str).is_ok(),
                    "Failed to parse: {}",
                    filename
                );
            } else {
                assert!(
                    parse_json(path_str).is_err(),
                    "Unexpectedly parsed: {}",
                    filename
                );
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

    #[test]
    fn test_step_4() {
        test_step("./tests/step4");
    }

    #[test]
    fn test_full() {
        test_step("./tests/full");
    }
}
