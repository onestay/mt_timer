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
        assert!(timer.pause_timer().is_err());
        assert!(timer.reset_timer().is_err());
        assert!(timer.resume_timer().is_err());
        assert!(timer.start_timer().is_ok());

        assert!(timer.reset_timer().is_err());
        assert!(timer.start_timer().is_err());
        assert!(timer.resume_timer().is_err());
        assert!(timer.pause_timer().is_ok());

        assert!(timer.resume_timer().is_ok());
        assert!(timer.finish_timer().is_ok());
        assert!(timer.reset_timer().is_ok());
    }

    #[test]
    fn test_timer_time() {
        let mut timer = Timer::new();
        timer.start_timer().expect("Illegal!");
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_time().expect("Illegal").as_secs(), 1);
    }
}

#[derive(std::cmp::PartialEq,Debug)]
pub enum TimerState {
    TimerInit,
    TimerRunning,
    TimerPaused,
    TimerFinished,
}
pub struct Timer {
    start_time: Option<Instant>,
    timer_state: TimerState,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start_time: None,
            timer_state: TimerState::TimerInit,
        }
    }

    pub fn get_time(&self) -> Result<Duration, TimerError> {
        if self.timer_state == TimerState::TimerInit {
            return Err(TimerError { code: 0x01 });
        }

        Ok(Instant::now().duration_since(self.start_time.expect("None")))
    }

    pub fn start_timer(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::TimerRunning)?;
        self.start_time = Some(Instant::now());

        Ok(())
    }

    pub fn pause_timer(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::TimerPaused)?;

        Ok(())
    }

    pub fn resume_timer(&mut self) -> Result<(), TimerError> {
        // If timer is in init state it shouldn't be able to be resumed. However we can't really check that in next_state()
        if self.timer_state == TimerState::TimerInit {
            return Err(TimerError { code: 0x01 });
        }
        self.timer_state = self.next_state(TimerState::TimerRunning)?;

        Ok(())
    }

    pub fn reset_timer(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::TimerInit)?;
        self.start_time = None;

        Ok(())
    }

    pub fn finish_timer(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::TimerFinished)?;
        
        Ok(())
    }

    fn next_state(&self, state: TimerState) -> Result<TimerState, TimerError> {
        let timer_error = TimerError { code: 0x01 };
        match state {
            TimerState::TimerInit => match self.timer_state {
                TimerState::TimerFinished => return Ok(TimerState::TimerInit),
                _ => return Err(timer_error),
            },
            TimerState::TimerRunning => match self.timer_state {
                TimerState::TimerPaused | TimerState::TimerInit => {
                    return Ok(TimerState::TimerRunning)
                }
                _ => return Err(timer_error),
            },
            TimerState::TimerPaused => match self.timer_state {
                TimerState::TimerRunning => return Ok(TimerState::TimerPaused),
                _ => return Err(timer_error),
            },
            TimerState::TimerFinished => match self.timer_state {
                TimerState::TimerRunning | TimerState::TimerPaused => {
                    return Ok(TimerState::TimerFinished)
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
            _ => "Unexpected Error",
        };

        write!(f, "{}", msg)
    }
}
