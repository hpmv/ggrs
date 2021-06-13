use crate::frame_info::BLANK_INPUT;
use crate::GameInput;

const FRAME_WINDOW_SIZE: usize = 30;
const MIN_UNIQUE_FRAMES: usize = 10;
const MIN_FRAME_ADVANTAGE: i32 = 3;
const MAX_FRAME_ADVANTAGE: i32 = 30;

#[derive(Debug)]
pub(crate) struct TimeSync {
    local: [i32; FRAME_WINDOW_SIZE],
    remote: [i32; FRAME_WINDOW_SIZE],
    last_inputs: [GameInput; MIN_UNIQUE_FRAMES],
}

impl TimeSync {
    pub(crate) fn new() -> Self {
        Self {
            local: [0; FRAME_WINDOW_SIZE],
            remote: [0; FRAME_WINDOW_SIZE],
            last_inputs: [BLANK_INPUT; MIN_UNIQUE_FRAMES],
        }
    }

    pub(crate) fn advance_frame(&mut self, input: GameInput, local_adv: i32, remote_adv: i32) {
        self.last_inputs[input.frame as usize % self.last_inputs.len()] = input;
        self.local[input.frame as usize % self.local.len()] = local_adv;
        self.remote[input.frame as usize % self.remote.len()] = remote_adv;
    }

    pub(crate) fn recommend_frame_delay(&self, require_idle_input: bool) -> u32 {
        // average local and remote frame advantages
        let local_sum: i32 = self.local.iter().sum();
        let local_avg = local_sum as f32 / self.local.len() as f32;
        let remote_sum: i32 = self.remote.iter().sum();
        let remote_avg = remote_sum as f32 / self.remote.len() as f32;

        // if we have the advantage, we are behind and don't need to wait.
        if local_avg >= remote_avg {
            return 0;
        }

        // meet in the middle
        let sleep_frames = (((remote_avg - local_avg) / 2.0f32) + 0.5) as i32;

        // only wait if the discrepancy is big enough
        if sleep_frames < MIN_FRAME_ADVANTAGE {
            return 0;
        }

        // if required, check if all past inputs are identical
        if require_idle_input {
            let ref_input = self.last_inputs[0];
            if !self
                .last_inputs
                .iter()
                .all(|input| input.equal(&ref_input, true))
            {
                return 0;
            }
        }

        // never recommend beyond maximum wait
        std::cmp::min(sleep_frames, MAX_FRAME_ADVANTAGE) as u32
    }
}
