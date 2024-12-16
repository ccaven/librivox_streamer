use rand::{seq::SliceRandom, thread_rng};

pub const STEPS: [i32; 9] = [-4, -3, -2, 1, 0, 1, 2, 3, 4];

pub fn choose_random_step() -> i32 {
    *STEPS.choose(&mut thread_rng()).unwrap()
}