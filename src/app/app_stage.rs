pub enum AppStageStatus<R> {
    Continue,
    Complete(R)
}

#[derive(Copy, Clone)]
pub enum AppStage {
    MainMenu,
    Game,
    OldGame,
    Editor,
}