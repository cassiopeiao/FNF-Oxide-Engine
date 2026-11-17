//! A small musical clock.
//!
//! FNF charts store times in milliseconds, so the conductor exposes a
//! millisecond position and derives beats/steps from BPM. It is deliberately
//! independent from Bevy's `Time` resource: gameplay calls [`Conductor::update`]
//! manually so pausing freezes the song cleanly.

#[derive(Debug, Clone)]
pub struct Conductor {
    bpm: f64,
    position_ms: f64,
    running: bool,
}

impl Conductor {
    pub fn new(bpm: f64) -> Self {
        Self {
            bpm: bpm.max(1.0),
            position_ms: 0.0,
            running: false,
        }
    }

    /// Start the clock at an arbitrary position. A negative position is used
    /// for the pre-song countdown.
    pub fn start_at(&mut self, position_ms: f64) {
        self.position_ms = position_ms;
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Advance by a frame delta. No-op when stopped.
    pub fn update(&mut self, delta_secs: f32) {
        if self.running {
            self.position_ms += f64::from(delta_secs) * 1000.0;
        }
    }

    pub fn position_ms(&self) -> f64 {
        self.position_ms
    }

    pub fn set_position_ms(&mut self, position_ms: f64) {
        self.position_ms = position_ms;
    }

    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    pub fn beat(&self) -> f64 {
        self.position_ms / 60_000.0 * self.bpm
    }

    pub fn step(&self) -> f64 {
        self.beat() * 4.0
    }
}

impl Default for Conductor {
    fn default() -> Self {
        Self::new(100.0)
    }
}
