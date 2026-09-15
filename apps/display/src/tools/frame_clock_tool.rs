// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/frame_clock_tool.rs
// # 📌 Amac: Native display RFB update, present hizi ve gercek render suresi telemetrisini olcer
// # 📌 Modul - Rust
// # Version: 0.40.2
// # Aciklama: FPS'i 1000/fps ile frametime gibi gostermek yerine bir saniyelik RFB/present oranlarini ve olculen render duration ortalamasini ayri uretir
// # Bagimli Oldugu Katman: Tool | View

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::tools::rfb_client_tool::RfbTelemetrySnapshot;

const RENDER_SAMPLE_WINDOW: usize = 120;
const TITLE_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const BITS_PER_BYTE: f64 = 8.0;
const BITS_PER_MEGABIT: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameTelemetrySnapshot {
    pub rfb_updates_per_second: f64,
    pub present_fps: f64,
    pub average_render_time_ms: f64,
    pub rfb_megabits_per_second: f64,
    pub dropped_frames: u64,
}

pub struct FrameClockTool {
    render_samples: VecDeque<Duration>,
    presents_since_sample: u64,
    last_sample_at: Instant,
    last_rfb_updates: u64,
    last_rfb_raw_bytes: u64,
}

impl FrameClockTool {
    pub fn new() -> Self {
        Self {
            render_samples: VecDeque::with_capacity(RENDER_SAMPLE_WINDOW),
            presents_since_sample: 0,
            last_sample_at: Instant::now(),
            last_rfb_updates: 0,
            last_rfb_raw_bytes: 0,
        }
    }

    pub fn record_present(&mut self, render_duration: Duration) {
        self.presents_since_sample = self.presents_since_sample.saturating_add(1);
        self.render_samples.push_back(render_duration);
        while self.render_samples.len() > RENDER_SAMPLE_WINDOW {
            self.render_samples.pop_front();
        }
    }

    pub fn sample(
        &mut self,
        rfb: RfbTelemetrySnapshot,
    ) -> Option<FrameTelemetrySnapshot> {
        let elapsed = self.last_sample_at.elapsed();
        if elapsed < TITLE_SAMPLE_INTERVAL {
            return None;
        }
        let elapsed_seconds = elapsed.as_secs_f64();
        let rfb_updates = rfb
            .framebuffer_updates
            .saturating_sub(self.last_rfb_updates);
        let rfb_raw_bytes = rfb.raw_bytes.saturating_sub(self.last_rfb_raw_bytes);
        let _transport_totals = (rfb.rectangles, rfb.frames_enqueued);
        let present_fps = self.presents_since_sample as f64 / elapsed_seconds;
        let rfb_updates_per_second = rfb_updates as f64 / elapsed_seconds;
        let rfb_megabits_per_second =
            rfb_raw_bytes as f64 * BITS_PER_BYTE / elapsed_seconds / BITS_PER_MEGABIT;

        self.last_sample_at = Instant::now();
        self.last_rfb_updates = rfb.framebuffer_updates;
        self.last_rfb_raw_bytes = rfb.raw_bytes;
        self.presents_since_sample = 0;

        Some(FrameTelemetrySnapshot {
            rfb_updates_per_second,
            present_fps,
            average_render_time_ms: self.average_render_time_ms(),
            rfb_megabits_per_second,
            dropped_frames: rfb.frames_dropped,
        })
    }

    fn average_render_time_ms(&self) -> f64 {
        if self.render_samples.is_empty() {
            return 0.0;
        }
        let total_seconds: f64 = self
            .render_samples
            .iter()
            .map(Duration::as_secs_f64)
            .sum();
        total_seconds * 1000.0 / self.render_samples.len() as f64
    }
}
