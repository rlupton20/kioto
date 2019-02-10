
pub trait EventHandler {
    fn handle_event(&mut self, event: u32) -> ();
}

