use std::time::{Duration, Instant};
use std::thread;


pub struct Clock {
    last_tick: Instant,
}

impl Clock {
    pub fn new() -> Clock {
        let now = Instant::now();
        Clock { last_tick: now }
    }

    pub fn tick(&mut self, ticks_per_second: f64) {
        if ticks_per_second.is_infinite() || ticks_per_second < 0.0 {
            panic!("Invalid `ticks_per_second` ({:?}): must be finite and positive", ticks_per_second);
        }

        let interval = Duration::from_secs_f64(ticks_per_second.recip());
        let wakeup_instant = self.last_tick + interval;
        let now = Instant::now();
        if wakeup_instant <= now {
            self.last_tick = now;
            return
        }

        let sleep_duration = wakeup_instant - now;
        thread::sleep(sleep_duration);

        let after_sleep = Instant::now();
        self.last_tick = after_sleep;
    }
}
