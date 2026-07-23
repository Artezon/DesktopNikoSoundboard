use crate::character::CharacterConfig;
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render,
    RenderImage, Styled, Window, WindowControlArea, div, img, prelude::FluentBuilder,
    transparent_black,
};
use gpui_component::{
    Icon, IconName, TitleBar,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use image::{Frame, imageops::FilterType};
use kira::sound::PlaybackState;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, PlaybackRate};
use rand::RngExt;
use std::sync::Arc;

#[derive(Default)]
pub struct CharacterPage {
    load_success: bool,
    name: String,
    action_text: Option<String>,
    idle_img_bytes: Vec<u8>,
    speaking_img_bytes: Vec<u8>,
    idle_pixel_art: bool,
    speaking_pixel_art: bool,
    sounds: Vec<StaticSoundData>,
    sound_weights: Vec<f64>,
    pitch_min: f64,
    pitch_max: f64,
    active_sounds: Vec<StaticSoundHandle>,
    cached_size: Option<(u32, u32)>,
    cached_idle: Option<Arc<RenderImage>>,
    cached_speaking: Option<Arc<RenderImage>>,
    audio_manager: Option<AudioManager>,
    focus_handle: Option<FocusHandle>,
}

fn render_image(bytes: &[u8], pixel_art: bool, max_w: u32, max_h: u32) -> Arc<RenderImage> {
    let img = image::load_from_memory(bytes).unwrap();
    let filter = if pixel_art {
        FilterType::Nearest
    } else {
        FilterType::Lanczos3
    };
    let img = img.resize(max_w, max_h, filter);
    let mut data = img.into_rgba8();
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Arc::new(RenderImage::new([Frame::new(data)]))
}

impl CharacterPage {
    pub fn new(config: &CharacterConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        let result = (|| -> Result<CharacterPage, anyhow::Error> {
            let manager = AudioManager::new(AudioManagerSettings::default())?;

            let idle_path = config.dir.join(&config.images.idle.file);
            let idle_bytes = std::fs::read(&idle_path)?;
            let idle_pixel_art = config.images.idle.pixel_art;

            let speaking_path = config.dir.join(&config.images.speaking.file);
            let speaking_bytes = std::fs::read(&speaking_path)?;
            let speaking_pixel_art = config.images.speaking.pixel_art;

            let mut sounds = Vec::new();
            for sound_file in &config.sounds {
                let sound_path = config.dir.join(sound_file);
                let data = StaticSoundData::from_file(sound_path)?;
                sounds.push(data);
            }

            let sound_weights: Vec<f64> = config
                .sound_random_weights
                .clone()
                .unwrap_or_else(|| vec![1.0; config.sounds.len()]);

            let (pitch_min, pitch_max) = match &config.pitch {
                Some(p) if p.len() >= 2 => (p[0], p[1]),
                Some(p) if p.len() == 1 => (p[0], p[0]),
                _ => (1.0, 1.0),
            };
            let (pitch_min, pitch_max) = (pitch_min.min(pitch_max), pitch_max.max(pitch_min));

            image::load_from_memory(&idle_bytes)?;
            image::load_from_memory(&speaking_bytes)?;

            Ok(CharacterPage {
                load_success: true,
                name: config.name.clone(),
                action_text: config.action_text.clone(),
                idle_img_bytes: idle_bytes,
                speaking_img_bytes: speaking_bytes,
                idle_pixel_art,
                speaking_pixel_art,
                sounds,
                sound_weights,
                pitch_min,
                pitch_max,
                active_sounds: Vec::new(),
                cached_size: None,
                cached_idle: None,
                cached_speaking: None,
                audio_manager: Some(manager),
                focus_handle: Some(focus_handle.clone()),
            })
        })();

        match result {
            Ok(state) => state,
            Err(_) => CharacterPage {
                name: config.name.clone(),
                focus_handle: Some(focus_handle),
                ..Default::default()
            },
        }
    }

    fn choose_sound(&self) -> (usize, f64) {
        let mut rng = rand::rng();

        let index = if self.sound_weights.len() > 1 {
            let total = self.sound_weights.iter().sum();
            if total > 0.0 {
                let roll = rng.random_range(0.0..total);
                let mut cumulative = 0.0;
                let mut chosen = 0;
                for (i, &w) in self.sound_weights.iter().enumerate() {
                    cumulative += w;
                    if roll < cumulative {
                        chosen = i;
                        break;
                    }
                }
                chosen
            } else {
                0
            }
        } else {
            0
        };

        let pitch = if self.pitch_max - self.pitch_min < f64::EPSILON {
            self.pitch_min
        } else {
            rng.random_range(self.pitch_min..self.pitch_max)
        };

        (index, pitch)
    }

    fn play_sound(&mut self, cx: &mut Context<Self>) {
        if self.sounds.is_empty() {
            return;
        }

        let (index, pitch) = self.choose_sound();

        let Some(manager) = &mut self.audio_manager else {
            return;
        };

        let settings = StaticSoundSettings::new().playback_rate(PlaybackRate(pitch));
        let sound = self.sounds[index].clone().with_settings(settings);

        if let Ok(handle) = manager.play(sound) {
            self.active_sounds.push(handle);
            cx.notify();

            let duration = self.sounds.get(index).map_or_else(
                || std::time::Duration::from_secs(0),
                |d| d.duration().div_f64(pitch),
            );

            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(duration).await;
                this.update(cx, |_, cx| cx.notify()).ok();
            })
            .detach();
        }
    }

    fn go_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for handle in &mut self.active_sounds {
            handle.stop(Default::default());
        }
        crate::go_to_select(window, cx);
    }
}

impl Render for CharacterPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.active_sounds
            .retain(|h| h.state() == PlaybackState::Playing);
        let is_playing = !self.active_sounds.is_empty();

        let top_bar = h_flex()
            .w_full()
            .items_center()
            .child(
                Button::new("back")
                    .icon(IconName::ArrowLeft)
                    .rounded_none()
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.go_back(window, cx);
                    })),
            )
            .child(
                div()
                    .flex_1()
                    .mx_1()
                    .window_control_area(WindowControlArea::Drag)
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(self.name.clone()),
            )
            .child(TitleBar::new().bg(transparent_black()).border_b_0().p_0());

        v_flex()
            .id("character-page")
            .track_focus(&self.focus_handle.as_ref().unwrap())
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.is_held {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "space" => this.play_sound(cx),
                    "escape" => this.go_back(window, cx),
                    _ => {}
                }
            }))
            .size_full()
            .child(top_bar)
            .when(self.load_success, |this| {
                let window_size = window.viewport_size();
                let w = window_size.width.as_f32() as u32;
                let h = window_size.height.as_f32() as u32;

                if self.cached_size != Some((w, h)) {
                    self.cached_idle = Some(render_image(
                        &self.idle_img_bytes,
                        self.idle_pixel_art,
                        w,
                        h,
                    ));
                    self.cached_speaking = Some(render_image(
                        &self.speaking_img_bytes,
                        self.speaking_pixel_art,
                        w,
                        h,
                    ));
                    self.cached_size = Some((w, h));
                }

                let image = if is_playing {
                    self.cached_speaking.clone().unwrap()
                } else {
                    self.cached_idle.clone().unwrap()
                };

                this.child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .p_4()
                        .gap_4()
                        .w_full()
                        .child(img(image).size_full())
                        .child(h_flex().justify_center().child({
                            let action_text = self
                                .action_text
                                .as_ref()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());

                            Button::new("sound")
                                .p_3()
                                .h_10()
                                .min_w_10()
                                .max_w_full()
                                .child(
                                    div()
                                        .text_ellipsis()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .when_none(&action_text, |this| {
                                            this.child(Icon::default().path("icons/sound.svg"))
                                        })
                                        .when_some(action_text, |this, text| this.child(text)),
                                )
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.play_sound(cx);
                                }))
                        })),
                )
            })
            .when(!self.load_success, |this| {
                this.child(
                    v_flex()
                        .flex_1()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .child("Character load failed"),
                )
            })
    }
}
