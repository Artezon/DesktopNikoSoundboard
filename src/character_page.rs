use crate::character::{CharDataSource, CharacterConfig};
use crate::pixel_img::pixel_img;
use crate::title_bar::TitleBar;
use anyhow::{Context as _, Error};
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render,
    RenderImage, Styled, Window, div, img, prelude::FluentBuilder,
};
use gpui_component::{
    Icon, IconName,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use image::{Frame, load_from_memory};
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
    idle_image: Option<Arc<RenderImage>>,
    speaking_image: Option<Arc<RenderImage>>,
    idle_pixel_art: bool,
    speaking_pixel_art: bool,
    sounds: Vec<StaticSoundData>,
    sound_weights: Vec<f64>,
    pitch_min: f64,
    pitch_max: f64,
    active_sounds: Vec<StaticSoundHandle>,
    audio_manager: Option<AudioManager>,
    focus_handle: Option<FocusHandle>,
}

fn render_image_from_bytes(bytes: &[u8]) -> Arc<RenderImage> {
    let img = load_from_memory(bytes).unwrap();
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

        let result = (|| -> Result<CharacterPage, Error> {
            let manager = AudioManager::new(AudioManagerSettings::default())?;

            let mut source = CharDataSource::new(&config.path).context("cannot read character")?;

            let idle_bytes = source.read_content(&config.images.idle.file)?;
            let idle_pixel_art = config.images.idle.pixel_art;

            let speaking_bytes = source.read_content(&config.images.speaking.file)?;
            let speaking_pixel_art = config.images.speaking.pixel_art;

            let mut sounds = Vec::new();
            for sound_file in &config.sounds {
                let bytes = source.read_content(sound_file)?;
                let data = StaticSoundData::from_cursor(std::io::Cursor::new(bytes))?;
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

            Ok(CharacterPage {
                load_success: true,
                name: config.name.clone(),
                action_text: config.action_text.clone(),
                idle_image: Some(render_image_from_bytes(&idle_bytes)),
                speaking_image: Some(render_image_from_bytes(&speaking_bytes)),
                idle_pixel_art,
                speaking_pixel_art,
                sounds,
                sound_weights,
                pitch_min,
                pitch_max,
                active_sounds: Vec::new(),
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.active_sounds
            .retain(|h| h.state() == PlaybackState::Playing);
        let is_playing = !self.active_sounds.is_empty();

        let top_bar = TitleBar::new().label(self.name.clone()).child(
            Button::new("back")
                .icon(IconName::ArrowLeft)
                .rounded_none()
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.go_back(window, cx);
                })),
        );

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
                let image = if is_playing {
                    self.speaking_image.clone().unwrap()
                } else {
                    self.idle_image.clone().unwrap()
                };

                let is_pixel_art = if is_playing {
                    self.speaking_pixel_art
                } else {
                    self.idle_pixel_art
                };

                this.child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .p_4()
                        .gap_4()
                        .w_full()
                        .child(if is_pixel_art {
                            pixel_img(image).size_full().into_any_element()
                        } else {
                            img(image).size_full().into_any_element()
                        })
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
