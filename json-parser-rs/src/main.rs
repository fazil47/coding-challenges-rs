use std::fmt::Error;

fn parse_file(file_path: &str) -> Result<(), Error> {
    let file =
        std::fs::read(file_path).expect(format!("Invalid file path: {}", file_path).as_str());
    let json = std::str::from_utf8(file.as_slice())
        .expect(format!("Invalid content for file: {}", file_path).as_str());

    if json.len() == 0 {
        eprintln!("Empty file: {}", file_path);
        return Err(Error);
    }

    let mut unmatched_opening_braces = 0;
    for c in json.chars() {
        if c == '{' {
            unmatched_opening_braces += 1;
        } else {
            if unmatched_opening_braces == 0 {
                eprintln!("Invalid content for file: {}", file_path);
                return Err(Error);
            }

            unmatched_opening_braces -= 1;
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        return Err(Error);
    }

    parse_file(args[1].as_str())
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
                assert!(parse_file(path_str).is_ok());
            } else {
                assert!(parse_file(path_str).is_err());
            }
        });
    }

    #[test]
    fn test_step_1() {
        test_step("./tests/step1");
    }
}
