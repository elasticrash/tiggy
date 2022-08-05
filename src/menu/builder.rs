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
    pub value: String,
}

pub fn build_menu() -> Vec<MenuItem> {
    let mut menu = vec![];
    menu.push(MenuItem {
        category: MenuType::DisplayMenu,
        value: "m".to_string(),
    });
    menu.push(MenuItem {
        category: MenuType::Exit,
        value: "x".to_string(),
    });
    menu.push(MenuItem {
        category: MenuType::Silent,
        value: "s".to_string(),
    });
    menu.push(MenuItem {
        category: MenuType::Dial,
        value: "d".to_string(),
    });
    menu.push(MenuItem {
        category: MenuType::Answer,
        value: "a".to_string(),
    });

    return menu;
}
