use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let first_arg = args.get(1).map(|arg| arg.as_str()).unwrap_or("");
    let filepath = match first_arg {
        "-c" | "-l" | "-w" | "-m" => args.get(2).map(|arg| arg.as_str()).unwrap_or(""),
        _ => first_arg,
    };

    let content = if filepath.is_empty() {
        let mut buffer = Vec::new();
        std::io::stdin().read_to_end(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    } else {
        std::fs::read_to_string(filepath).unwrap()
    };

    let flag = first_arg;

    match flag {
        "-c" => println!(" {} {}", content.bytes().count(), filepath),
        "-l" => println!(" {} {}", content.lines().count(), filepath),
        "-w" => println!(" {} {}", content.split_whitespace().count(), filepath),
        "-m" => println!(" {} {}", content.chars().count(), filepath),
        _ => println!(
            "\t{}\t{}\t{}\t{}",
            content.lines().count(),
            content.split_whitespace().count(),
            content.bytes().count(),
            filepath
        ),
    }
}
