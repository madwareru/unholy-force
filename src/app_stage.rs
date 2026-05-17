pub enum AppStageStatus<R> {
    Continue,
    Complete(R)
}

pub trait AppStage {
    type R;
    fn process(&mut self) -> AppStageStatus<Self::R>;
    fn render(&mut self);
}