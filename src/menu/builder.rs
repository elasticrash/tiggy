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
    let mut menu = vec![];
    menu.push(MenuItem {
        category: MenuType::DisplayMenu,
        value: KeyCode::Char('m'),
    });
    menu.push(MenuItem {
        category: MenuType::Exit,
        value: KeyCode::Char('x'),
    });
    menu.push(MenuItem {
        category: MenuType::Silent,
        value: KeyCode::Char('s'),
    });
    menu.push(MenuItem {
        category: MenuType::Dial,
        value: KeyCode::Char('d'),
    });
    menu.push(MenuItem {
        category: MenuType::Answer,
        value: KeyCode::Char('a'),
    });

    menu
}
