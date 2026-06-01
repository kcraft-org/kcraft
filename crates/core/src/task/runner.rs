pub trait TaskRunner: Send {
    fn execute(&mut self) -> Result<(), String>;
    fn abort(&mut self) -> bool {
        false
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn status(&self) -> &str {
        ""
    }
    fn progress(&self) -> (i64, i64) {
        (0, 100)
    }
    fn is_multistep(&self) -> bool {
        false
    }
    fn step_progress(&self) -> (i64, i64) {
        (0, 100)
    }
    fn step_status(&self) -> String {
        String::new()
    }
    fn warnings(&self) -> Vec<String> {
        Vec::new()
    }
    fn fail_reason(&self) -> String {
        String::new()
    }
}
