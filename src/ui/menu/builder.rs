use crossterm::event::KeyCode;

#[derive(Clone, Debug)]
pub enum MenuType {
    DisplayMenu,
    Exit,
    Silent,
    Dial,
    Answer,
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub category: MenuType,
    pub value: KeyCode,
}

pub fn build_menu() -> Vec<MenuItem> {
    let menu = vec![
        MenuItem {
            category: MenuType::DisplayMenu,
            value: KeyCode::Char('m'),
        },
        MenuItem {
            category: MenuType::Exit,
            value: KeyCode::Char('x'),
        },
        MenuItem {
            category: MenuType::Silent,
            value: KeyCode::Char('s'),
        },
        MenuItem {
            category: MenuType::Dial,
            value: KeyCode::Char('d'),
        },
        MenuItem {
            category: MenuType::Answer,
            value: KeyCode::Char('a'),
        },
    ];
    menu
}
