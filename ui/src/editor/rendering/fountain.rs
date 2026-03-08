fn fountain_line_style(kind: &LineKind) -> Option<LineRenderStyle> {
    match kind {
        LineKind::SceneHeading => Some(LineRenderStyle::new(FontVariant::Bold, COLOR_SCENE, 1.0, 1.0)),
        LineKind::Action => Some(LineRenderStyle::new(FontVariant::Regular, COLOR_ACTION, 1.0, 1.0)),
        LineKind::Character => Some(LineRenderStyle::new(
            FontVariant::Bold,
            COLOR_CHARACTER,
            1.0,
            1.0,
        )),
        LineKind::Dialogue => Some(LineRenderStyle::new(
            FontVariant::Regular,
            COLOR_DIALOGUE,
            1.0,
            1.0,
        )),
        LineKind::Parenthetical => Some(LineRenderStyle::new(
            FontVariant::Italic,
            COLOR_PARENTHETICAL,
            1.0,
            1.0,
        )),
        LineKind::Transition => Some(LineRenderStyle::new(
            FontVariant::BoldItalic,
            COLOR_TRANSITION,
            1.0,
            1.0,
        )),
        _ => None,
    }
}
