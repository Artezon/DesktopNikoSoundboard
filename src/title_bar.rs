use gpui::{
    App, InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    Styled, Window, WindowControlArea, div, prelude::FluentBuilder, transparent_black,
};
use gpui_component::{TitleBar as GpuiComponentTitleBar, h_flex};

struct TitleBarState {
    should_move: bool,
}

#[derive(IntoElement)]
pub struct TitleBar {
    label: SharedString,
    children: Vec<gpui::AnyElement>,
}

impl TitleBar {
    pub fn new() -> Self {
        Self {
            label: SharedString::default(),
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_state(cx, |_, _| TitleBarState { should_move: false });

        h_flex()
            .w_full()
            .items_center()
            .when(self.children.is_empty(), |this| this.pl_1p5())
            .children(self.children)
            .child(
                div()
                    .flex_1()
                    .mx_1()
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = true;
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = false;
                        }),
                    )
                    .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                        if state.should_move {
                            state.should_move = false;
                            window.start_window_move();
                        }
                    }))
                    .text_ellipsis()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(self.label),
            )
            .child(
                GpuiComponentTitleBar::new()
                    .bg(transparent_black())
                    .border_b_0()
                    .p_0(),
            )
    }
}
