use crate::character::{CharacterConfig, app_dir, load_char_list};
use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Render, Styled, Task, Window, WindowControlArea, div, transparent_black,
};
use gpui_component::{
    TitleBar, button::Button, h_flex, scroll::ScrollableElement, text::markdown, v_flex,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub struct SelectPage {
    characters: Vec<CharacterConfig>,
    focus_handle: FocusHandle,
    fs_watcher: Option<RecommendedWatcher>,
    _fs_watch_task: Option<Task<()>>,
}

impl SelectPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        let mut this = Self {
            characters: load_char_list(),
            focus_handle,
            fs_watcher: None,
            _fs_watch_task: None,
        };

        this.start_watching(cx);
        this
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        let Some(root) = app_dir() else { return };
        let filter_root = root.clone();

        let (tx, rx) = std::sync::mpsc::channel::<notify::Event>();

        let Ok(mut watcher) =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                let Ok(event) = res else { return };

                let is_relevant = event.paths.iter().any(|p| {
                    p.strip_prefix(&filter_root)
                        .is_ok_and(|rel| rel.starts_with("characters"))
                });

                if is_relevant {
                    let _ = tx.send(event);
                }
            })
        else {
            return;
        };

        if watcher.watch(&root, RecursiveMode::Recursive).is_err() {
            return;
        }

        self.fs_watcher = Some(watcher);

        let task = cx.spawn(async move |this, cx| {
            let mut rx = rx;
            loop {
                let (event, returned_rx) = cx
                    .background_executor()
                    .spawn(async move {
                        let event = rx.recv().ok();
                        (event, rx)
                    })
                    .await;

                rx = returned_rx;
                let Some(event) = event else { break };
                if this
                    .update(cx, |this, cx| this.reload_characters(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        });

        self._fs_watch_task = Some(task);
    }

    fn reload_characters(&mut self, _event: notify::Event, cx: &mut Context<Self>) {
        self.characters = load_char_list();
        cx.notify();
    }
}

impl Render for SelectPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let top_bar = h_flex()
            .w_full()
            .items_center()
            .pl_1p5()
            .child(
                div()
                    .flex_1()
                    .mx_1()
                    .window_control_area(WindowControlArea::Drag)
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child("Select character"),
            )
            .child(TitleBar::new().bg(transparent_black()).border_b_0().p_0());

        let buttons: Vec<_> = self
            .characters
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let name = c.name.clone();
                let path = c.path.clone();
                Button::new(format!("char_{}", i))
                    .w_full()
                    .max_w_128()
                    .child(
                        div()
                            .w_full()
                            .text_ellipsis()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_center()
                            .child(name),
                    )
                    .on_click(cx.listener(
                        move |this: &mut SelectPage,
                              _: &ClickEvent,
                              window: &mut Window,
                              cx: &mut Context<SelectPage>| {
                            if let Some(config) = this.characters.iter().find(|c| c.path == path) {
                                crate::go_to_character(config, window, cx);
                            }
                        },
                    ))
            })
            .collect();

        let body = if self.characters.is_empty() {
            v_flex()
                .flex_1()
                .size_full()
                .items_center()
                .justify_center()
                .text_center()
                .p_4()
                .child(markdown(
                    if let Some(dir) = app_dir()
                        && let Some(dir) = dir.join("characters").to_str()
                    {
                        format!("No characters\n\nPut some in this folder:\n**{dir}**")
                    } else {
                        "No characters".to_string()
                    },
                ))
                .into_any_element()
        } else {
            v_flex()
                .flex_1()
                .min_h_0()
                .p_4()
                .gap_2()
                .size_full()
                .items_center()
                .justify_center()
                .children(buttons)
                .overflow_y_scrollbar()
                .into_any_element()
        };

        v_flex()
            .id("select-page")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, _window, cx| {
                if event.is_held {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "escape" => cx.quit(),
                    _ => {}
                }
            }))
            .size_full()
            .child(top_bar)
            .child(body)
    }
}
