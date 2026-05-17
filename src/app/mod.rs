use crate::{
    app::{
        app_stage::{AppStage, AppStageStatus},
        main_menu_stage::MainMenuCommand
    }
};

pub mod app_stage;
mod main_menu_stage;

pub struct App {
    app_stage: AppStage,
    main_menu_stage : main_menu_stage::MainMenuStage,
}

impl App {
    pub fn new() -> Self {
        let main_menu_stage = main_menu_stage::MainMenuStage::new();
        let app_stage = AppStage::MainMenu;
        Self {
            app_stage,
            main_menu_stage,
        }
    }

    pub fn render(&self) {
        match self.app_stage {
            AppStage::MainMenu => {
                self.main_menu_stage.render();
            },
            AppStage::Game => todo!(),
            AppStage::OldGame => todo!(),
            AppStage::Editor => todo!()
        }
    }

    pub fn process(&mut self) -> AppStageStatus<()> {
        match self.app_stage {
            AppStage::MainMenu => {
                match self.process_main_menu() {
                    Some(result_stage) => {
                        self.app_stage = result_stage;
                    }
                    None => { return AppStageStatus::Complete(()); }
                }
            }
            _ => todo!()
        }
        AppStageStatus::Continue
    }

    fn process_main_menu(&mut self) -> Option<AppStage> {
        match self.main_menu_stage.process() {
            AppStageStatus::Continue => {},
            AppStageStatus::Complete(command) => {
                match command {
                    MainMenuCommand::OpenOldGame => {
                        return Some(AppStage::OldGame);
                    }
                    MainMenuCommand::StartNewGame => {
                        return Some(AppStage::Game);
                    }
                    MainMenuCommand::OpenEditor => {
                        return Some(AppStage::Editor);
                    }
                    MainMenuCommand::Exit => {
                        return None;
                    }
                    MainMenuCommand::VisitGithub => {
                        webbrowser::open("https://github.com/madwareru/unholy-force")
                            .unwrap();
                    }
                    MainMenuCommand::VisitGamedev => {
                        webbrowser::open("https://gamedev.ru/users/?id=41788")
                            .unwrap();
                    }
                    MainMenuCommand::VisitTelegram => {
                        webbrowser::open("https://t.me/obscure_computer_science")
                            .unwrap();
                    }
                    MainMenuCommand::VisitVK => {
                        webbrowser::open("https://vk.com/madware")
                            .unwrap();
                    }
                    MainMenuCommand::VisitMastodon => {
                        webbrowser::open("https://mastodon.gamedev.place/@madware")
                            .unwrap();
                    }
                    MainMenuCommand::LeaveFeedback => {
                        webbrowser::open("https://github.com/madwareru/unholy-force/issues/new/choose")
                            .unwrap();
                    }
                    MainMenuCommand::Donate => {
                        // todo
                    }
                }
            }
        }
        Some(self.app_stage)
    }
}