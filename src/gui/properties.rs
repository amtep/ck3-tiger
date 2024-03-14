use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use strum::VariantNames;
use strum_macros::{Display, FromRepr, IntoStaticStr, VariantNames};
use thiserror::Error;

#[cfg(doc)]
use crate::datatype::Datatype;
use crate::game::GameFlags;
use crate::gui::BuiltinWidget;
#[cfg(doc)]
use crate::gui::GuiBlock;
use crate::item::Item;
use crate::lowercase::Lowercase;

use GuiValidation::*;
use WidgetProperty::*;

/// The various values or blocks or datatype calculations that the gui properties can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuiValidation {
    /// Accept any value; we don't know.
    UncheckedValue,
    /// A datatype expression; we don't know the specific type.
    DatatypeExpr,
    /// A datatype expression that ends with a promote. Can be any type.
    // TODO: check that it is a singular type, not a collection
    Datacontext,
    /// A datatype expression that returns a collection of some sort.
    // TODO: check that it is indeed a collection
    Datamodel,
    /// "yes", "no", or a [`Datatype::bool`] expression.
    Boolean,
    /// Only a literal "yes" will do.
    Yes,
    /// A `|`-separated list of values from [`ALIGN`].
    Align,
    /// An integer value or a [`Datatype::int32`] expression.
    Integer,
    /// A non-negative integer value or a [`Datatype::uint32`] expression.
    UnsignedInteger,
    /// A numeric value or a [`Datatype::float`] expression.
    Number,
    /// A numeric value or a [`Datatype::float`] or [`Datatype::int32`] expression.
    NumberOrInt32,
    /// A numeric value possibly followed by an `f` or a [`Datatype::float`] expression.
    // TODO: the f is used in vanilla. Check if it's harmless or maybe required.
    NumberF,
    /// A numeric value, possibly followed by a `%` mark, or a [`Datatype::int32`] or
    /// [`Datatype::float`] expression.
    NumberOrPercent,
    /// A block with two numeric values, possibly followed by `%` marks, or a
    /// [`Datatype::CVector2f`] expression.
    TwoNumberOrPercent,
    /// A block with 2 numbers, or a [`Datatype::CVector2f`] expression.
    CVector2f,
    /// A block with 2 integers, or a [`Datatype::CVector2i`] expression.
    CVector2i,
    /// A block with 3 numbers, or a [`Datatype::CVector3f`] expression.
    CVector3f,
    /// A block with 4 numbers, or a [`Datatype::CVector4f`] expression.
    CVector4f,
    /// A block with 4 numbers (RGBA), or a [`Datatype::CVector4f`] expression.
    /// The difference with [`GuiValidation::CVector4f`] is in the error messages.
    Color,
    /// A datatype expression returning a CString
    CString,
    /// A key for this item type, or a datatype expression that can be converted to string.
    Item(Item),
    /// A key for this item type, or a datatype expression that can be converted to string.
    /// May be the empty string.
    ItemOrBlank(Item),
    /// One of the strings in the [`BLENDMODES`] array
    Blendmode,
    /// One of the mouse buttons in the array given here.
    MouseButton(&'static [&'static str]),
    /// A `|`-separated list of the mouse buttons in the array given here.
    MouseButtonSet(&'static [&'static str]),
    /// One of the strings in the array given here.
    Choice(&'static [&'static str]),
    /// A `|`-separated list of strings from the array given here.
    ChoiceSet(&'static [&'static str]),
    /// A block containing one widget, or the name of a template containing one widget.
    Widget,
    /// A string containing a localization text format specifier starting with `#`
    Format,
    /// A block containing two textformat names
    FormatOverride,
    /// Some text. May contain datatypes or loca keys, but it's optional.
    RawText,
    /// Some text. May contain datatypes or loca keys.
    Text,
    /// A property that takes a block of other properties. This block may have override blocks in it.
    ComplexProperty,
}

/// All the properties that can be used in gui widgets.
// These need to be in lexical order, for the `TryFrom` implementation to work right.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, VariantNames, FromRepr, Display,
)]
#[allow(non_camel_case_types)]
#[strum(serialize_all = "lowercase")] // for "loop"
pub enum WidgetProperty {
    accept_tabs,
    active_item,
    addcolumn,
    addrow,
    align,
    allow_outside,
    alpha,
    alwaystransparent,
    animate_negative_changes,
    animation,
    animation_speed,
    attachto,
    autoresize,
    autoresize_slider,
    autoresizescrollarea,
    autoresizeviewport,
    axis_label,
    background_texture,
    bezier,
    blend_mode,
    bottomtotop,
    button_ignore,
    button_trigger,
    buttontext,
    camera_fov_y_degrees,
    camera_look_at,
    camera_near_far,
    camera_position,
    camera_rotation_pitch_limits,
    camera_translation_limits,
    camera_zoom_limits,
    checked,
    click_modifiers,
    clicksound,
    coat_of_arms,
    coat_of_arms_mask,
    coat_of_arms_slot,
    color,
    colorpicker_reticule_icon,
    constantbuffers,
    cursorcolor,
    datacontext,
    datamodel,
    datamodel_reuse_widgets,
    datamodel_wrap,
    dec_button,
    default_clicksound,
    default_format,
    delay,
    direction,
    disableframe,
    distribute_visual_state,
    down,
    downframe,
    downhoverframe,
    downpressedframe,
    drag_drop_args,
    drag_drop_base_type,
    drag_drop_data,
    drag_drop_id,
    dragdropargs,
    dragdropid,
    draggable_by,
    droptarget,
    duration,
    effect,
    effectname,
    elide,
    enabled,
    end_sound,
    endangle,
    entity_enable_sound,
    entity_instance,
    even_row_widget,
    expand_item,
    expandbutton,
    filter_mouse,
    fittype,
    flipdirection,
    focus_on_visible,
    focuspolicy,
    font,
    fontcolor,
    fontsize,
    fontsize_min,
    fonttintcolor,
    fontweight,
    force_data_properties_update,
    forcedown,
    format_override,
    frame,
    frame_tier,
    framesize,
    from,
    gfx_environment_file,
    gfxtype,
    glow,
    glow_alpha,
    glow_alpha_mask,
    glow_blur_passes,
    glow_generation_rules,
    glow_ignore_inside_pixels,
    glow_radius,
    glow_texture_downscale,
    grayscale,
    grid_entity_name,
    header_height,
    highlightchecked,
    ignore_in_debug_draw,
    ignore_unset_buttons,
    ignoreinvisible,
    inc_button,
    indent,
    index,
    inherit_data_context,
    inherit_visibility,
    inherit_visual_state,
    input_action,
    intersectionmask,
    intersectionmask_texture,
    invert_reticule_color,
    invertprogress,
    item,
    keyframe_editor_lane_container,
    layer,
    layoutanchor,
    layoutpolicy_horizontal,
    layoutpolicy_vertical,
    layoutstretchfactor_horizontal,
    layoutstretchfactor_vertical,
    line_cap,
    line_feather_distance,
    line_type,
    list,
    Loop, // titlecased to avoid collision with builtin loop
    loopinterval,
    margin,
    margin_bottom,
    margin_left,
    margin_right,
    margin_top,
    marker,
    mask,
    mask_uv_scale,
    max,
    max_height,
    max_update_rate,
    max_width,
    maxcharacters,
    maxhorizontalslots,
    maximumsize,
    maxverticalslots,
    min,
    min_dist_from_screen_edge,
    min_height,
    min_width,
    minimumsize,
    mipmaplodbias,
    mirror,
    modal,
    modality,
    modify_texture,
    movable,
    multiline,
    name,
    next,
    noprogresstexture,
    odd_row_widget,
    on_finish,
    on_keyframe_move,
    on_start,
    onalt,
    onchangefinish,
    onchangestart,
    onclick,
    oncolorchanged,
    oncoloredited,
    oncreate,
    ondatacontextchanged,
    ondefault,
    ondoubleclick,
    oneditingfinished,
    oneditingfinished_with_changes,
    oneditingstart,
    onenter_signal,
    onfocusout,
    onleave_signal,
    onmousehierarchyenter,
    onmousehierarchyleave,
    onpressed,
    onreleased,
    onreturnpressed,
    onrightclick,
    onselectionchanged,
    onshift,
    ontextchanged,
    ontextedited,
    onvaluechanged,
    overframe,
    oversound,
    page,
    pan_position,
    parentanchor,
    password,
    plotpoints,
    points,
    pop_out,
    pop_out_v,
    portrait_context,
    portrait_offset,
    portrait_scale,
    portrait_texture,
    position,
    position_x,
    position_y,
    preferscrollwidgetsize,
    progress_change_to_duration_curve,
    progresstexture,
    pseudo_localization_enabled,
    raw_text,
    raw_tooltip,
    realtime,
    recursive,
    reorder_on_mouse,
    resizable,
    resizeparent,
    restart_on_show,
    restrictparent_min,
    reuse_widgets,
    rightclick_modifiers,
    righttoleft,
    rotate_uv,
    row_height,
    scale,
    scale_mode,
    scissor,
    scrollbar_horizontal,
    scrollbar_vertical,
    scrollbaralign_horizontal,
    scrollbaralign_vertical,
    scrollbarpolicy_horizontal,
    scrollbarpolicy_vertical,
    scrollwidget,
    selectallonfocus,
    selectedindex,
    selectioncolor,
    set_parent_size_to_minimum,
    setitemsizefromcell,
    shaderfile,
    shortcut,
    size,
    skip_initial_animation,
    slider,
    snap_to_pixels,
    soundeffect,
    soundparam,
    spacing,
    spriteborder,
    spriteborder_bottom,
    spriteborder_left,
    spriteborder_right,
    spriteborder_top,
    spritetype,
    stackmode,
    start_sound,
    startangle,
    state,
    step,
    sticky,
    tabfocusroot,
    text,
    text_selectable,
    text_validator,
    texture,
    texture_density,
    timeline_line_direction,
    timeline_line_height,
    timeline_texts,
    timeline_time_points,
    tintcolor,
    to,
    tooltip,
    tooltip_enabled,
    tooltip_horizontalbehavior,
    tooltip_offset,
    tooltip_parentanchor,
    tooltip_type,
    tooltip_verticalbehavior,
    tooltip_visible,
    tooltip_widgetanchor,
    tooltipwidget,
    track,
    tracknavigation,
    translate_uv,
    trigger_on_create,
    trigger_when,
    upframe,
    uphoverframe,
    uppressedframe,
    url,
    useragent,
    uv_scale,
    value,
    video,
    viewportwidget,
    visible,
    visible_at_creation,
    wheelstep,
    widgetanchor,
    widgetid,
    width,
    zoom,
    zoom_max,
    zoom_min,
    zoom_step,
    zoomwidget,
}

#[derive(Error, Debug)]
pub enum TryWidgetPropertyError {
    #[error("widget property `{0}` not found")]
    NotFound(String),
}

impl<'a> TryFrom<&Lowercase<'a>> for WidgetProperty {
    type Error = TryWidgetPropertyError;

    fn try_from(s: &Lowercase<'a>) -> Result<Self, Self::Error> {
        match WidgetProperty::VARIANTS.binary_search(&s.as_str()) {
            // unwrap is safe here because of how VARIANTS is constructed
            Ok(i) => Ok(WidgetProperty::from_repr(i).unwrap()),
            Err(_) => Err(TryWidgetPropertyError::NotFound(s.to_string())),
        }
    }
}

const LAYOUT_POLICIES: &[&str] = &["expanding", "fixed", "growing", "preferred", "shrinking"];

pub const BLENDMODES: &[&str] =
    &["add", "alphamultiply", "colordodge", "darken", "mask", "multiply", "normal", "overlay"];

// TODO: warn about contradicting alignments (left|right or top|vcenter)
// TODO: is nobaseline only for text widgets?
pub const ALIGN: &[&str] =
    &["left", "right", "top", "bottom", "center", "hcenter", "vcenter", "nobaseline"];

impl GuiValidation {
    /// Get the validation logic for a specific widget property
    pub fn from_property(property: WidgetProperty) -> Self {
        #[allow(clippy::match_same_arms)] // keep it alphabetic
        match property {
            accept_tabs => Boolean,
            active_item => Widget,
            addcolumn => NumberOrPercent,
            addrow => NumberOrPercent,
            align => Align,
            allow_outside => Boolean,
            alpha => Number,
            alwaystransparent => Boolean,
            animate_negative_changes => Boolean,
            animation => ComplexProperty,
            animation_speed => CVector2f,
            attachto => ComplexProperty,
            autoresize => Boolean,
            autoresize_slider => Boolean,
            autoresizescrollarea => Boolean,
            autoresizeviewport => Boolean,
            axis_label => Widget,
            background_texture => Item(Item::File),
            bezier => CVector4f,
            blend_mode => Blendmode,
            bottomtotop => Boolean,
            button_ignore => MouseButton(&["both", "none", "left", "right"]),
            button_trigger => UncheckedValue, // only example is "none"
            buttontext => Widget,
            camera_fov_y_degrees => Integer,
            camera_look_at => CVector3f,
            camera_near_far => CVector2f,
            camera_position => CVector3f,
            camera_rotation_pitch_limits => CVector2f,
            camera_translation_limits => CVector3f,
            camera_zoom_limits => CVector2f,
            checked => Boolean,
            click_modifiers => ComplexProperty,
            clicksound => ItemOrBlank(Item::Sound),
            coat_of_arms => Item(Item::File),
            coat_of_arms_mask => Item(Item::File),
            coat_of_arms_slot => CVector4f,
            color => Color,
            colorpicker_reticule_icon => Widget,
            constantbuffers => DatatypeExpr,
            cursorcolor => Color,
            datacontext => Datacontext,
            datamodel => Datamodel,
            datamodel_reuse_widgets => Boolean,
            datamodel_wrap => Integer,
            dec_button => Widget,
            default_clicksound => ItemOrBlank(Item::Sound),
            default_format => Format,
            delay => Number,
            direction => Choice(&["horizontal", "vertical"]),
            disableframe => Integer,
            distribute_visual_state => Boolean,
            down => Boolean,
            downframe => Integer,
            downhoverframe => Integer,
            downpressedframe => Integer,
            drag_drop_args => CString,
            drag_drop_base_type => Choice(&["icon", "coat_of_arms_icon"]),
            drag_drop_data => Datacontext,
            drag_drop_id => UncheckedValue, // TODO what are the options?
            dragdropargs => RawText,
            dragdropid => RawText,
            draggable_by => MouseButtonSet(&["left", "right", "middle"]),
            droptarget => Boolean,
            duration => Number,
            effect => DatatypeExpr,
            effectname => UncheckedValue, // TODO validate effect names
            elide => Choice(&["right", "middle", "left"]),
            enabled => Boolean,
            end_sound => ComplexProperty,
            endangle => NumberOrInt32,
            entity_enable_sound => Boolean,
            entity_instance => Item(Item::Entity),
            even_row_widget => Widget,
            expand_item => Widget,
            expandbutton => Widget,
            filter_mouse => MouseButtonSet(&["all", "none", "left", "right", "wheel"]),
            fittype => Choice(&["center", "centercrop", "fill", "end", "start"]),
            flipdirection => Boolean,
            focus_on_visible => Boolean,
            focuspolicy => Choice(&["click", "all", "none"]),
            font => Item(Item::Font),
            fontcolor => Color,
            fontsize => Integer,
            fontsize_min => Integer,
            fonttintcolor => Color,
            fontweight => UncheckedValue, // TODO: what are the options?
            force_data_properties_update => Boolean,
            forcedown => DatatypeExpr,
            format_override => FormatOverride,
            frame => Integer,
            frame_tier => Integer,
            framesize => CVector2i,
            from => CVector2f,
            gfx_environment_file => Item(Item::File),
            gfxtype => UncheckedValue, // TODO: what are the options?
            glow => ComplexProperty,
            glow_alpha => Number,
            glow_alpha_mask => Integer,
            glow_blur_passes => Integer,
            glow_generation_rules => ComplexProperty,
            glow_ignore_inside_pixels => Boolean,
            glow_radius => Integer,
            glow_texture_downscale => NumberF,
            grayscale => Boolean,
            grid_entity_name => Item(Item::Entity),
            header_height => Integer,
            highlightchecked => Boolean,
            ignore_in_debug_draw => Boolean,
            ignore_unset_buttons => MouseButtonSet(&["right", "middle", "left"]), // middle and left are guesses
            ignoreinvisible => Boolean,
            inc_button => Widget,
            indent => Integer,
            index => Integer,
            inherit_data_context => Boolean,
            inherit_visibility => Choice(&["yes", "no", "hidden"]),
            inherit_visual_state => Boolean,
            input_action => Item(Item::Shortcut),
            intersectionmask => Boolean,
            intersectionmask_texture => Item(Item::File),
            invert_reticule_color => Boolean,
            invertprogress => Boolean,
            item => Widget,
            keyframe_editor_lane_container => Widget,
            layer => Item(Item::GuiLayer),
            layoutanchor => UncheckedValue, // TODO: only example is "bottomleft"
            layoutpolicy_horizontal => ChoiceSet(LAYOUT_POLICIES),
            layoutpolicy_vertical => ChoiceSet(LAYOUT_POLICIES),
            layoutstretchfactor_horizontal => NumberOrInt32,
            layoutstretchfactor_vertical => NumberOrInt32,
            line_cap => Boolean,
            line_feather_distance => Integer,
            line_type => UncheckedValue, // TODO: only example is "nodeline"
            list => Widget,
            Loop => Boolean,
            loopinterval => Number,
            margin => TwoNumberOrPercent,
            margin_bottom => NumberOrInt32,
            margin_left => NumberOrInt32,
            margin_right => NumberOrInt32,
            margin_top => NumberOrInt32,
            marker => Widget,
            mask => Item(Item::File),
            mask_uv_scale => CVector2f,
            max => NumberOrInt32,
            max_height => Integer,
            max_update_rate => Integer,
            max_width => Integer,
            maxcharacters => UnsignedInteger,
            maxhorizontalslots => Integer,
            maximumsize => TwoNumberOrPercent,
            maxverticalslots => Integer,
            min => NumberOrInt32,
            min_dist_from_screen_edge => Integer,
            min_height => Integer,
            min_width => Integer,
            minimumsize => TwoNumberOrPercent,
            mipmaplodbias => Integer,
            mirror => ChoiceSet(&["horizontal", "vertical"]),
            modal => Boolean,
            modality => UncheckedValue, // TODO: only example is "all"
            modify_texture => ComplexProperty,
            movable => Boolean,
            multiline => Boolean,
            name => UncheckedValue,
            next => UncheckedValue, // TODO: choices are states in the same widget
            noprogresstexture => Item(Item::File),
            odd_row_widget => Widget,
            on_finish => DatatypeExpr,
            on_keyframe_move => DatatypeExpr,
            on_start => DatatypeExpr,
            onalt => DatatypeExpr,
            onchangefinish => DatatypeExpr,
            onchangestart => DatatypeExpr,
            onclick => DatatypeExpr,
            oncolorchanged => DatatypeExpr,
            oncoloredited => DatatypeExpr,
            oncreate => DatatypeExpr,
            ondatacontextchanged => DatatypeExpr,
            ondefault => DatatypeExpr,
            ondoubleclick => DatatypeExpr,
            oneditingfinished => DatatypeExpr,
            oneditingfinished_with_changes => DatatypeExpr,
            oneditingstart => DatatypeExpr,
            onenter_signal => DatatypeExpr,
            onfocusout => DatatypeExpr,
            onleave_signal => DatatypeExpr,
            onmousehierarchyenter => DatatypeExpr,
            onmousehierarchyleave => DatatypeExpr,
            onpressed => DatatypeExpr,
            onreleased => DatatypeExpr,
            onreturnpressed => DatatypeExpr,
            onrightclick => DatatypeExpr,
            onselectionchanged => DatatypeExpr,
            onshift => DatatypeExpr,
            ontextchanged => DatatypeExpr,
            ontextedited => DatatypeExpr,
            onvaluechanged => DatatypeExpr,
            overframe => Integer,
            oversound => ItemOrBlank(Item::Sound),
            page => Integer,
            pan_position => CVector2f,
            parentanchor => Align,
            password => Boolean,
            plotpoints => DatatypeExpr,
            points => DatatypeExpr,
            pop_out => Boolean,
            pop_out_v => NumberOrInt32,
            portrait_context => DatatypeExpr,
            portrait_offset => CVector2f,
            portrait_scale => CVector2f,
            portrait_texture => Item(Item::File),
            position => TwoNumberOrPercent,
            position_x => Integer,
            position_y => Integer,
            preferscrollwidgetsize => Boolean,
            progress_change_to_duration_curve => CVector4f,
            progresstexture => Item(Item::File),
            pseudo_localization_enabled => Boolean,
            raw_text => RawText,
            raw_tooltip => RawText,
            realtime => Boolean,
            recursive => Yes,
            reorder_on_mouse => UncheckedValue, // TODO: only example is "presstop"
            resizable => Boolean,
            resizeparent => Boolean,
            restart_on_show => Boolean,
            restrictparent_min => Boolean,
            reuse_widgets => Boolean,
            rightclick_modifiers => ComplexProperty,
            righttoleft => Boolean,
            rotate_uv => Number,
            row_height => Integer,
            scale => Number,
            scale_mode => UncheckedValue, // TODO: only example is "fixedwidth"
            scissor => Boolean,
            scrollbar_horizontal => Widget,
            scrollbar_vertical => Widget,
            scrollbaralign_horizontal => Choice(&["top", "bottom"]),
            scrollbaralign_vertical => Choice(&["left", "right"]),
            scrollbarpolicy_horizontal => Choice(&["as_needed", "always_off", "always_on"]), // TODO: always_on is a guess
            scrollbarpolicy_vertical => Choice(&["as_needed", "always_off", "always_on"]),
            scrollwidget => Widget,
            selectallonfocus => Boolean,
            selectedindex => CVector2i,
            selectioncolor => Color,
            set_parent_size_to_minimum => Boolean,
            setitemsizefromcell => Boolean,
            shaderfile => ItemOrBlank(Item::File),
            shortcut => Item(Item::Shortcut),
            size => TwoNumberOrPercent,
            skip_initial_animation => Boolean,
            slider => Widget,
            snap_to_pixels => Boolean,
            soundeffect => Item(Item::Sound),
            soundparam => ComplexProperty,
            spacing => NumberF,
            spriteborder => CVector2f,
            spriteborder_bottom => Integer,
            spriteborder_left => Integer,
            spriteborder_right => Integer,
            spriteborder_top => Integer,
            spritetype => UncheckedValue, // TODO
            stackmode => UncheckedValue,  // TODO only example is "top"
            start_sound => ComplexProperty,
            startangle => NumberOrInt32,
            state => ComplexProperty,
            step => NumberOrInt32,
            sticky => Boolean,
            tabfocusroot => Boolean,
            text => Text,
            text_selectable => Boolean,
            text_validator => DatatypeExpr,
            texture => Item(Item::File),
            texture_density => Number,
            timeline_line_direction => UncheckedValue, // TODO only example is "up"
            timeline_line_height => Integer,
            timeline_texts => Widget,
            timeline_time_points => Integer,
            tintcolor => Color,
            to => CVector2f,
            tooltip => Text,
            tooltip_enabled => Boolean,
            tooltip_horizontalbehavior => Choice(&["mirror", "slide", "flip"]),
            tooltip_offset => TwoNumberOrPercent,
            tooltip_parentanchor => Align,
            tooltip_type => Choice(&["mouse", "widget"]),
            tooltip_verticalbehavior => Choice(&["mirror", "slide", "flip"]),
            tooltip_visible => Boolean,
            tooltip_widgetanchor => Align,
            tooltipwidget => Widget,
            track => Widget,
            tracknavigation => UncheckedValue, // TODO only example is "direct"
            translate_uv => CVector2f,
            trigger_on_create => Boolean,
            trigger_when => Boolean,
            upframe => Integer,
            uphoverframe => Integer,
            uppressedframe => Integer,
            url => RawText,
            useragent => UncheckedValue,
            uv_scale => CVector2f,
            value => NumberOrInt32,
            video => Item(Item::File),
            viewportwidget => Widget,
            visible => Boolean,
            visible_at_creation => Boolean,
            wheelstep => NumberOrInt32,
            widgetanchor => Align,
            widgetid => UncheckedValue,
            width => Number,
            zoom => Number,
            zoom_max => Number,
            zoom_min => Number,
            zoom_step => Number,
            zoomwidget => Widget,
        }
    }
}

impl WidgetProperty {
    /// Return which games support a given widget property
    // LAST UPDATED CK3 VERSION 1.9.2.1
    // LAST UPDATED VIC3 VERSION 1.3.6
    // LAST UPDATED IMPERATOR VERSION 2.0.3
    pub fn to_game_flags(self) -> GameFlags {
        #[allow(clippy::match_same_arms)] // alphabetic is better
        match self {
            animate_negative_changes | autoresize_slider | click_modifiers => {
                GameFlags::Ck3 | GameFlags::Vic3
            }

            coat_of_arms | coat_of_arms_mask => GameFlags::Ck3,

            coat_of_arms_slot | colorpicker_reticule_icon => GameFlags::Ck3 | GameFlags::Vic3,

            drag_drop_args | drag_drop_base_type | drag_drop_id => GameFlags::Ck3,

            drag_drop_data
            | focus_on_visible
            | force_data_properties_update
            | grid_entity_name
            | ignore_in_debug_draw
            | ignore_unset_buttons
            | index => GameFlags::Ck3 | GameFlags::Vic3,

            input_action => GameFlags::Vic3,

            invert_reticule_color | keyframe_editor_lane_container | Loop => {
                GameFlags::Ck3 | GameFlags::Vic3
            }

            max_height => GameFlags::Ck3,

            max_update_rate | min_dist_from_screen_edge => GameFlags::Ck3 | GameFlags::Vic3,

            min_height => GameFlags::Ck3,

            on_keyframe_move
            | onalt
            | ondefault
            | ondoubleclick
            | oneditingfinished_with_changes
            | oneditingstart
            | onfocusout
            | onshift
            | progress_change_to_duration_curve
            | pseudo_localization_enabled
            | raw_text
            | raw_tooltip
            | restart_on_show
            | rightclick_modifiers
            | selectallonfocus
            | skip_initial_animation
            | timeline_line_direction
            | timeline_line_height
            | timeline_texts
            | timeline_time_points => GameFlags::Ck3 | GameFlags::Vic3,

            tooltip_enabled => GameFlags::Vic3 | GameFlags::Imperator,
            tooltip_visible => GameFlags::Ck3,
            video => GameFlags::Ck3 | GameFlags::Vic3,

            frame_tier | pop_out_v | ondatacontextchanged | dragdropid | dragdropargs
            | forcedown | url | bottomtotop | onleave_signal | onenter_signal => {
                GameFlags::Imperator
            }

            _ => GameFlags::all(),
        }
    }
}

/// The container type of a [`GuiBlock`], which determines which properties are accepted.
/// Can be either a [`BuiltinWidget`] or a [`WidgetProperty`] of the [`GuiValidation::ComplexProperty`] type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyContainer {
    // The widget's ultimate base type.
    BuiltinWidget(BuiltinWidget),
    // A property that can hold other properties.
    ComplexProperty(WidgetProperty),
    // A property that can hold a widget.
    WidgetProperty(WidgetProperty),
}

impl From<BuiltinWidget> for PropertyContainer {
    fn from(w: BuiltinWidget) -> Self {
        PropertyContainer::BuiltinWidget(w)
    }
}

#[derive(Debug, Error, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WidgetPropertyContainerError {
    #[error("property `{0}` cannot be a container")]
    WrongPropertyKind(WidgetProperty),
}

impl TryFrom<WidgetProperty> for PropertyContainer {
    type Error = WidgetPropertyContainerError;
    fn try_from(prop: WidgetProperty) -> Result<Self, Self::Error> {
        let validation = GuiValidation::from_property(prop);
        if validation == GuiValidation::ComplexProperty {
            Ok(PropertyContainer::ComplexProperty(prop))
        } else if validation == GuiValidation::Widget {
            Ok(PropertyContainer::WidgetProperty(prop))
        } else {
            Err(WidgetPropertyContainerError::WrongPropertyKind(prop))
        }
    }
}

impl Display for PropertyContainer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            PropertyContainer::BuiltinWidget(builtin) => write!(f, "builtin widget {builtin}"),
            PropertyContainer::ComplexProperty(prop) => write!(f, "complex property {prop}"),
            PropertyContainer::WidgetProperty(prop) => write!(f, "widget property {prop}"),
        }
    }
}
