use std::{
    time::{Duration, Instant},
};

use thiserror::Error;

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
        timer.pause().expect("Illegal!");
        std::thread::sleep(Duration::from_secs(2));
        timer.resume().expect("Illegal");
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_time().expect("illegal").as_secs(), 2);
    }

    #[test]
    fn test_subtimer() {
        let mut timer = Timer::new();
        let a = timer.add_subtimer().expect("Error");
        let b = timer.add_subtimer().expect("Error");

        timer.start().expect("Illegal");
        assert!(timer.add_subtimer().is_err());
        std::thread::sleep(Duration::from_secs(1));
        let a = timer.finish_subtimer(a).expect("Error");
        assert_eq!(a.time.expect("Should contain time").as_secs(), 1);
        assert_eq!(timer.get_subtimer(4).is_err(), true);
        timer.finish().expect("Illegal");
        std::thread::sleep(Duration::from_secs(1));
        let b = timer.get_subtimer(b).expect("Error");
        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(b.time.expect("Should contain time").as_secs(), 1);

        timer.reset().expect("Illegal");
        assert_eq!(timer.get_subtimer(0).is_err(), true);

        let a = timer.add_subtimer().expect("Illegal");
        assert_eq!(timer.finish_subtimer(a).is_err(), true);
    }
}

#[derive(std::cmp::PartialEq, Debug, Clone, Copy)]
pub enum TimerState {
    Init,
    Running,
    Paused,
    Finished,
}

pub struct Timer {
    start_time: Option<Instant>,
    timer_state: TimerState,
    last_paused: Option<Instant>,
    sub_timers: Vec<SubTimer>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start_time: None,
            timer_state: TimerState::Init,
            last_paused: None,
            sub_timers: vec![],
        }
    }

    pub fn get_time(&self) -> Result<Duration, TimerError> {
        if self.timer_state == TimerState::Init {
            return Err(TimerError::Unsupported{operation: "get time".to_string(), state: self.timer_state});
        }
        if let Some(start_time) = self.start_time {
            Ok(Instant::now().duration_since(start_time))
        } else {
            Err(TimerError::NoneUnexpected)
        }
    }

    pub fn start(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Running)?;
        self.start_time = Some(Instant::now());

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Paused)?;
        self.last_paused = Some(Instant::now());
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), TimerError> {
        // If timer is in init state it shouldn't be able to be resumed. However we can't really check that in next_state()
        if self.timer_state == TimerState::Init {
            return Err(TimerError::IllegalStateTransition{current: self.timer_state, next: TimerState::Running});
        }
        self.timer_state = self.next_state(TimerState::Running)?;
        let start_time = match self.start_time {
            Some(start_time) => start_time,
            None => return Err(TimerError::NoneUnexpected),
        };

        let last_paused = match self.last_paused {
            Some(start_time) => start_time,
            None => return Err(TimerError::NoneUnexpected),
        };

        let time_diff = start_time + Instant::now().duration_since(last_paused);
        self.start_time = Some(time_diff);

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Init)?;
        self.start_time = None;
        self.last_paused = None;

        self.sub_timers.clear();

        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), TimerError> {
        self.timer_state = self.next_state(TimerState::Finished)?;
        let time = self.get_time()?;

        for sub_timer in &mut self.sub_timers {
            sub_timer.finished = true;
            sub_timer.time = Some(time)
        }

        Ok(())
    }

    fn next_state(&self, state: TimerState) -> Result<TimerState, TimerError> {
        let timer_error = TimerError::IllegalStateTransition{current: self.timer_state, next: state};
        match state {
            TimerState::Init => match self.timer_state {
                TimerState::Finished => return Ok(TimerState::Init),
                _ => return Err(timer_error),
            },
            TimerState::Running => match self.timer_state {
                TimerState::Paused | TimerState::Init => return Ok(TimerState::Running),
                _ => return Err(timer_error),
            },
            TimerState::Paused => match self.timer_state {
                TimerState::Running => return Ok(TimerState::Paused),
                _ => return Err(timer_error),
            },
            TimerState::Finished => match self.timer_state {
                TimerState::Running | TimerState::Paused => return Ok(TimerState::Finished),
                _ => return Err(timer_error),
            },
        }
    }

    pub fn add_subtimer(&mut self) -> Result<usize, TimerError> {
        if self.timer_state != TimerState::Init {
            return Err(TimerError::Unsupported{state: self.timer_state, operation: "add subtimer".to_string()});
        }
        let sub_timer = SubTimer {
            time: None,
            finished: false,
        };

        self.sub_timers.push(sub_timer);

        return Ok(self.sub_timers.len() - 1);
    }

    pub fn finish_subtimer(&mut self, index: usize) -> Result<&SubTimer, TimerError> {
        if self.timer_state != TimerState::Running {
            return Err(TimerError::IllegalStateTransition{current:self.timer_state, next: TimerState::Finished});
        }

        self.check_subtimer_index(index)?;

        let time = self.get_time()?;

        let sub_timer = &mut self.sub_timers[index];
        if sub_timer.finished {
            return Err(TimerError::Unsupported{operation: "finish subtimer".to_string(), state: TimerState::Finished});
        }
        sub_timer.time = Some(time);
        sub_timer.finished = true;

        let mut done = true;
        for sub_timer in &self.sub_timers {
            if !sub_timer.finished {
                done = false;
                break;
            }
        }

        if done {
            self.finish()?;
        }

        Ok(&self.sub_timers[index])
    }

    pub fn delete_subtimer(&mut self, index: usize) -> Result<(), TimerError> {
        if self.timer_state != TimerState::Init {
            return Err(TimerError::Unsupported{operation: "delete subtimer".to_string(), state: self.timer_state});
        }
        self.check_subtimer_index(index)?;

        self.sub_timers.remove(index);
        Ok(())
    }

    pub fn get_subtimer(&mut self, index: usize) -> Result<&SubTimer, TimerError> {
        self.check_subtimer_index(index)?;

        Ok(&self.sub_timers[index])
    }

    fn check_subtimer_index(&self, index: usize) -> Result<(), TimerError> {
        if self.sub_timers.len() == 0 || index > self.sub_timers.len() {
            return Err(TimerError::InvalidSubtimerIndex(index));
        }

        Ok(())
    }

    pub fn resume_subtimer(&mut self, index: usize) -> Result<(), TimerError> {
        if self.timer_state == TimerState::Finished {
            self.resume()?;
        } else if self.timer_state != TimerState::Running {
            return Err(TimerError::Unsupported{state: self.timer_state, operation: "resume subtimer".to_string()});
        }

        self.check_subtimer_index(index)?;

        let subtimer = &mut self.sub_timers[index];
        subtimer.finished = false;
        subtimer.time = None;

        Ok(())
    }
}
#[derive(Debug, Error)]
pub enum TimerError {
    #[error("Illegal timer transition attempted. Can't go from state {current:?} to {next:?}")]
    IllegalStateTransition {
        current: TimerState,
        next: TimerState
    },
    #[error("Operation {operation:?} not supported when timer in state {state:?}")]
    Unsupported {
        state: TimerState,
        operation: String
    },
    #[error("Invalid subtimer index `{0}`")]
    InvalidSubtimerIndex(usize),
    #[error("'None' not expected")]
    NoneUnexpected,
    #[error("Subtimer not finished")]
    SubTimerNotFinished,
    #[error("An unexpected error occured")]
    Unexpected
}

#[derive(Debug, PartialEq)]
pub struct SubTimer {
    time: Option<Duration>,
    finished: bool,
}

impl SubTimer {
    pub fn get_time(&self) -> Result<Duration, TimerError> {
        if !self.finished {
            return Err(TimerError::SubTimerNotFinished);
        }

        let time = match self.time {
            Some(time) => time,
            None => return Err(TimerError::NoneUnexpected),
        };

        Ok(time)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
