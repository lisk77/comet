use serde::Serialize;
use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
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
                let sender = std::env::var("COMET_DIAGNOSTICS_OUTPUT")
                    .ok()
                    .and_then(|output| output.strip_prefix("file:").map(PathBuf::from))
                    .as_ref()
                    .and_then(start_jsonl_writer);
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

fn start_jsonl_writer(path: &PathBuf) -> Option<mpsc::SyncSender<Record>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .ok()?;
    let (sender, receiver) = mpsc::sync_channel::<Record>(CHANNEL_CAPACITY);
    std::thread::Builder::new()
        .name("comet-diagnostics".to_owned())
        .spawn(move || {
            let mut writer = BufWriter::new(file);
            while let Ok(record) = receiver.recv() {
                let json = JsonlRecord {
                    timestamp_ms: record.timestamp_ms,
                    module: record.module,
                    kind: record.kind,
                    data: &record.data,
                };
                if serde_json::to_writer(&mut writer, &json).is_err()
                    || writer.write_all(b"\n").is_err()
                    || writer.flush().is_err()
                {
                    break;
                }
            }
        })
        .ok()?;
    Some(sender)
}
