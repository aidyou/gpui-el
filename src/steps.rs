//! Element Plus Steps.
//!
//! Follows `steps.md`, `steps.vue`, `item.vue`, and Theme Chalk
//! `steps.scss` / `step.scss`: responsive or fixed-width steps, horizontal and
//! vertical directions, centered content, simple mode, status-derived colors,
//! custom icons, descriptions, and connector progress.

use std::{rc::Rc, sync::Arc};

use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::{
    Icon, Theme, el_icon,
    theme::{DANGER, PRIMARY, SUCCESS},
};

/// Step completion state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepStatus {
    Wait,
    Process,
    Finish,
    Error,
    Success,
}

impl StepStatus {
    fn color(self, theme: &Theme) -> u32 {
        match self {
            Self::Wait => theme.disabled_text(),
            Self::Process => theme.primary_text(),
            Self::Finish => PRIMARY,
            Self::Error => DANGER,
            Self::Success => SUCCESS,
        }
    }

    fn fills_connector(self) -> bool {
        !matches!(self, Self::Wait | Self::Process)
    }
}

/// Steps layout direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StepsDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// One step.
#[derive(Clone)]
pub struct StepItem {
    title: SharedString,
    description: Option<SharedString>,
    icon: Option<Icon>,
    status: Option<StepStatus>,
}

impl StepItem {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
            status: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = Some(status);
        self
    }
}

type Handler = Rc<dyn Fn(usize, usize, &mut App)>;

/// An Element Plus step bar.
#[derive(IntoElement)]
#[must_use]
pub struct Steps {
    id: ElementId,
    items: Vec<StepItem>,
    active: usize,
    direction: StepsDirection,
    align_center: bool,
    simple: bool,
    space: Option<f32>,
    process_status: StepStatus,
    finish_status: StepStatus,
    onchange: Option<Handler>,
}

/// Creates an empty step bar and adds steps with [`Steps::step`].
#[track_caller]
pub fn el_steps() -> Steps {
    Steps::new()
}

impl Default for Steps {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            items: Vec::new(),
            active: 0,
            direction: StepsDirection::Horizontal,
            align_center: false,
            simple: false,
            space: None,
            process_status: StepStatus::Process,
            finish_status: StepStatus::Finish,
            onchange: None,
        }
    }
}

impl Steps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn step(mut self, step: StepItem) -> Self {
        self.items.push(step);
        self
    }

    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    pub fn direction(mut self, direction: StepsDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn align_center(mut self, align: bool) -> Self {
        self.align_center = align;
        self
    }

    pub fn simple(mut self, simple: bool) -> Self {
        self.simple = simple;
        self
    }

    pub fn space(mut self, space: f32) -> Self {
        self.space = Some(space.max(0.));
        self
    }

    pub fn process_status(mut self, status: StepStatus) -> Self {
        self.process_status = status;
        self
    }

    pub fn finish_status(mut self, status: StepStatus) -> Self {
        self.finish_status = status;
        self
    }

    pub fn onchange(mut self, handler: impl Fn(usize, usize, &mut App) + 'static) -> Self {
        self.onchange = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
struct StepsState {
    previous_active: Option<usize>,
}

fn status_for(
    index: usize,
    active: usize,
    item: &StepItem,
    process_status: StepStatus,
    finish_status: StepStatus,
) -> StepStatus {
    item.status.unwrap_or({
        if index < active {
            finish_status
        } else if index == active {
            process_status
        } else {
            StepStatus::Wait
        }
    })
}

impl RenderOnce for Steps {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| StepsState::default());
        if let Some(previous) = state.read(cx).previous_active
            && previous != self.active
            && let Some(handler) = self.onchange.clone()
        {
            handler(self.active, previous, cx);
        }
        state.update(cx, |state, _| state.previous_active = Some(self.active));
        let theme = Theme::current(cx);
        let statuses: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                status_for(
                    index,
                    self.active,
                    item,
                    self.process_status,
                    self.finish_status,
                )
            })
            .collect();
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "steps".into())
            .flex()
            .when(self.simple, |el| {
                el.items_center()
                    .p(px(13.))
                    .px(px(8.))
                    .rounded(px(4.))
                    .bg(rgb(theme.fill(crate::theme::Fill::Light)))
            })
            .when(
                !self.simple && self.direction == StepsDirection::Vertical,
                |el| el.flex_col(),
            );
        for (index, item) in self.items.iter().cloned().enumerate() {
            let Some(&status) = statuses.get(index) else {
                continue;
            };
            let color = status.color(&theme);
            let last = index + 1 == self.items.len();
            if self.simple {
                let mut step = div()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        format!("step-{index}").into(),
                    ))
                    .debug_selector(move || format!("step-{index}"))
                    .flex()
                    .items_center()
                    .when(!last, |el| el.flex_1())
                    .child(
                        div()
                            .w(px(16.))
                            .h(px(16.))
                            .mr(px(10.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(color))
                            .child(if let Some(icon) = item.icon.clone() {
                                icon.size(18.).into_any_element()
                            } else if status == StepStatus::Success {
                                el_icon("check").size(14.).into_any_element()
                            } else if status == StepStatus::Error {
                                el_icon("close").size(14.).into_any_element()
                            } else {
                                div().into_any_element()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(16.))
                            .line_height(px(20.))
                            .when(status == StepStatus::Process, |el| {
                                el.font_weight(gpui::FontWeight::BOLD)
                            })
                            .text_color(rgb(color))
                            .child(item.title),
                    );
                if !last {
                    step = step.child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px(px(12.))
                            .child(
                                div().flex().items_center().justify_center().child(
                                    div()
                                        .text_color(rgb(theme.disabled_text()))
                                        .child(el_icon("arrow-right").size(14.)),
                                ),
                            ),
                    );
                }
                root = root.child(step);
                continue;
            }
            let vertical = self.direction == StepsDirection::Vertical;
            let custom_icon = item.icon.is_some();
            let icon_size = if custom_icon && !vertical { 40. } else { 24. };
            let connector_fill = statuses
                .get(index + 1)
                .is_some_and(|status| status.fills_connector());
            let mut step = div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    format!("step-{index}").into(),
                ))
                .debug_selector(move || format!("step-{index}"))
                .relative()
                .when(self.space.is_some(), |el| {
                    if vertical {
                        el.h(px(self.space.unwrap_or(0.)))
                    } else {
                        el.w(px(self.space.unwrap_or(0.)))
                    }
                })
                .when(self.space.is_none() && !vertical && !last, |el| el.flex_1())
                .when(vertical, |el| el.flex().items_start());
            if !last {
                let line = div()
                    .absolute()
                    .bg(rgb(theme.disabled_text()))
                    .when(vertical, |el| {
                        el.left(px(icon_size / 2. - 1.))
                            .top(px(0.))
                            .bottom(px(0.))
                            .w(px(2.))
                            .flex()
                            .flex_col()
                    })
                    .when(!vertical, |el| {
                        el.top(px(icon_size / 2. - 1.))
                            .left(px(0.))
                            .right(px(0.))
                            .h(px(2.))
                            .flex()
                    })
                    .child(
                        div()
                            .bg(rgb(color))
                            .when(vertical, |el| {
                                if connector_fill {
                                    el.h_full()
                                } else {
                                    el.h(px(0.))
                                }
                            })
                            .when(!vertical, |el| {
                                if connector_fill {
                                    el.w_full()
                                } else {
                                    el.w(px(0.))
                                }
                            }),
                    );
                step = step.child(line);
            }
            let icon = div()
                .relative()
                .w(px(icon_size))
                .h(px(icon_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(icon_size / 2.))
                .border_2()
                .border_color(rgb(color))
                .bg(rgb(theme.background()))
                .text_color(rgb(color))
                .child(if let Some(icon) = item.icon.clone() {
                    icon.size(24.).into_any_element()
                } else if status == StepStatus::Success {
                    el_icon("check").size(14.).into_any_element()
                } else if status == StepStatus::Error {
                    el_icon("close").size(14.).into_any_element()
                } else {
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child((index + 1).to_string())
                        .into_any_element()
                });
            let mut main = div()
                .when(vertical, |el| el.pl(px(10.)).flex_1())
                .when(self.align_center && !vertical, |el| {
                    el.text_center().px(px(20.))
                })
                .child(
                    div()
                        .text_size(px(16.))
                        .line_height(if vertical { px(24.) } else { px(38.) })
                        .when(status == StepStatus::Process, |el| {
                            el.font_weight(gpui::FontWeight::BOLD)
                        })
                        .text_color(rgb(color))
                        .child(item.title),
                );
            if let Some(description) = item.description {
                main = main.child(
                    div()
                        .mt(px(-5.))
                        .text_size(px(12.))
                        .line_height(px(20.))
                        .text_color(rgb(color))
                        .child(description),
                );
            }
            step = step.child(div().when(vertical, |el| el.flex()).child(icon).child(main));
            root = root.child(step);
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_follow_active_process_and_finish_rules() {
        let items = [
            StepItem::new("One"),
            StepItem::new("Two"),
            StepItem::new("Three"),
        ];
        let statuses: Vec<_> = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                status_for(index, 1, item, StepStatus::Process, StepStatus::Finish)
            })
            .collect();
        assert_eq!(
            statuses,
            vec![StepStatus::Finish, StepStatus::Process, StepStatus::Wait]
        );
        assert!(StepStatus::Finish.fills_connector());
        assert!(!StepStatus::Process.fills_connector());
    }

    #[gpui::test]
    fn vertical_steps_render_titles_and_descriptions(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().p(px(20.)).child(
                    el_steps()
                        .id("steps")
                        .active(1)
                        .direction(StepsDirection::Vertical)
                        .step(StepItem::new("Account").description("Create account"))
                        .step(StepItem::new("Profile").description("Fill profile"))
                        .step(StepItem::new("Done").description("Review")),
                )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("step-0").is_some());
        assert!(view.debug_bounds("step-1").is_some());
        assert!(view.debug_bounds("step-2").is_some());
    }
}
