use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{
    FormHandle, FormLabelPosition, FormRule, el_button, el_form, el_form_item, el_icon, el_input,
};

use crate::{Gallery, copy::Copy};

#[derive(Default)]
pub(crate) struct State {
    form: FormHandle,
    submitted: bool,
}

impl Gallery {
    pub(crate) fn forms(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let form = self.form.form.clone();
        if form.borrow().model().is_empty() {
            let mut form = form.borrow_mut();
            form.set_model(
                [
                    ("name".into(), "".into()),
                    ("email".into(), "".into()),
                    ("city".into(), "Shanghai".into()),
                ]
                .into(),
            );
            form.set_rules([
                FormRule::new("name").required(true).min(2).max(12),
                FormRule::new("email")
                    .required(true)
                    .validator(|value| {
                        if value.contains('@') {
                            Ok(())
                        } else {
                            Err("Invalid email".into())
                        }
                    })
                    .message("Please enter a valid email"),
                FormRule::new("city").required(true),
            ]);
        }
        let field = |prop: &'static str, id: &'static str, width: f32, disabled: bool| {
            let form = form.clone();
            let view = view.clone();
            let value = form
                .borrow()
                .field(&SharedString::from(prop))
                .cloned()
                .unwrap_or_default();
            div()
                .w(px(width))
                .child(el_input().id(id).value(value).disabled(disabled).oninput(
                    move |value, _, cx| {
                        form.borrow_mut().set_field(prop, value);
                        view.update(cx, |this, cx| {
                            this.form.submitted = false;
                            cx.notify();
                        });
                    },
                ))
        };
        let horizontal = el_form(form.clone())
            .id("form-horizontal")
            .label_width(72.)
            .status_icon(true)
            .show_actions(true)
            .onsubmit(cx.listener(|this, _, _, cx| {
                this.form.submitted = true;
                cx.notify();
            }))
            .child(
                el_form_item(form.clone(), "name")
                    .id("form-name")
                    .label(s.form_name.clone())
                    .child(field("name", "form-name-input", 280., false)),
            )
            .child(
                el_form_item(form.clone(), "email")
                    .id("form-email")
                    .label(s.form_email.clone())
                    .child(field("email", "form-email-input", 280., false)),
            )
            .child(
                el_form_item(form.clone(), "city")
                    .id("form-city")
                    .label(s.form_city.clone())
                    .child(field("city", "form-city-input", 280., false)),
            );
        let vertical = el_form(form.clone())
            .id("form-vertical")
            .vertical()
            .label_position(FormLabelPosition::Top)
            .show_actions(true)
            .child(
                el_form_item(form.clone(), "name")
                    .id("form-vertical-name")
                    .label(s.form_name.clone())
                    .child(field("name", "form-vertical-name-input", 280., false)),
            )
            .child(
                el_form_item(form.clone(), "email")
                    .id("form-vertical-email")
                    .label(s.form_email.clone())
                    .child(field("email", "form-vertical-email-input", 280., false)),
            );
        let inline = el_form(form.clone())
            .id("form-inline")
            .inline()
            .label_width(56.)
            .show_actions(true)
            .child(
                el_form_item(form.clone(), "name")
                    .id("form-inline-name")
                    .label(s.form_name.clone())
                    .child(field("name", "form-inline-name-input", 180., false)),
            )
            .child(
                el_form_item(form.clone(), "city")
                    .id("form-inline-city")
                    .label(s.form_city.clone())
                    .child(field("city", "form-inline-city-input", 180., false)),
            );
        let disabled = el_form(form.clone())
            .id("form-disabled")
            .disabled(true)
            .label_width(72.)
            .show_actions(true)
            .child(
                el_form_item(form.clone(), "name")
                    .id("form-disabled-name")
                    .label(s.form_name.clone())
                    .child(field("name", "form-disabled-name-input", 280., true)),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "form-horizontal",
                &s.form_horizontal,
                || "let form = FormState::new();\nel_form(form.clone())\n    .child(el_form_item(form.clone(), \"name\").label(\"Name\").child(el_input()));".into(),
                horizontal.into_any_element(),
            ))
            .child(self.section(
                cx,
                "form-vertical",
                &s.form_vertical,
                || "el_form(form.clone()).vertical().label_position(FormLabelPosition::Top);".into(),
                vertical.into_any_element(),
            ))
            .child(self.section(
                cx,
                "form-inline",
                &s.form_inline,
                || "el_form(form.clone()).inline().label_width(56.);".into(),
                inline.into_any_element(),
            ))
            .child(self.section(
                cx,
                "form-disabled",
                &s.form_disabled,
                || "el_form(form.clone()).disabled(true).show_actions(true);".into(),
                disabled.into_any_element(),
            ))
            .when(self.form.submitted, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(13.))
                        .text_color(gpui::rgb(0x67c23a))
                        .child(el_icon("circle-check").size(14.))
                        .child(s.form_submitted.clone()),
                )
            })
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("form-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick({
                            let form = form.clone();
                            cx.listener(move |this, _, _, cx| {
                                form.borrow_mut().reset();
                                this.form.submitted = false;
                                cx.notify();
                            })
                        }),
                ),
            )
    }
}
