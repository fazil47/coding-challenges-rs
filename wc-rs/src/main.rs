use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let first_arg = if args.len() >= 2 {
        args[1].as_str()
    } else {
        ""
    };
    let filepath =
        if first_arg == "-c" || first_arg == "-l" || first_arg == "-w" || first_arg == "-m" {
            if args.len() >= 3 {
                args[2].as_str()
            } else {
                ""
            }
        } else {
            first_arg
        };
    let content = if filepath == "" {
        let mut buffer: Vec<u8> = Vec::new();
        std::io::stdin().read_to_end(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    } else {
        std::fs::read_to_string(filepath).unwrap()
    };
    let flag = first_arg;

    if flag == "-c" {
        println!(" {} {}", content.bytes().count(), filepath);
    } else if flag == "-l" {
        println!(" {} {}", content.lines().count(), filepath);
    } else if flag == "-w" {
        println!(" {} {}", content.split_whitespace().count(), filepath);
    } else if flag == "-m" {
        println!(" {} {}", content.chars().count(), filepath);
    } else {
        println!(
            "\t{}\t{}\t{}\t{}",
            content.lines().count(),
            content.split_whitespace().count(),
            content.bytes().count(),
            filepath
        );
    }
}
