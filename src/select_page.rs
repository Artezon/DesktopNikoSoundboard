use crate::character::{self, CharacterConfig};
use gpui::{
    ClickEvent, Context, EventEmitter, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowControlArea, div, transparent_black,
};
use gpui_component::{
    TitleBar, button::Button, h_flex, scroll::ScrollableElement, text::markdown, v_flex,
};

pub enum SelectPageEvent {
    CharacterChosen(usize),
}

pub struct SelectPage {
    pub(crate) characters: Vec<CharacterConfig>,
}

impl SelectPage {
    pub fn new() -> Self {
        Self {
            characters: character::load_char_list(),
        }
    }
}

impl EventEmitter<SelectPageEvent> for SelectPage {}

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

        if self.characters.is_empty() {
            return v_flex().size_full().child(top_bar).child(
                v_flex()
                    .flex_1()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .text_center()
                    .p_4()
                    .child(markdown(
                        "No characters\n\nPut some in the **characters** folder",
                    )),
            );
        }

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
                        move |_this: &mut SelectPage,
                              _: &ClickEvent,
                              _: &mut Window,
                              cx: &mut Context<SelectPage>| {
                            cx.emit(SelectPageEvent::CharacterChosen(i));
                        },
                    ))
            })
            .collect();

        let list = v_flex()
            .flex_1()
            .min_h_0()
            .p_4()
            .gap_2()
            .size_full()
            .items_center()
            .justify_center()
            .children(buttons)
            .overflow_y_scrollbar();

        v_flex().size_full().child(top_bar).child(list)
    }
}
