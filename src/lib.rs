use std::{
    fmt,
    time::{Duration, Instant},
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_timer_check_state() {
        let mut timer = Timer::new();
        assert!(timer.pause().is_err());
        assert!(timer.reset().is_err());
        assert!(timer.resume().is_err());
        assert!(timer.start().is_ok());

        assert!(timer.reset().is_err());
        assert!(timer.start().is_err());
        assert!(timer.resume().is_err());
        assert!(timer.pause().is_ok());

        assert!(timer.resume().is_ok());
        assert!(timer.finish().is_ok());
        assert!(timer.reset().is_ok());
    }

    #[test]
    fn test_timer_time() {
        let mut timer = Timer::new();
        timer.start().expect("Illegal!");
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_time().expect("Illegal").as_secs(), 1);
    }
}

#[derive(std::cmp::PartialEq, Debug)]
pub enum TimerState {
    Init,
    Running,
    Paused,
    Finished,
}
pub struct Timer {
    start_time: Option<Instant>,
    timer_state: TimerState,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start_time: None,
            timer_state: TimerState::Init,
        }
    }

    pub fn get_time(&self) -> Result<Duration, TimerError> {
        if self.timer_state == TimerState::Init {
            return Err(TimerError { code: 0x01 });
        }

        Ok(Instant::now().duration_since(self.start_time.expect("None")))
    }

    pub fn start(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Running)?;
        self.start_time = Some(Instant::now());

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Paused)?;

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), TimerError> {
        // If timer is in init state it shouldn't be able to be resumed. However we can't really check that in next_state()
        if self.timer_state == TimerState::Init {
            return Err(TimerError { code: 0x01 });
        }
        self.timer_state = self.next_state(TimerState::Running)?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Init)?;
        self.start_time = None;

        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Finished)?;

        Ok(())
    }

    fn next_state(&self, state: TimerState) -> Result<TimerState, TimerError> {
        let timer_error = TimerError { code: 0x01 };
        match state {
            TimerState::Init => match self.timer_state {
                TimerState::Finished => return Ok(TimerState::Init),
                _ => return Err(timer_error),
            },
            TimerState::Running => match self.timer_state {
                TimerState::Paused | TimerState::Init => {
                    return Ok(TimerState::Running)
                }
                _ => return Err(timer_error),
            },
            TimerState::Paused => match self.timer_state {
                TimerState::Running => return Ok(TimerState::Paused),
                _ => return Err(timer_error),
            },
            TimerState::Finished => match self.timer_state {
                TimerState::Running | TimerState::Paused => {
                    return Ok(TimerState::Finished)
                }
                _ => return Err(timer_error),
            },
        }
    }
}

#[derive(Debug)]
pub struct TimerError {
    code: usize,
}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.code {
            0x01 => "Illegal timer state",
            0x02 => "Invalid subtimer index",
            _ => "Unexpected Error",
        };

        write!(f, "{}", msg)
    }
}
