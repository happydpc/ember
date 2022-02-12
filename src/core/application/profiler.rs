use std::collections::VecDeque;

pub struct AppProfile{
    pub render_times: VecDeque<u128>,
    pub update_times: VecDeque<u128>,
    pub render_avgs: VecDeque<f32>,
    pub update_avgs: VecDeque<f32>,
    pub window_size: u32,
}