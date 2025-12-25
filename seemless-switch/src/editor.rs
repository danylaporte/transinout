use nih_plug::editor::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::{ParamButton, ParamSlider, ParamSliderExt};
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::SeemlessSwitchParams;

#[derive(Lens)]
pub struct EditorData {
    pub params: Arc<SeemlessSwitchParams>,
}

impl Model for EditorData {}

pub(crate) fn create_editor(
    params: Arc<SeemlessSwitchParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::default(), move |cx, _| {
        EditorData {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            // Title
            Label::new(cx, "Seemless Switch")
                .font_size(24.0)
                .font_weight(FontWeightKeyword::Bold)
                .height(Pixels(40.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));

            // Active toggle (large button) - FIXED
            ParamButton::new(cx, EditorData::params, |params| &params.active)
                .height(Pixels(50.0))
                .width(Percentage(80.0))
                .top(Pixels(10.0));

            // Allow Pitch Bend toggle - FIXED
            ParamButton::new(cx, EditorData::params, |params| &params.allow_pitch_bend)
                .height(Pixels(30.0))
                .width(Percentage(80.0))
                .top(Pixels(10.0));

            // Expression slider - FIXED
            ParamSlider::new(cx, EditorData::params, |params| &params.expr)
                .height(Pixels(30.0))
                .top(Pixels(15.0))
                .with_label("Expression");

            // Mod Wheel slider - FIXED
            ParamSlider::new(cx, EditorData::params, |params| &params.mw)
                .height(Pixels(30.0))
                .top(Pixels(10.0))
                .with_label("Mod Wheel");
        })
        .row_between(Pixels(10.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .width(Percentage(100.0))
        .height(Percentage(100.0));
    })
}
