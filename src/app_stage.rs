pub enum AppStageStatus<R> {
    Continue,
    Complete(R)
}

pub trait AppStageLogic {
    type R;
    fn process(&mut self) -> AppStageStatus<Self::R>;
    fn render(&mut self);
}

pub enum AppStage {
    MainMenu,
    Game,
    OldGame,
    Editor,
}