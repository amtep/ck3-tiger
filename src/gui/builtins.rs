use strum::VariantNames;
use strum_macros::{Display, FromRepr, IntoStaticStr, VariantNames};
use thiserror::Error;

use crate::game::GameFlags;
use crate::lowercase::Lowercase;

/// Widget types that are defined by the game engine and don't need to be defined in gui script.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, FromRepr, Display,
)]
#[allow(non_camel_case_types)]
pub enum BuiltinWidget {
    axis,
    background,
    button,
    button_group,
    cameracontrolwidget,
    checkbutton,
    colormap_picker,
    colorpicker,
    container,
    contextmenu,
    datacontext_from_model,
    dockable_container,
    drag_drop_icon,
    drag_drop_target,
    dragdropicon,
    dragdroptarget,
    dropdown,
    dynamicgridbox,
    editbox,
    fixedgridbox,
    flowcontainer,
    game_button,
    hbox,
    icon,
    line,
    line_deprecated,
    margin_widget,
    mini_map,
    minimap,
    minimap_window,
    overlappingitembox,
    piechart,
    pieslice,
    plotline,
    portrait_button,
    progressbar,
    right_click_menu_widget,
    scrollarea,
    scrollbar,
    taborder,
    target,
    text_occluder,
    textbox,
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
    // LAST UPDATED CK3 VERSION 1.11.3
    // LAST UPDATED VIC3 VERSION 1.3.6
    // LAST UPDATED IMPERATOR VERSION 2.0.3
    pub fn to_game_flags(self) -> GameFlags {
        match self {
            BuiltinWidget::drag_drop_icon
            | BuiltinWidget::drag_drop_target
            | BuiltinWidget::game_button => GameFlags::Ck3,

            BuiltinWidget::minimap
            | BuiltinWidget::minimap_window
            | BuiltinWidget::right_click_menu_widget => GameFlags::Vic3,

            BuiltinWidget::dragdropicon
            | BuiltinWidget::mini_map
            | BuiltinWidget::dragdroptarget
            | BuiltinWidget::target
            | BuiltinWidget::taborder => GameFlags::Imperator,

            BuiltinWidget::colormap_picker
            | BuiltinWidget::datacontext_from_model
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
