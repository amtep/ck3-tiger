use strum::VariantNames;
use strum_macros::{Display, EnumVariantNames, FromRepr, IntoStaticStr};
use thiserror::Error;

use crate::game::GameFlags;
use crate::lowercase::Lowercase;

/// Widget types that are defined by the game engine and don't need to be defined in gui script.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, EnumVariantNames, FromRepr, Display,
)]
#[allow(non_camel_case_types)]
pub enum BuiltinWidget {
    animation,
    attachto,
    axis,
    axis_label,
    background,
    button,
    button_group,
    buttontext,
    cameracontrolwidget,
    checkbutton,
    click_modifiers,
    colormap_picker,
    colorpicker,
    colorpicker_reticule_icon,
    container,
    contextmenu,
    datacontext_from_model,
    dockable_container,
    drag_drop_icon,
    drag_drop_target,
    dragdropicon,
    dropdown,
    dynamicgridbox,
    editbox,
    end_sound,
    expand_item,
    expandbutton,
    fixedgridbox,
    flowcontainer,
    game_button,
    glow,
    glow_generation_rules,
    hbox,
    icon,
    keyframe_editor_lane_container,
    line,
    line_deprecated,
    list,
    margin_widget,
    marker,
    mini_map,
    minimap,
    minimap_window,
    modify_texture,
    overlappingitembox,
    piechart,
    pieslice,
    plotline,
    portrait_button,
    progressbar,
    right_click_menu_widget,
    rightclick_modifiers,
    scrollarea,
    scrollbar,
    scrollbar_horizontal,
    scrollbar_vertical,
    scrollwidget,
    soundparam,
    start_sound,
    state,
    text_occluder,
    textbox,
    timeline_texts,
    tools_dragdrop_widget,
    tools_keyframe_button,
    tools_keyframe_editor,
    tools_keyframe_editor_lane,
    tools_player_timeline,
    tools_table,
    tree,
    treemapchart,
    treemapslice,
    vbox,
    webwindow,
    widget,
    window,
    zoomarea,
}

#[derive(Error, Debug)]
pub enum TryBuiltinWidgetError {
    #[error("builtin widget `{0}` not found")]
    NotFound(String),
}

impl<'a> TryFrom<&Lowercase<'a>> for BuiltinWidget {
    type Error = TryBuiltinWidgetError;

    fn try_from(s: &Lowercase<'a>) -> Result<Self, Self::Error> {
        match BuiltinWidget::VARIANTS.binary_search(&s.as_str()) {
            // unwrap is safe here because of how VARIANTS is constructed
            Ok(i) => Ok(BuiltinWidget::from_repr(i).unwrap()),
            Err(_) => Err(TryBuiltinWidgetError::NotFound(s.to_string())),
        }
    }
}

impl BuiltinWidget {
    /// Return which games support the given builtin widget type
    // TODO - imperator - remove the non-imperator ones from GameFlags::all(), and add any that are missing.
    pub fn to_game_flags(self) -> GameFlags {
        match self {
            BuiltinWidget::drag_drop_icon
            | BuiltinWidget::drag_drop_target
            | BuiltinWidget::game_button => GameFlags::Ck3,

            BuiltinWidget::minimap
            | BuiltinWidget::minimap_window
            | BuiltinWidget::right_click_menu_widget => GameFlags::Vic3,

            BuiltinWidget::dragdropicon | BuiltinWidget::mini_map => GameFlags::Imperator,

            BuiltinWidget::cameracontrolwidget
            | BuiltinWidget::click_modifiers
            | BuiltinWidget::colormap_picker
            | BuiltinWidget::colorpicker_reticule_icon
            | BuiltinWidget::datacontext_from_model
            | BuiltinWidget::keyframe_editor_lane_container
            | BuiltinWidget::rightclick_modifiers
            | BuiltinWidget::timeline_texts
            | BuiltinWidget::tools_dragdrop_widget
            | BuiltinWidget::tools_keyframe_button
            | BuiltinWidget::tools_keyframe_editor
            | BuiltinWidget::tools_keyframe_editor_lane
            | BuiltinWidget::tools_player_timeline
            | BuiltinWidget::treemapchart
            | BuiltinWidget::treemapslice => GameFlags::Ck3 | GameFlags::Vic3,

            BuiltinWidget::button | BuiltinWidget::webwindow => {
                GameFlags::Vic3 | GameFlags::Imperator
            }

            _ => GameFlags::all(),
        }
    }

    pub fn builtin_current_game(s: &Lowercase) -> Option<BuiltinWidget> {
        if let Ok(builtin) = Self::try_from(s) {
            if builtin.to_game_flags().contains(GameFlags::game()) {
                return Some(builtin);
            }
        }
        None
    }
}
