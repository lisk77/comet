use serde::Serialize;
use std::time::{Duration, Instant};

const MODULE: &str = "renderer2d";
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Default, Serialize)]
pub struct Renderer2DDiagnostics {
    pub cpu_frame_time_ms: f64,
    pub surface_wait_time_ms: f64,
    pub cpu_render_work_time_ms: f64,
    pub passes: u32,
    pub draw_calls: u32,
    pub sprite_instances: u32,
    pub glyphs: u32,
    pub pending_font_jobs: u32,
    pub uploaded_bytes: u64,
    pub presentation_interval_ms: f64,
    pub snapshot_sequence: u64,
    pub snapshot_age_ms: f64,
    pub reused_snapshots: u64,
    pub replaced_snapshots: u64,
}

pub(crate) struct Renderer2DDiagnosticsPublisher {
    diagnostics: comet_diagnostics::Diagnostics,
    last_publish: Instant,
}

impl Renderer2DDiagnosticsPublisher {
    pub(crate) fn from_env() -> Option<Self> {
        let diagnostics = comet_diagnostics::Diagnostics::from_env();
        diagnostics.is_enabled(MODULE).then(|| Self {
            diagnostics,
            last_publish: Instant::now() - SNAPSHOT_INTERVAL,
        })
    }

    pub(crate) fn publish(&mut self, snapshot: &Renderer2DDiagnostics) {
        if self.last_publish.elapsed() < SNAPSHOT_INTERVAL {
            return;
        }
        self.last_publish = Instant::now();
        self.diagnostics.publish(MODULE, "snapshot", snapshot);
    }
}
