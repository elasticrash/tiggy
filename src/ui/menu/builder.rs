use crossterm::event::KeyCode;
use tui::{
    style::Style,
    text::{Span, Spans},
};

#[derive(Clone, Debug)]
pub enum MenuType {
    DisplayMenu,
    Exit,
    Silent,
    Dial,
    Answer,
    Unregister,
    Quiet,
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
            category: MenuType::Unregister,
            value: KeyCode::Char('u'),
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
        MenuItem {
            category: MenuType::Quiet,
            value: KeyCode::Char('q'),
        },
    ];
    menu
}

/// Menu printed on the UI
pub fn print_menu() -> Vec<Spans<'static>> {
    vec![
        { Spans::from(Span::styled("s. Toggle Silent mode", Style::default())) },
        {
            Spans::from(Span::styled(
                "d. Dial Number & (enter to sumbit)",
                Style::default(),
            ))
        },
        { Spans::from(Span::styled("   or (esc to cancel)", Style::default())) },
        { Spans::from(Span::styled("x. Exit", Style::default())) },
    ]
}
