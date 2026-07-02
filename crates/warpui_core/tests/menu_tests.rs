use warpui_core::platform::menu::Menu;

#[test]
fn standard_menu_title_does_not_make_it_window_menu() {
    let menu = Menu::new("Window", Vec::new());

    assert!(!menu.is_window_menu());
}

#[test]
fn window_menu_role_is_independent_of_display_title() {
    let menu = Menu::window("窗口", Vec::new());

    assert!(menu.is_window_menu());
}
