//! Element Plus Form and FormItem.
//!
//! Follows `form.md`, `form.vue`, `form.ts`, `form-item.vue`, and Theme Chalk
//! `form.scss`/`form-item.scss`: shared model/rules, label layout, required
//! marks, validation messages, reset, validate, and submit.
//!
//! Native differences: this implementation uses a shared `FormState` handle
//! and synchronous string rules; nested property paths, async validators,
//! automatic field registration/trigger validation, scroll-to-error, and
//! imperative focus/blur exposes are not mapped.

use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{Locale, Theme, el_button, el_icon};

/// String-valued form model.
pub type FormModel = BTreeMap<SharedString, SharedString>;

/// One field validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormError {
    pub prop: SharedString,
    pub message: SharedString,
}

type Validator = Rc<dyn Fn(&str) -> Result<(), String>>;

/// Form validation trigger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FormValidateTrigger {
    #[default]
    Change,
    Blur,
}

/// One synchronous validation rule.
#[derive(Clone)]
pub struct FormRule {
    prop: SharedString,
    required: bool,
    min: Option<usize>,
    max: Option<usize>,
    message: Option<SharedString>,
    validator: Option<Validator>,
    trigger: FormValidateTrigger,
}

impl FormRule {
    pub fn new(prop: impl Into<SharedString>) -> Self {
        Self {
            prop: prop.into(),
            required: false,
            min: None,
            max: None,
            message: None,
            validator: None,
            trigger: FormValidateTrigger::Change,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn validator(mut self, validator: impl Fn(&str) -> Result<(), String> + 'static) -> Self {
        self.validator = Some(Rc::new(validator));
        self
    }

    pub fn trigger(mut self, trigger: FormValidateTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn prop(&self) -> &SharedString {
        &self.prop
    }

    pub fn is_required(&self) -> bool {
        self.required
    }
}

/// Form label layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FormLayout {
    #[default]
    Horizontal,
    Vertical,
    Inline,
}

/// Form label position.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FormLabelPosition {
    Left,
    #[default]
    Right,
    Top,
}

#[derive(Clone)]
struct FormConfig {
    layout: FormLayout,
    label_position: FormLabelPosition,
    label_width: f32,
    label_suffix: SharedString,
    hide_required_asterisk: bool,
    required_mark_left: bool,
    show_message: bool,
    inline_message: bool,
    status_icon: bool,
}

impl Default for FormConfig {
    fn default() -> Self {
        Self {
            layout: FormLayout::Horizontal,
            label_position: FormLabelPosition::Right,
            label_width: 96.,
            label_suffix: String::new().into(),
            hide_required_asterisk: false,
            required_mark_left: true,
            show_message: true,
            inline_message: false,
            status_icon: false,
        }
    }
}

type ValidateHandler = Rc<dyn Fn(&FormModel, bool, &[FormError], &mut Window, &mut App)>;
type SubmitHandler = Rc<dyn Fn(&FormModel, &mut Window, &mut App)>;
type ResetHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone, Default)]
struct FormHandlers {
    onvalidate: Option<ValidateHandler>,
    onsubmit: Option<SubmitHandler>,
    onreset: Option<ResetHandler>,
}

/// Shared handle for a Form and its FormItems.
pub type FormHandle = Rc<RefCell<FormState>>;

/// Shared state for a Form and its FormItems.
#[derive(Default)]
pub struct FormState {
    model: FormModel,
    initial_model: FormModel,
    rules: Vec<FormRule>,
    errors: BTreeMap<SharedString, SharedString>,
    config: FormConfig,
    handlers: FormHandlers,
}

impl FormState {
    pub fn new() -> FormHandle {
        Rc::new(RefCell::new(Self::default()))
    }

    pub fn set_model(&mut self, model: FormModel) {
        self.model = model.clone();
        self.initial_model = model;
        self.errors.clear();
    }

    pub fn set_initial_model(&mut self, model: FormModel) {
        self.initial_model = model;
    }

    pub fn set_field(&mut self, prop: impl Into<SharedString>, value: impl Into<SharedString>) {
        self.model.insert(prop.into(), value.into());
    }

    pub fn set_field_and_validate(
        &mut self,
        prop: impl Into<SharedString>,
        value: impl Into<SharedString>,
        trigger: FormValidateTrigger,
        cx: &mut App,
    ) -> Vec<FormError> {
        let prop = prop.into();
        self.model.insert(prop.clone(), value.into());
        self.validate_triggered(Some(&prop), trigger, cx)
    }

    pub fn field(&self, prop: &SharedString) -> Option<&SharedString> {
        self.model.get(prop)
    }

    pub fn model(&self) -> &FormModel {
        &self.model
    }

    pub fn set_rules(&mut self, rules: impl IntoIterator<Item = FormRule>) {
        self.rules = rules.into_iter().collect();
    }

    pub fn error(&self, prop: &SharedString) -> Option<&SharedString> {
        self.errors.get(prop)
    }

    pub fn clear_validate(&mut self, prop: Option<&SharedString>) {
        if let Some(prop) = prop {
            self.errors.remove(prop);
        } else {
            self.errors.clear();
        }
    }

    pub fn validate(&mut self, prop: Option<&SharedString>, cx: &mut App) -> Vec<FormError> {
        self.validate_with_locale(prop, Locale::current(cx), None)
    }

    pub fn validate_triggered(
        &mut self,
        prop: Option<&SharedString>,
        trigger: FormValidateTrigger,
        cx: &mut App,
    ) -> Vec<FormError> {
        self.validate_with_locale(prop, Locale::current(cx), Some(trigger))
    }

    fn validate_with_locale(
        &mut self,
        prop: Option<&SharedString>,
        locale: Locale,
        trigger: Option<FormValidateTrigger>,
    ) -> Vec<FormError> {
        if let Some(prop) = prop {
            self.errors.remove(prop);
        } else {
            self.errors.clear();
        }
        let mut errors = Vec::new();
        for rule in self
            .rules
            .iter()
            .filter(|rule| prop.is_none_or(|prop| rule.prop == *prop))
            .filter(|rule| trigger.is_none_or(|trigger| rule.trigger == trigger))
        {
            let value = self.model.get(&rule.prop).map_or("", SharedString::as_ref);
            let label = rule.prop.as_ref();
            let message = if rule.required && value.trim().is_empty() {
                rule.message
                    .clone()
                    .unwrap_or_else(|| locale.form_required().replace("{label}", label).into())
            } else if let Some(min) = rule.min
                && value.chars().count() < min
            {
                rule.message.clone().unwrap_or_else(|| {
                    locale
                        .form_min()
                        .replace("{label}", label)
                        .replace("{min}", &min.to_string())
                        .into()
                })
            } else if let Some(max) = rule.max
                && value.chars().count() > max
            {
                rule.message.clone().unwrap_or_else(|| {
                    locale
                        .form_max()
                        .replace("{label}", label)
                        .replace("{max}", &max.to_string())
                        .into()
                })
            } else if let Some(validator) = &rule.validator {
                match validator(value) {
                    Ok(()) => continue,
                    Err(message) => rule.message.clone().unwrap_or_else(|| message.into()),
                }
            } else {
                continue;
            };
            self.errors.insert(rule.prop.clone(), message.clone());
            errors.push(FormError {
                prop: rule.prop.clone(),
                message,
            });
        }
        errors
    }

    pub fn reset(&mut self) {
        self.model = self.initial_model.clone();
        self.errors.clear();
    }

    pub fn submit(&mut self, window: &mut Window, cx: &mut App) -> bool {
        let errors = self.validate(None, cx);
        let valid = errors.is_empty();
        if let Some(handler) = self.handlers.onvalidate.clone() {
            handler(&self.model, valid, &errors, window, cx);
        }
        if valid && let Some(handler) = self.handlers.onsubmit.clone() {
            handler(&self.model, window, cx);
        }
        valid
    }

    pub fn reset_and_notify(&mut self, window: &mut Window, cx: &mut App) {
        self.reset();
        if let Some(handler) = self.handlers.onreset.clone() {
            handler(window, cx);
        }
    }
}

/// An Element Plus form container.
#[derive(IntoElement)]
#[must_use]
pub struct Form {
    id: ElementId,
    state: FormHandle,
    children: Vec<AnyElement>,
    layout: FormLayout,
    label_position: FormLabelPosition,
    label_width: f32,
    label_suffix: SharedString,
    hide_required_asterisk: bool,
    required_mark_left: bool,
    show_message: bool,
    inline_message: bool,
    status_icon: bool,
    disabled: bool,
    show_actions: bool,
    onvalidate: Option<ValidateHandler>,
    onsubmit: Option<SubmitHandler>,
    onreset: Option<ResetHandler>,
}

/// Creates a form around a shared state handle.
#[track_caller]
pub fn el_form(state: FormHandle) -> Form {
    Form::new(state)
}

impl Form {
    pub fn new(state: FormHandle) -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            state,
            children: Vec::new(),
            layout: FormLayout::Horizontal,
            label_position: FormLabelPosition::Right,
            label_width: 96.,
            label_suffix: String::new().into(),
            hide_required_asterisk: false,
            required_mark_left: true,
            show_message: true,
            inline_message: false,
            status_icon: false,
            disabled: false,
            show_actions: false,
            onvalidate: None,
            onsubmit: None,
            onreset: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn layout(mut self, layout: FormLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn inline(self) -> Self {
        self.layout(FormLayout::Inline)
    }

    pub fn vertical(self) -> Self {
        self.layout(FormLayout::Vertical)
    }

    pub fn label_position(mut self, position: FormLabelPosition) -> Self {
        self.label_position = position;
        self
    }

    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    pub fn label_suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.label_suffix = suffix.into();
        self
    }

    pub fn hide_required_asterisk(mut self, hide: bool) -> Self {
        self.hide_required_asterisk = hide;
        self
    }

    pub fn required_mark_left(mut self, left: bool) -> Self {
        self.required_mark_left = left;
        self
    }

    pub fn show_message(mut self, show: bool) -> Self {
        self.show_message = show;
        self
    }

    pub fn inline_message(mut self, inline: bool) -> Self {
        self.inline_message = inline;
        self
    }

    pub fn status_icon(mut self, show: bool) -> Self {
        self.status_icon = show;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_actions(mut self, show: bool) -> Self {
        self.show_actions = show;
        self
    }

    pub fn onvalidate(
        mut self,
        handler: impl Fn(&FormModel, bool, &[FormError], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onvalidate = Some(Rc::new(handler));
        self
    }

    pub fn onsubmit(
        mut self,
        handler: impl Fn(&FormModel, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onsubmit = Some(Rc::new(handler));
        self
    }

    pub fn onreset(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onreset = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Form {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Form {
            id,
            state,
            children,
            layout,
            label_position,
            label_width,
            label_suffix,
            hide_required_asterisk,
            required_mark_left,
            show_message,
            inline_message,
            status_icon,
            disabled,
            show_actions,
            onvalidate,
            onsubmit,
            onreset,
        } = self;
        state.borrow_mut().config = FormConfig {
            layout,
            label_position,
            label_width,
            label_suffix,
            hide_required_asterisk,
            required_mark_left,
            show_message,
            inline_message,
            status_icon,
        };
        state.borrow_mut().handlers = FormHandlers {
            onvalidate,
            onsubmit,
            onreset,
        };
        let locale = Locale::current(cx);
        let mut root = div()
            .id(id.clone())
            .flex()
            .when(layout == FormLayout::Inline, |el| {
                el.flex_wrap().items_center().gap(px(24.))
            })
            .when(layout != FormLayout::Inline, |el| {
                el.flex_col().gap(px(18.))
            })
            .children(children);
        if show_actions {
            root = root.child(
                div().flex().items_center().gap(px(12.)).children([
                    el_button()
                        .id(ElementId::NamedChild(Arc::new(id.clone()), "submit".into()))
                        .primary()
                        .label(locale.form_submit())
                        .disabled(disabled)
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.borrow_mut().submit(window, cx);
                                window.refresh();
                            }
                        }),
                    el_button()
                        .id(ElementId::NamedChild(Arc::new(id.clone()), "reset".into()))
                        .label(locale.form_reset())
                        .disabled(disabled)
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.borrow_mut().reset_and_notify(window, cx);
                                window.refresh();
                            }
                        }),
                ]),
            );
        }
        root.into_any_element()
    }
}

/// An Element Plus form field row.
#[derive(IntoElement)]
#[must_use]
pub struct FormItem {
    id: ElementId,
    state: FormHandle,
    prop: SharedString,
    label: Option<SharedString>,
    content: AnyElement,
    required: Option<bool>,
    error: Option<SharedString>,
    label_width: Option<f32>,
    label_position: Option<FormLabelPosition>,
    show_message: Option<bool>,
    inline_message: Option<bool>,
    status_icon: Option<bool>,
}

/// Creates a form item bound to a shared state and model key.
#[track_caller]
pub fn el_form_item(state: FormHandle, prop: impl Into<SharedString>) -> FormItem {
    FormItem::new(state, prop)
}

impl FormItem {
    pub fn new(state: FormHandle, prop: impl Into<SharedString>) -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            state,
            prop: prop.into(),
            label: None,
            content: div().into_any_element(),
            required: None,
            error: None,
            label_width: None,
            label_position: None,
            show_message: None,
            inline_message: None,
            status_icon: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.content = child.into_any_element();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width);
        self
    }

    pub fn label_position(mut self, position: FormLabelPosition) -> Self {
        self.label_position = Some(position);
        self
    }

    pub fn show_message(mut self, show: bool) -> Self {
        self.show_message = Some(show);
        self
    }

    pub fn inline_message(mut self, inline: bool) -> Self {
        self.inline_message = Some(inline);
        self
    }

    pub fn status_icon(mut self, show: bool) -> Self {
        self.status_icon = Some(show);
        self
    }
}

impl RenderOnce for FormItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let FormItem {
            id,
            state,
            prop,
            label,
            content,
            required,
            error,
            label_width,
            label_position,
            show_message,
            inline_message,
            status_icon,
        } = self;
        let config = state.borrow().config.clone();
        let error = error.or_else(|| state.borrow().error(&prop).cloned());
        let required = required.unwrap_or_else(|| {
            state
                .borrow()
                .rules
                .iter()
                .any(|rule| rule.prop == prop && rule.required)
        });
        let label_position = label_position.unwrap_or(config.label_position);
        let horizontal =
            config.layout != FormLayout::Vertical && label_position != FormLabelPosition::Top;
        let label_width = label_width.unwrap_or(config.label_width);
        let show_message = show_message.unwrap_or(config.show_message);
        let inline_message = inline_message.unwrap_or(config.inline_message);
        let inline_error = error.clone().filter(|_| show_message && inline_message);
        let status_icon = status_icon.unwrap_or(config.status_icon);
        let theme = Theme::current(cx);
        let mut label_row = div().flex().items_center().gap(px(4.));
        if required && !config.hide_required_asterisk && config.required_mark_left {
            label_row = label_row.child(div().text_color(rgb(crate::theme::DANGER)).child("*"));
        }
        if let Some(label) = &label {
            label_row = label_row.child(label.clone());
        }
        if !config.label_suffix.is_empty() {
            label_row = label_row.child(config.label_suffix.clone());
        }
        if required && !config.hide_required_asterisk && !config.required_mark_left {
            label_row = label_row.child(div().text_color(rgb(crate::theme::DANGER)).child("*"));
        }
        let label_element = div()
            .when(horizontal, |el| el.w(px(label_width)).flex_shrink_0())
            .when(label_position == FormLabelPosition::Right, |el| {
                el.justify_end()
            })
            .text_size(px(14.))
            .text_color(rgb(theme.regular_text()))
            .child(label_row);
        let mut content_column = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .min_w_0()
            .flex_1()
            .child(content);
        if let Some(error) = error.filter(|_| show_message && !inline_message) {
            content_column = content_column.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_size(px(12.))
                    .text_color(rgb(crate::theme::DANGER))
                    .when(status_icon, |el| {
                        el.child(el_icon("circle-close").size(12.))
                    })
                    .child(error),
            );
        } else if status_icon {
            content_column = content_column.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_size(px(12.))
                    .text_color(rgb(crate::theme::SUCCESS))
                    .child(el_icon("circle-check").size(12.)),
            );
        }
        let mut item = div().id(id).w_full();
        if let Some(error) = inline_error {
            item = item
                .flex()
                .items_center()
                .gap(px(12.))
                .child(label_element)
                .child(content_column)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .text_size(px(12.))
                        .text_color(rgb(crate::theme::DANGER))
                        .when(status_icon, |el| {
                            el.child(el_icon("circle-close").size(12.))
                        })
                        .child(error),
                );
        } else if horizontal {
            item = item
                .flex()
                .items_start()
                .gap(px(12.))
                .child(label_element)
                .child(content_column);
        } else {
            item = item
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(label_element)
                .child(content_column);
        }
        item.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_reports_required_length_and_custom_errors() {
        let state = FormState::new();
        state.borrow_mut().set_model(BTreeMap::from([
            ("name".into(), "ab".into()),
            ("email".into(), "bad".into()),
        ]));
        state.borrow_mut().set_rules([
            FormRule::new("name").required(true).min(3),
            FormRule::new("email")
                .required(true)
                .validator(|value| {
                    if value.contains('@') {
                        Ok(())
                    } else {
                        Err("Invalid email".into())
                    }
                })
                .message("Email is invalid"),
        ]);
        let errors = state
            .borrow_mut()
            .validate_with_locale(None, Locale::default(), None);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].prop, "name");
        assert_eq!(errors[1].message, "Email is invalid");
        state.borrow_mut().set_field("name", "abcd");
        state.borrow_mut().set_field("email", "a@b");
        assert!(
            state
                .borrow_mut()
                .validate_with_locale(None, Locale::default(), None)
                .is_empty()
        );
    }

    #[test]
    fn triggered_validation_updates_only_the_matching_field() {
        let state = FormState::new();
        state.borrow_mut().set_model(BTreeMap::from([
            ("name".into(), "".into()),
            ("email".into(), "".into()),
        ]));
        state.borrow_mut().set_rules([
            FormRule::new("name")
                .required(true)
                .trigger(FormValidateTrigger::Change),
            FormRule::new("email")
                .required(true)
                .trigger(FormValidateTrigger::Blur),
        ]);
        state.borrow_mut().validate_with_locale(
            Some(&"name".into()),
            Locale::default(),
            Some(FormValidateTrigger::Change),
        );
        assert!(state.borrow().error(&"name".into()).is_some());
        state.borrow_mut().validate_with_locale(
            Some(&"email".into()),
            Locale::default(),
            Some(FormValidateTrigger::Blur),
        );
        assert!(state.borrow().error(&"name".into()).is_some());
        assert!(state.borrow().error(&"email".into()).is_some());
        state.borrow_mut().set_field("name", "valid");
        state.borrow_mut().validate_with_locale(
            Some(&"name".into()),
            Locale::default(),
            Some(FormValidateTrigger::Change),
        );
        assert!(state.borrow().error(&"name".into()).is_none());
        assert!(state.borrow().error(&"email".into()).is_some());
    }

    #[test]
    fn reset_restores_initial_model_and_clears_errors() {
        let state = FormState::new();
        state
            .borrow_mut()
            .set_model(BTreeMap::from([("name".into(), "initial".into())]));
        state
            .borrow_mut()
            .set_rules([FormRule::new("name").required(true)]);
        state.borrow_mut().set_field("name", "");
        assert_eq!(
            state
                .borrow_mut()
                .validate_with_locale(None, Locale::default(), None)
                .len(),
            1
        );
        state.borrow_mut().reset();
        assert_eq!(
            state.borrow().field(&"name".into()),
            Some(&"initial".into())
        );
        assert!(state.borrow().errors.is_empty());
    }
}
