use gpui::{App, Hsla, rgba, transparent_black};
use gpui_component::Theme;
use std::sync::LazyLock;

pub struct ThemeColors {
    pub bg: Hsla,
    pub btn_bg: Hsla,
    pub btn_hover: Hsla,
    pub btn_active: Hsla,
}

pub static LIGHT: LazyLock<ThemeColors> = LazyLock::new(|| ThemeColors {
    bg: Hsla::from(rgba(0xffffff80)),
    btn_bg: Hsla::from(rgba(0x11111120)),
    btn_hover: Hsla::from(rgba(0x11111130)),
    btn_active: Hsla::from(rgba(0x11111140)),
});

pub static DARK: LazyLock<ThemeColors> = LazyLock::new(|| ThemeColors {
    bg: Hsla::from(rgba(0x00000060)),
    btn_bg: Hsla::from(rgba(0xffffff20)),
    btn_hover: Hsla::from(rgba(0xffffff40)),
    btn_active: Hsla::from(rgba(0xffffff60)),
});

pub fn current(cx: &App) -> &'static ThemeColors {
    if Theme::global(cx).is_dark() {
        &DARK
    } else {
        &LIGHT
    }
}

pub fn setup_theme(cx: &mut App) {
    let current = current(cx);
    let theme = Theme::global_mut(cx);
    theme.font_family = "Inter".into();
    theme.tokens.background = current.bg.into();
    theme.tokens.button = current.btn_bg.into();
    theme.tokens.button_hover = current.btn_hover.into();
    theme.tokens.button_active = current.btn_active.into();
    theme.input = transparent_black();
    theme.shadow = false;
}
