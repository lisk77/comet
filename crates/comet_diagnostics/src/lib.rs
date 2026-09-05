use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::{create_dir_all, File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
    sync::{mpsc, Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

const CHANNEL_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct Diagnostics {
    enabled_modules: Arc<EnabledModules>,
    sender: Option<mpsc::SyncSender<Record>>,
}

struct EnabledModules {
    selected: Option<HashSet<String>>,
}

struct Record {
    timestamp_ms: u128,
    module: &'static str,
    kind: &'static str,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct JsonlRecord<'a> {
    timestamp_ms: u128,
    module: &'static str,
    kind: &'static str,
    data: &'a serde_json::Value,
}

impl Diagnostics {
    pub fn from_env() -> Self {
        static DIAGNOSTICS: OnceLock<Diagnostics> = OnceLock::new();
        DIAGNOSTICS
            .get_or_init(|| {
                let selected = std::env::var("COMET_DIAGNOSTICS").ok().map(|modules| {
                    modules
                        .split(',')
                        .map(str::trim)
                        .filter(|module| !module.is_empty())
                        .map(str::to_owned)
                        .collect()
                });
                let sender = selected.as_ref().and_then(|_| start_jsonl_writer());
                Diagnostics {
                    enabled_modules: Arc::new(EnabledModules { selected }),
                    sender,
                }
            })
            .clone()
    }

    pub fn is_enabled(&self, module: &str) -> bool {
        self.sender.is_some()
            && self
                .enabled_modules
                .selected
                .as_ref()
                .is_none_or(|modules| modules.contains(module))
    }

    pub fn publish<T>(&self, module: &'static str, kind: &'static str, data: &T)
    where
        T: Serialize,
    {
        let Some(sender) = &self.sender else {
            return;
        };
        if !self.is_enabled(module) {
            return;
        }
        let Ok(data) = serde_json::to_value(data) else {
            return;
        };
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let _ = sender.try_send(Record {
            timestamp_ms,
            module,
            kind,
            data,
        });
    }
}

fn start_jsonl_writer() -> Option<mpsc::SyncSender<Record>> {
    let (sender, receiver) = mpsc::sync_channel::<Record>(CHANNEL_CAPACITY);
    std::thread::Builder::new()
        .name("comet-diagnostics".to_owned())
        .spawn(move || {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            let mut writers = HashMap::<&'static str, BufWriter<File>>::new();
            while let Ok(record) = receiver.recv() {
                let Some(writer) = writer_for(&mut writers, record.module, timestamp) else {
                    continue;
                };
                if write_record(writer, &record).is_err() {
                    writers.remove(record.module);
                }
            }
        })
        .ok()?;
    Some(sender)
}

fn writer_for<'a>(
    writers: &'a mut HashMap<&'static str, BufWriter<File>>,
    module: &'static str,
    timestamp: u128,
) -> Option<&'a mut BufWriter<File>> {
    match writers.entry(module) {
        Entry::Occupied(entry) => Some(entry.into_mut()),
        Entry::Vacant(entry) => {
            let directory = Path::new(".comet")
                .join("diagnostics")
                .join(module_directory(module));
            create_dir_all(&directory).ok()?;
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(directory.join(format!("{timestamp}.jsonl")))
                .ok()?;
            Some(entry.insert(BufWriter::new(file)))
        }
    }
}

fn write_record(writer: &mut BufWriter<File>, record: &Record) -> Result<(), ()> {
    let json = JsonlRecord {
        timestamp_ms: record.timestamp_ms,
        module: record.module,
        kind: record.kind,
        data: &record.data,
    };
    serde_json::to_writer(&mut *writer, &json).map_err(|_| ())?;
    writer.write_all(b"\n").map_err(|_| ())?;
    writer.flush().map_err(|_| ())
}

fn module_directory(module: &str) -> String {
    module
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => character,
            _ => '_',
        })
        .collect()
}
