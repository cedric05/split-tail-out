use std::collections::HashMap;
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use clap::Parser;

/// Split Tail Output
/// splits tail output into multiple files
#[derive(Parser)]
struct CommandArgs {
    /// provide tail output captured file
    /// if filename is not provided, input will be captured from stdin
    #[clap(long, short)]
    filename: Option<String>,
}

const FILENAME_REGEX: &str = "==> (?P<filename>.*) <==";

fn process<T>(lines: std::io::Lines<T>) -> anyhow::Result<()>
where
    T: std::io::BufRead,
{
    let re = regex::Regex::new(FILENAME_REGEX).context("regex for sure compiles")?;
    let mut file_name_map = HashMap::new();
    let mut running_file = None;
    for line in lines {
        let mut line = line.context("unable to capture line")?;
        match re.captures(&line) {
            Some(cap) => {
                let full_path = cap
                    .name("filename")
                    .context("captured regex should contain filename for sure")?
                    .as_str();
                let file_name = Path::new(full_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                if !file_name_map.contains_key(&file_name) {
                    let file = File::options()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&file_name)
                        .context("unable to open log file")?;
                    file_name_map.insert(file_name.clone(), file);
                }
                running_file = file_name_map.get_mut(&file_name);
            }
            None => match &mut running_file {
                Some(file) => {
                    line.push('\n');
                    file.write_all(&line.as_bytes())?;
                }
                None => {}
            },
        }
    }
    Ok(())
}
fn main() -> anyhow::Result<()> {
    let args = CommandArgs::parse();
    match args.filename {
        Some(filename) => {
            let file = File::open(filename).context("unable to open file with given filename")?;
            process(std::io::BufReader::new(file).lines())
        }
        None => {
            let stdin_file = stdin();
            process(stdin_file.lines())
        }
    }
}
