use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiColor {
    PlainBackground,
    PanelBackground,
    InputBackground,
    ButtonBackground,
    ButtonHover,
    ButtonPressed,
    ActiveBackground,
    Border,
}

impl UiColor {
    pub(crate) const ALL: [Self; 8] = [
        Self::PlainBackground,
        Self::PanelBackground,
        Self::InputBackground,
        Self::ButtonBackground,
        Self::ButtonHover,
        Self::ButtonPressed,
        Self::ActiveBackground,
        Self::Border,
    ];

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::PlainBackground => "plain_background",
            Self::PanelBackground => "panel_background",
            Self::InputBackground => "input_background",
            Self::ButtonBackground => "button_background",
            Self::ButtonHover => "button_hover",
            Self::ButtonPressed => "button_pressed",
            Self::ActiveBackground => "active_background",
            Self::Border => "border_color",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PlainBackground => "Plain editor",
            Self::PanelBackground => "Panels / popups",
            Self::InputBackground => "Input fields",
            Self::ButtonBackground => "Buttons",
            Self::ButtonHover => "Button hover",
            Self::ButtonPressed => "Button pressed",
            Self::ActiveBackground => "Active controls / file",
            Self::Border => "Borders",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiPalette([Vec4; UiColor::ALL.len()]);

impl Default for UiPalette {
    fn default() -> Self {
        Self([
            Vec4::new(0.96, 0.96, 0.97, 1.0),
            Vec4::new(0.92, 0.93, 0.95, 1.0),
            Vec4::new(0.99, 0.99, 1.00, 1.0),
            Vec4::new(0.80, 0.82, 0.84, 1.0),
            Vec4::new(0.74, 0.77, 0.80, 1.0),
            Vec4::new(0.68, 0.72, 0.76, 1.0),
            Vec4::new(0.65, 0.75, 0.87, 1.0),
            Vec4::new(0.65, 0.68, 0.72, 1.0),
        ])
    }
}

impl UiPalette {
    pub(crate) fn dark() -> Self {
        Self([
            Vec4::new(0.12, 0.13, 0.15, 1.0),
            Vec4::new(0.16, 0.17, 0.19, 1.0),
            Vec4::new(0.12, 0.13, 0.15, 1.0),
            Vec4::new(0.24, 0.26, 0.29, 1.0),
            Vec4::new(0.30, 0.33, 0.37, 1.0),
            Vec4::new(0.18, 0.20, 0.23, 1.0),
            Vec4::new(0.22, 0.31, 0.43, 1.0),
            Vec4::new(0.34, 0.36, 0.40, 1.0),
        ])
    }

    pub(crate) fn rgba(&self, role: UiColor) -> Vec4 {
        self.0[role as usize]
    }

    pub(crate) fn set(&mut self, role: UiColor, rgba: Vec4) {
        self.0[role as usize] = clamp_vec4_rgba(rgba);
    }

    pub(crate) fn color(&self, role: UiColor) -> Color {
        color_from_rgba(self.rgba(role))
    }
}

#[derive(Component)]
#[require(BackgroundColor)]
pub(crate) struct ThemedBackground(pub(crate) UiColor);

#[derive(Component)]
pub(crate) struct OpaqueThemedBackground;

#[derive(Component)]
#[require(BorderColor)]
pub(crate) struct ThemedBorder;

#[derive(Component)]
#[require(BackgroundColor)]
pub(crate) struct ThemedButton;

#[derive(Component)]
pub(crate) struct ThemeColorSliderRow(pub(crate) ThemeSliderChannel);

#[derive(Component)]
#[require(TextColor)]
pub(crate) enum ThemedText {
    Main,
    Muted,
}

pub(crate) fn themed_button_color(state: &EditorState, interaction: Interaction) -> Color {
    state.ui_colors.color(match interaction {
        Interaction::None => UiColor::ButtonBackground,
        Interaction::Hovered => UiColor::ButtonHover,
        Interaction::Pressed => UiColor::ButtonPressed,
    })
}

// Run after presentation so newly spawned widgets and live theme changes are
// styled in the same frame. Document text and color-picker swatches opt out.
pub(crate) fn sync_theme_widgets(
    state: Res<EditorState>,
    mut backgrounds: Query<
        (
            &ThemedBackground,
            Has<OpaqueThemedBackground>,
            &mut BackgroundColor,
        ),
        Without<ThemedButton>,
    >,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (
            With<ThemedButton>,
            Without<MarkdownMetadataFieldButton>,
            Without<MarkdownMetadataDropdownOptionButton>,
            Without<ThemeOverlayTabButton>,
            Without<StoryQueryDropdownOptionNode>,
            Without<WorkspaceLinkFolderOption>,
        ),
    >,
    mut texts: Query<(&ThemedText, &mut TextColor)>,
    mut borders: Query<&mut BorderColor, With<ThemedBorder>>,
) {
    for (role, opaque, mut color) in &mut backgrounds {
        let theme_color = state.ui_colors.color(role.0);
        color.set_if_neq(BackgroundColor(if opaque {
            theme_color.with_alpha(1.0)
        } else {
            theme_color
        }));
    }
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(themed_button_color(&state, *interaction)));
    }
    for (role, mut color) in &mut texts {
        color.set_if_neq(TextColor(match role {
            ThemedText::Main => state.text_main_color,
            ThemedText::Muted => state.text_muted_color,
        }));
    }
    for mut border in &mut borders {
        border.set_if_neq(BorderColor::all(state.ui_colors.color(UiColor::Border)));
    }
}
