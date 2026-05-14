pub enum MainMenuData {
    Home {
        heading: String,
        buttons: Vec<MainMenuButton>,
        hovered_button: isize,
    },
    ChapterSelect {

    },
    SaveSelect,
    Settings,
}

pub struct MainMenuButton {
    pub text: String,
}