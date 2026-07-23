use crate::character::{self, CharacterConfig};
use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Render, Styled, Window, WindowControlArea, div, transparent_black,
};
use gpui_component::{
    TitleBar, button::Button, h_flex, scroll::ScrollableElement, text::markdown, v_flex,
};

pub struct SelectPage {
    characters: Vec<CharacterConfig>,
    focus_handle: FocusHandle,
}

impl SelectPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            characters: character::load_char_list(),
            focus_handle,
        }
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
                            crate::go_to_character(&this.characters[i], window, cx);
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
                    "No characters\n\nPut some in the **characters** folder",
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
