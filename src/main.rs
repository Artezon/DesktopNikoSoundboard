#![windows_subsystem = "windows"]

mod asset_source;
mod character;
mod character_page;
mod select_page;
mod theme;

use gpui::{
    AppContext, Context, Entity, IntoElement, Render, Subscription, Window,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, px, size,
};
use gpui_component::{Root, Theme, TitleBar, init};

use character_page::{CharacterPage, CharacterPageEvent};
use select_page::{SelectPage, SelectPageEvent};

enum Page {
    Select(Entity<SelectPage>),
    Character(Entity<CharacterPage>),
}

struct App {
    page: Page,
    _window_appearance_subscription: Option<gpui::Subscription>,
    _character_chosen_subscription: Option<Subscription>,
    _go_back_subscription: Option<Subscription>,
}

impl App {
    fn new(cx: &mut Context<Self>) -> Self {
        let select_page = cx.new(|_| SelectPage::new());
        let _character_chosen_subscription =
            Some(cx.subscribe(&select_page, Self::on_character_chosen));

        Self {
            page: Page::Select(select_page),
            _window_appearance_subscription: None,
            _character_chosen_subscription,
            _go_back_subscription: None,
        }
    }

    fn on_character_chosen(
        &mut self,
        select_page: Entity<SelectPage>,
        event: &SelectPageEvent,
        cx: &mut Context<Self>,
    ) {
        let SelectPageEvent::CharacterChosen(index) = event;
        let Some(config) = select_page.read(cx).characters.get(*index).cloned() else {
            return;
        };

        let character_page = cx.new(|_| CharacterPage::load(&config));
        self._go_back_subscription = Some(cx.subscribe(&character_page, Self::on_go_back));
        self.page = Page::Character(character_page);
        cx.notify();
    }

    fn on_go_back(
        &mut self,
        _character_page: Entity<CharacterPage>,
        event: &CharacterPageEvent,
        cx: &mut Context<Self>,
    ) {
        let CharacterPageEvent::GoBack = event;

        let select_page = cx.new(|_| SelectPage::new());
        self._character_chosen_subscription =
            Some(cx.subscribe(&select_page, Self::on_character_chosen));
        self._go_back_subscription = None;
        self.page = Page::Select(select_page);
        cx.notify();
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

        let view = cx.new(|cx| App::new(cx));

        let mut title_opts = TitleBar::title_bar_options();
        title_opts.title = Some("Niko :3".into());

        let options = WindowOptions {
            window_background: WindowBackgroundAppearance::Blurred,
            titlebar: Some(title_opts),
            window_bounds: Some(WindowBounds::centered(size(px(272.), px(360.)), cx)),
            window_min_size: Some(size(px(272.), px(360.))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            view.update(cx, |this, cx| {
                this._window_appearance_subscription =
                    Some(cx.observe_window_appearance(window, |_, window, cx| {
                        Theme::sync_system_appearance(Some(window), &mut *cx);
                        theme::setup_theme(cx);
                    }));
            });
            cx.new(|cx| Root::new(view.clone(), window, cx))
        })
        .unwrap();
    });
}
