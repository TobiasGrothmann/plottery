//! The native menu bar; its items reach the app as [`MenuAction`]s, see [`use_menu_action`].

use dioxus::desktop::{
    muda::{
        accelerator::{Accelerator, Code, Modifiers},
        AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu,
    },
    use_muda_event_handler,
};

/// What a menu item of the app asks for; the item's menu id is the variant's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Run,
    PlotSettings,
    BackToProjects,
    ToggleParams,
    ToggleLayers,
    ToggleConsole,
    ToggleFullscreenPreview,
    ResetZoom,
    PlotterRemote,
}

impl MenuAction {
    const ALL: [MenuAction; 9] = [
        MenuAction::Run,
        MenuAction::PlotSettings,
        MenuAction::BackToProjects,
        MenuAction::ToggleParams,
        MenuAction::ToggleLayers,
        MenuAction::ToggleConsole,
        MenuAction::ToggleFullscreenPreview,
        MenuAction::ResetZoom,
        MenuAction::PlotterRemote,
    ];

    fn id(self) -> String {
        format!("{self:?}")
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }

    /// Single-key shortcuts (F, R) stay with the page, so they don't fire while typing.
    fn item(self, text: &str, accelerator: Option<Accelerator>) -> MenuItem {
        MenuItem::with_id(self.id(), text, true, accelerator)
    }
}

fn cmd(code: Code) -> Option<Accelerator> {
    Some(Accelerator::new(Some(Modifiers::SUPER), code))
}

pub fn menu_bar() -> Menu {
    let app_menu = Submenu::with_items(
        "Plottery Editor",
        true,
        &[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("Plottery Editor".to_string()),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ],
    )
    .expect("Failed to build the app menu");

    let edit_menu = Submenu::with_items(
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ],
    )
    .expect("Failed to build the edit menu");

    let project_menu = Submenu::with_items(
        "Project",
        true,
        &[
            &MenuAction::Run.item("Run", cmd(Code::KeyR)),
            &MenuAction::PlotSettings.item("Plot Settings…", cmd(Code::Comma)),
            &PredefinedMenuItem::separator(),
            &MenuAction::BackToProjects.item("Back to Projects", cmd(Code::BracketLeft)),
        ],
    )
    .expect("Failed to build the project menu");

    let view_menu = Submenu::with_items(
        "View",
        true,
        &[
            &MenuAction::ToggleParams.item("Toggle Parameters", cmd(Code::Digit1)),
            &MenuAction::ToggleLayers.item("Toggle Layers", cmd(Code::Digit2)),
            &MenuAction::ToggleConsole.item("Toggle Console", cmd(Code::Digit3)),
            &PredefinedMenuItem::separator(),
            &MenuAction::ToggleFullscreenPreview.item("Fullscreen Preview (F)", None),
            &MenuAction::ResetZoom.item("Reset Zoom", cmd(Code::Digit0)),
        ],
    )
    .expect("Failed to build the view menu");

    let plotter_menu = Submenu::with_items(
        "Plotter",
        true,
        &[&MenuAction::PlotterRemote.item("Remote (R)", None)],
    )
    .expect("Failed to build the plotter menu");

    let window_menu = Submenu::with_items(
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(None),
        ],
    )
    .expect("Failed to build the window menu");
    window_menu.set_as_windows_menu_for_nsapp();

    let menu = Menu::with_items(&[
        &app_menu,
        &edit_menu,
        &project_menu,
        &view_menu,
        &plotter_menu,
        &window_menu,
    ])
    .expect("Failed to build the menu bar");

    // dioxus handles these ids itself
    if cfg!(debug_assertions) {
        let help_menu = Submenu::with_items(
            "Help",
            true,
            &[
                &MenuItem::with_id(
                    "dioxus-toggle-dev-tools",
                    "Toggle Developer Tools",
                    true,
                    None,
                ),
                &MenuItem::with_id("dioxus-float-top", "Float on Top", true, None),
            ],
        )
        .expect("Failed to build the help menu");
        help_menu.set_as_help_menu_for_nsapp();
        menu.append(&help_menu)
            .expect("Failed to add the help menu");
    }
    menu
}

/// Calls `handler` for each menu item chosen while the calling component is mounted.
pub fn use_menu_action(mut handler: impl FnMut(MenuAction) + 'static) {
    use_muda_event_handler(move |event| {
        if let Some(action) = MenuAction::from_id(&event.id().0) {
            handler(action);
        }
    });
}
