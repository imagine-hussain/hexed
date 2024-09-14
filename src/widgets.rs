use std::time::Instant;

pub struct FrameCounter {
    last_frame: Instant,
    tick_number: u64,
    framerate: u32,
}

impl FrameCounter {
    const FRAMERATE_UPDATE_INTERVAL: u64 = 10;

    pub fn new() -> Self {
        FrameCounter {
            last_frame: Instant::now(),
            tick_number: 0,
            framerate: 0,
        }
    }

    pub fn register_tick(&mut self) {
        self.tick_number += 1;
        if self.tick_number % Self::FRAMERATE_UPDATE_INTERVAL == 0 {
            self.update_framerate();
            self.update_delta_time();
        }
    }

    fn update_framerate(&mut self) -> u32 {
        let delta_time = self.last_frame.elapsed().as_millis();
        self.framerate = match delta_time == 0 {
            true => 99,
            false => ((Self::FRAMERATE_UPDATE_INTERVAL as u128 * 1_000) / delta_time) as u32,
        };
        self.framerate
    }

    fn update_delta_time(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn fps(&self) -> u32 {
        self.framerate
    }
}
