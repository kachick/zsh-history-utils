use eyre::Result;
use std::collections::BTreeMap;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

use crate::zsh::history::{HistoryEntry, HistoryLines};

pub fn run(paths: &[PathBuf], filter: &Option<PathBuf>) -> Result<()> {
    let mut entries_map: BTreeMap<u64, Vec<HistoryEntry>> = BTreeMap::new();

    for path in paths {
        let file = std::fs::File::open(path)?;
        for line in HistoryLines::new(file) {
            let entry = HistoryEntry::parse(&line?)?;

            let is_target = match filter {
                None => true,
                Some(p) => {
                    let output = Command::new(p)
                        .env("ZSH_HISTORY_UTILS_COMMAND", &entry.command)
                        .output()
                        .expect("should execute process");
                    output.status.success()
                }
            };

            if is_target {
                let key = entry.start_time;
                match entries_map.get_mut(&key) {
                    None => {
                        entries_map.insert(key, vec![entry]);
                    }
                    Some(entries) => {
                        entries.push(entry);
                    }
                }
            }
        }
    }

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for entries in entries_map.values() {
        for entry in entries {
            stdout.write_all(&entry.to_bytes())?;
        }
    }

    Ok(())
}
