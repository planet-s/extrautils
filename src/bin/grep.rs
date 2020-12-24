extern crate extra;

use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;

static MAN_PAGE: &str = /* @MANSTART{grep} */ r#"
NAME
    grep - print lines matching a pattern

SYNOPSIS
    grep [--help] [-chHnv] PATTERN [FILE...]

DESCRIPTION
    grep searches the named input FILEs for lines containing a match to the given PATTERN. If no
    files are specified, grep searches the standard input. grep prints the matching lines.

OPTIONS
    -c
    --count
        Print count of matching lines, instead of those lines.

    -H
    --with-filename
        Include filename header with each match (default for multiple files).

    -h
    --no-filename
        Never include filename header (default for single files or stdin).

    --help
        Print this manual page.

    -n
    --line-number
        Prefix each line of output with the line number of the match.

    -v
    --invert-match
        Invert matching.
"#; /* @MANEND */

#[derive(Copy, Clone)]
struct Flags {
    line_numbers: bool,
    invert_match: bool,
    count: bool,
    filename_headers: Option<bool>,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            line_numbers: false,
            invert_match: false,
            count: false,
            filename_headers: None,
        }
    }
}

fn main() {
    let args = env::args().skip(1);
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut flags = Flags::new();
    let mut pattern = String::new();
    let mut files = Vec::with_capacity(args.len());
    for arg in args {
        if arg.starts_with('-') {
            match arg.as_str() {
                "--help" => {
                    print!("{}", MAN_PAGE);
                    exit(0);
                }
                "-h" | "--no-filename" => flags.filename_headers = Some(false),
                "-H" | "--with-filename" => flags.filename_headers = Some(true),
                "-n" | "--line-number" => flags.line_numbers = true,
                "-v" | "--invert-match" => flags.invert_match = true,
                "-c" | "--count" => flags.count = true,
                _ => {
                    eprintln!("Error, unknown option: {}", arg);
                    exit(2);
                }
            }
        } else if pattern.is_empty() {
            pattern = arg.clone();
        } else {
            files.push(arg)
        }
    }

    if pattern.is_empty() {
        eprintln!("You must provide a pattern");
        exit(2);
    }
    if flags.filename_headers == None {
        flags.filename_headers = Some(files.len() > 1);
    }

    let mut found = false;
    let mut error = false;
    if files.is_empty() {
        found = do_simple_search(BufReader::new(stdin), "(standard input)", &pattern, flags);
    } else {
        for path in files {
            match File::open(&Path::new(&path)) {
                Ok(f) => {
                    found |= do_simple_search(BufReader::new(f), &path, &pattern, flags);
                }
                Err(err) => {
                    eprintln!("Error opening {}: {}", path, err);
                    error = true;
                }
            }
        }
    }
    if error {
        exit(2);
    }
    if !found {
        exit(1);
    }
}

fn do_simple_search<T: BufRead>(reader: T, path: &str, pattern: &str, flags: Flags) -> bool {
    let mut count = 0;
    for (line_num, result) in reader.lines().enumerate() {
        if let Ok(line) = result {
            let mut is_match = line.contains(pattern);
            if flags.invert_match {
                is_match = !is_match
            }
            if is_match {
                count += 1;
                if !flags.count {
                    if flags.filename_headers.unwrap() {
                        print!("{}:", path);
                    }
                    if flags.line_numbers {
                        print!("{}:", line_num + 1);
                    }
                    println!("{}", line);
                }
            }
        }
    }

    if flags.count {
        if flags.filename_headers.unwrap() {
            print!("{}:", path);
        }
        println!("{}", count);
    }

    return count > 0;
}
