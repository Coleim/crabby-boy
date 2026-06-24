use crate::crabby_boy::CrabbyBoy;

pub trait Display {
    fn draw(&mut self, emulator: &CrabbyBoy);
    fn handle_events(&mut self);
    fn is_running(&self) -> bool;
}
