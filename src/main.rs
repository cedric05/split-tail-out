use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

use anyhow::Context;

const FILENAME_REGEX: &str = "==> (?P<filename>.*) <==";
fn main() -> anyhow::Result<()> {
    let mut args = args();
    let filename = args.nth(1).context("filename argument is required")?;
    let file = File::open(filename).context("unable to open file with given filename")?;
    let re = regex::Regex::new(FILENAME_REGEX).context("regex for sure compiles")?;
    let mut file_name_map = HashMap::new();
    let mut running_file = None;
    for line in std::io::BufReader::new(file).lines() {
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
