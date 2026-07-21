#![windows_subsystem = "windows"]

use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::Result;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::*;
use image::{Frame, imageops::FilterType};
use kira::sound::PlaybackState;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, PlaybackRate};
use rand::RngExt;

const BG_DARK: u32 = 0x00000040;
const BG_LIGHT: u32 = 0xffffff40;

const NIKO_PNG: &[u8] = include_bytes!("../assets/niko.png");
const NIKO_PANCAKES_PNG: &[u8] = include_bytes!("../assets/niko_pancakes.png");
const YIPPEE_MP3: &[u8] = include_bytes!("../assets/yippee.mp3");

fn render_image(bytes: &[u8], max_w: u32, max_h: u32) -> Arc<RenderImage> {
    let img = image::load_from_memory(bytes).unwrap();
    let img = img.resize(max_w, max_h, FilterType::Nearest);
    let mut data = img.into_rgba8();
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Arc::new(RenderImage::new([Frame::new(data)]))
}

type SoundHandle = kira::sound::static_sound::StaticSoundHandle;

struct NikoApp {
    audio_manager: AudioManager,
    sound_data: StaticSoundData,
    active_sounds: Vec<SoundHandle>,
    playing_count: Arc<AtomicU32>,
    cached_size: Option<(u32, u32)>,
    cached_niko: Option<Arc<RenderImage>>,
    cached_pancakes: Option<Arc<RenderImage>>,
    _subscription: Option<gpui::Subscription>,
}

impl NikoApp {
    fn new() -> Result<Self> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        let cursor = std::io::Cursor::new(YIPPEE_MP3.to_vec());
        let sound_data = StaticSoundData::from_cursor(cursor)?;

        Ok(Self {
            audio_manager: manager,
            sound_data,
            active_sounds: Vec::new(),
            playing_count: Arc::new(AtomicU32::new(0)),
            cached_size: None,
            cached_niko: None,
            cached_pancakes: None,
            _subscription: None,
        })
    }

    fn play_yippee(&mut self) -> f64 {
        let mut rng = rand::rng();
        let pitch: f64 = rng.random_range(0.95..1.05);
        let settings = StaticSoundSettings::new().playback_rate(PlaybackRate(pitch));
        let sound = self.sound_data.clone().with_settings(settings);
        if let Ok(handle) = self.audio_manager.play(sound) {
            self.playing_count.fetch_add(1, Ordering::SeqCst);
            self.active_sounds.push(handle);
        }
        pitch
    }

    fn is_sound_playing(&self) -> bool {
        self.playing_count.load(Ordering::SeqCst) > 0
    }

    fn on_yippee_click(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let pitch = self.play_yippee();
        cx.notify();
        let actual_duration =
            Duration::from_secs_f64(self.sound_data.duration().as_secs_f64() / pitch);
        let counter = self.playing_count.clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(actual_duration + Duration::from_millis(50))
                .await;
            counter.fetch_sub(1, Ordering::SeqCst);
            this.update(cx, |_, cx| cx.notify()).ok();
        })
        .detach();
    }

    fn ensure_images_for_size(&mut self, w: u32, h: u32) {
        if self.cached_size == Some((w, h)) {
            return;
        }
        self.cached_niko = Some(render_image(NIKO_PNG, w, h));
        self.cached_pancakes = Some(render_image(NIKO_PANCAKES_PNG, w, h));
        self.cached_size = Some((w, h));
    }
}

impl Render for NikoApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.active_sounds
            .retain(|h| h.state() == PlaybackState::Playing);
        let is_playing = self.is_sound_playing();
        let window_size = window.viewport_size();
        let (w, h) = (
            window_size.width.as_f32() as u32,
            window_size.height.as_f32() as u32,
        );

        self.ensure_images_for_size(w, h);
        let image = if is_playing {
            self.cached_pancakes.clone().unwrap()
        } else {
            self.cached_niko.clone().unwrap()
        };

        v_flex()
            .size_full()
            .child(TitleBar::new().bg(rgba(0x00000000)).border_b_0())
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .p_4()
                    .gap_4()
                    .w_full()
                    .child(img(image).size_full())
                    .child(h_flex().justify_center().child({
                        let bg = if cx.theme().is_dark() {
                            Hsla::from(rgba(0xcccccc40))
                        } else {
                            Hsla::from(rgba(0x33333340))
                        };
                        Button::new("yippee")
                            .label(":3")
                            .px_3()
                            .py_5()
                            .border_0()
                            .bg(bg)
                            .on_click(cx.listener(Self::on_yippee_click))
                    })),
            )
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        init(cx);
        Theme::sync_system_appearance(None, cx);
        {
            let bg = if Theme::global(cx).is_dark() {
                BG_DARK
            } else {
                BG_LIGHT
            };
            Theme::global_mut(cx).tokens.background = ThemeToken::from(Hsla::from(rgba(bg)));
        }

        let fonts = vec![
            Cow::Borrowed(&include_bytes!("../assets/fonts/Inter-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/Inter-Italic.ttf")[..]),
        ];
        cx.text_system().add_fonts(fonts).unwrap();
        Theme::global_mut(cx).font_family = "Inter".into();

        cx.spawn(async move |cx| {
            let niko_app = NikoApp::new().unwrap();
            let view = cx.new(|_| niko_app);

            let mut title_opts = TitleBar::title_bar_options();
            title_opts.title = Some("Niko :3".into());

            let options = cx.update(|app| WindowOptions {
                window_background: WindowBackgroundAppearance::Blurred,
                titlebar: Some(title_opts),
                window_bounds: Some(WindowBounds::centered(size(px(272.), px(360.)), app)),
                window_min_size: Some(size(px(272.), px(360.))),
                ..Default::default()
            });

            cx.open_window(options, |window, cx| {
                view.update(cx, |this, cx| {
                    this._subscription =
                        Some(cx.observe_window_appearance(window, |_, window, cx| {
                            Theme::sync_system_appearance(Some(window), &mut *cx);
                            let bg = if Theme::global(cx).is_dark() {
                                BG_DARK
                            } else {
                                BG_LIGHT
                            };
                            Theme::global_mut(cx).tokens.background =
                                ThemeToken::from(Hsla::from(rgba(bg)));
                        }));
                });
                cx.new(|cx| Root::new(view.clone(), window, cx))
            })
            .unwrap();
        })
        .detach();
    });
}
