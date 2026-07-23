#![windows_subsystem = "windows"]

mod asset_source;
mod character;
mod character_page;
mod select_page;
mod theme;

use gpui::{
    AppContext, BorrowAppContext, Context, CursorHideMode, Entity, Global, IntoElement, Render,
    Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, px, size,
};
use gpui_component::{Root, Theme, TitleBar, init};

use character::CharacterConfig;
use character_page::CharacterPage;
use select_page::SelectPage;

enum Page {
    Select(Entity<SelectPage>),
    Character(Entity<CharacterPage>),
}

struct App {
    page: Page,
    _window_appearance_subscription: Option<gpui::Subscription>,
}

impl App {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            page: Page::Select(cx.new(|cx| SelectPage::new(window, cx))),
            _window_appearance_subscription: None,
        }
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.page {
            Page::Select(page) => page.clone().into_any_element(),
            Page::Character(page) => page.clone().into_any_element(),
        }
    }
}

struct AppGlobal {
    app: Entity<App>,
}

impl Global for AppGlobal {}

pub fn go_to_character(config: &CharacterConfig, window: &mut Window, cx: &mut gpui::App) {
    cx.update_global::<AppGlobal, ()>(|global, cx| {
        global.app.update(cx, |this, cx| {
            this.page = Page::Character(cx.new(|cx| CharacterPage::new(config, window, cx)));
            cx.notify();
        });
    });
}

pub fn go_to_select(window: &mut Window, cx: &mut gpui::App) {
    cx.update_global::<AppGlobal, ()>(|global, cx| {
        global.app.update(cx, |this, cx| {
            this.page = Page::Select(cx.new(|cx| SelectPage::new(window, cx)));
            cx.notify();
        });
    });
}

fn main() {
    let app = gpui_platform::application().with_assets(asset_source::Assets);

    app.run(move |cx| {
        init(cx);

        let asset_source = cx.asset_source();
        let fonts = vec![
            asset_source
                .load("fonts/Inter-Regular.ttf")
                .unwrap()
                .unwrap(),
            asset_source
                .load("fonts/Inter-Italic.ttf")
                .unwrap()
                .unwrap(),
        ];
        cx.text_system().add_fonts(fonts).unwrap();

        Theme::sync_system_appearance(None, cx);
        theme::setup_theme(cx);

        let mut title_opts = TitleBar::title_bar_options();
        title_opts.title = Some("Niko :3".into());

        let options = WindowOptions {
            window_background: WindowBackgroundAppearance::Blurred,
            titlebar: Some(title_opts),
            window_bounds: Some(WindowBounds::centered(size(px(272.), px(360.)), cx)),
            window_min_size: Some(size(px(272.), px(360.))),
            ..Default::default()
        };

        cx.set_cursor_hide_mode(CursorHideMode::Never);

        cx.open_window(options, |window, cx| {
            let app = cx.new(|cx| App::new(window, cx));
            cx.set_global(AppGlobal { app: app.clone() });

            app.update(cx, |this, cx| {
                this._window_appearance_subscription =
                    Some(cx.observe_window_appearance(window, |_, window, cx| {
                        Theme::sync_system_appearance(Some(window), &mut *cx);
                        theme::setup_theme(cx);
                    }));
            });
            cx.new(|cx| Root::new(app.clone(), window, cx))
        })
        .unwrap();
    });
}
