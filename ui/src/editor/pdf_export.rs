use super::*;
use printpdf::{
    Actions, Color as PdfColor, Line, LinePoint, LinkAnnotation, Mm, Op, PaintMode, ParsedFont,
    PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, RawImage, Rect, Rgb, TextItem,
    XObjectTransform,
};

const COURIER_REGULAR: &[u8] = include_bytes!("../../../fonts/Courier Prime/Courier Prime.ttf");
const COURIER_BOLD: &[u8] = include_bytes!("../../../fonts/Courier Prime/Courier Prime Bold.ttf");
const COURIER_ITALIC: &[u8] =
    include_bytes!("../../../fonts/Courier Prime/Courier Prime Italic.ttf");
const COURIER_BOLD_ITALIC: &[u8] =
    include_bytes!("../../../fonts/Courier Prime/Courier Prime Bold Italic.ttf");
const SEGOE_REGULAR: &[u8] = include_bytes!("../../../fonts/segoe-ui-4/Segoe UI.ttf");
const SEGOE_BOLD: &[u8] = include_bytes!("../../../fonts/segoe-ui-4/Segoe UI Bold.ttf");
const SEGOE_ITALIC: &[u8] = include_bytes!("../../../fonts/segoe-ui-4/Segoe UI Italic.ttf");
const SEGOE_BOLD_ITALIC: &[u8] =
    include_bytes!("../../../fonts/segoe-ui-4/Segoe UI Bold Italic.ttf");

#[derive(Clone, Debug)]
pub(crate) struct PdfExportSnapshot {
    pub(crate) title: String,
    pub(crate) format: DocumentFormat,
    pub(crate) page_width_pt: f32,
    pub(crate) page_height_pt: f32,
    pub(crate) text_left_pt: f32,
    pub(crate) text_width_pt: f32,
    pub(crate) lines_per_page: usize,
    pub(crate) page_step_lines: usize,
    pub(crate) pages: Vec<PdfExportPage>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PdfExportPage {
    pub(crate) lines: Vec<PdfExportLine>,
}

#[derive(Clone, Debug)]
pub(crate) struct PdfExportLine {
    pub(crate) top_pt: f32,
    pub(crate) style: LineRenderStyle,
    pub(crate) allow_link_color: bool,
    pub(crate) visual: ProcessedVisualLine,
}

#[derive(Clone, Debug)]
pub(crate) struct PdfExportOutcome {
    pub(crate) page_count: usize,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone)]
struct PdfFonts {
    regular: printpdf::FontId,
    bold: printpdf::FontId,
    italic: printpdf::FontId,
    bold_italic: printpdf::FontId,
}

impl PdfFonts {
    fn id(&self, variant: FontVariant) -> printpdf::FontId {
        match variant {
            FontVariant::Regular => self.regular.clone(),
            FontVariant::Bold => self.bold.clone(),
            FontVariant::Italic => self.italic.clone(),
            FontVariant::BoldItalic => self.bold_italic.clone(),
        }
    }
}

pub(crate) fn normalize_pdf_path(mut path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.set_extension("pdf");
    }
    path
}

pub(crate) fn build_pdf_export_snapshot(
    state: &mut EditorState,
) -> Result<PdfExportSnapshot, String> {
    if state.document_format == DocumentFormat::Canvas {
        return Err(
            "Canvas PDF export is not supported yet; export a Fountain or Markdown document."
                .to_string(),
        );
    }

    let base_paper_height =
        ((processed_page_step_lines() as f32 * LINE_HEIGHT) - PAGE_GAP).max(1.0);
    let text_width = (A4_WIDTH_POINTS - state.page_margin_left - state.page_margin_right).max(1.0);
    let text_height =
        (base_paper_height - state.page_margin_top - state.page_margin_bottom).max(1.0);
    let char_width = default_char_width_for_format(state.document_format).max(0.1);
    let wrap_columns = ((text_width / char_width) + 1e-4).floor().max(1.0) as usize;
    let lines_per_page = ((text_height / LINE_HEIGHT) + 1e-4).floor().max(1.0) as usize;
    let page_step_lines = processed_page_step_lines().max(1);
    let spacer_lines = page_step_lines.saturating_sub(lines_per_page);

    // Export is always the stable paginated processed representation. Temporarily setting these
    // two presentation flags lets the existing canonical line builder do exactly the same page
    // fill work without polluting its viewport cache.
    let previous_paginated = state.processed_paginated;
    let previous_display_mode = state.display_mode;
    state.processed_paginated = true;
    state.display_mode = DisplayMode::Processed;
    let cache = build_processed_cache(state, wrap_columns, lines_per_page, spacer_lines);

    let page_count = processed_page_count_for_lines(&cache.lines, page_step_lines).max(1);
    let mut pages = Vec::with_capacity(page_count);
    for page_index in 0..page_count {
        let page_start = page_index.saturating_mul(page_step_lines);
        let mut top_units = 0.0_f32;
        let mut page = PdfExportPage::default();
        for line_offset in 0..lines_per_page {
            let Some(visual) = cache.lines.get(page_start.saturating_add(line_offset)) else {
                break;
            };
            let (style, allow_link_color) = processed_visual_line_style_for_state(state, visual);
            if !visual.is_spacer {
                page.lines.push(PdfExportLine {
                    top_pt: state.page_margin_top + top_units * LINE_HEIGHT,
                    style,
                    allow_link_color,
                    visual: visual.clone(),
                });
            }
            top_units += style.line_height_scale.max(0.0);
        }
        pages.push(page);
    }
    state.processed_paginated = previous_paginated;
    state.display_mode = previous_display_mode;

    let title = state
        .paths
        .load_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("BasScript document")
        .to_string();

    Ok(PdfExportSnapshot {
        title,
        format: state.document_format,
        page_width_pt: A4_WIDTH_POINTS,
        page_height_pt: A4_HEIGHT_POINTS,
        text_left_pt: state.page_margin_left,
        text_width_pt: text_width,
        lines_per_page,
        page_step_lines,
        pages,
    })
}

pub(crate) fn export_pdf_to_path(
    state: &mut EditorState,
    requested_path: PathBuf,
) -> Result<(PathBuf, PdfExportOutcome), String> {
    let path = normalize_pdf_path(requested_path);
    let snapshot = build_pdf_export_snapshot(state)?;
    let outcome = write_pdf_snapshot(&snapshot, state, &path)?;
    Ok((path, outcome))
}

pub(crate) fn write_pdf_snapshot(
    snapshot: &PdfExportSnapshot,
    state: &EditorState,
    path: &Path,
) -> Result<PdfExportOutcome, String> {
    let mut pdf = PdfDocument::new(&snapshot.title);
    let mut backend_warnings = Vec::new();
    let fonts = add_pdf_fonts(&mut pdf, snapshot.format)?;
    let mut warnings = Vec::<String>::new();
    let mut pages = Vec::with_capacity(snapshot.pages.len());
    debug_assert!(snapshot.lines_per_page <= snapshot.page_step_lines);

    for page in &snapshot.pages {
        let mut ops = Vec::<Op>::new();
        for line in &page.lines {
            if let Some(image) = line.visual.image_block.as_ref() {
                push_pdf_image(
                    &mut pdf,
                    &mut ops,
                    &mut warnings,
                    snapshot,
                    state,
                    line.top_pt,
                    image,
                );
                continue;
            }
            if let Some(checked) = line.visual.markdown_checklist_checked {
                push_pdf_checklist(&mut ops, snapshot, line.top_pt, checked);
            }
            push_pdf_text_line(&mut ops, snapshot, state, &fonts, line);
        }
        pages.push(PdfPage::new(
            Mm(snapshot.page_width_pt / POINTS_PER_INCH * MM_PER_INCH),
            Mm(snapshot.page_height_pt / POINTS_PER_INCH * MM_PER_INCH),
            ops,
        ));
    }

    pdf.with_pages(pages);
    let bytes = exact_a4_pdf_bytes(
        &pdf,
        snapshot.page_width_pt,
        snapshot.page_height_pt,
        &mut backend_warnings,
    )?;
    atomic_write(path, &bytes)?;

    Ok(PdfExportOutcome {
        page_count: snapshot.pages.len(),
        warnings,
    })
}

fn exact_a4_pdf_bytes(
    pdf: &PdfDocument,
    width_pt: f32,
    height_pt: f32,
    warnings: &mut Vec<printpdf::PdfWarnMsg>,
) -> Result<Vec<u8>, String> {
    let mut document = pdf.to_lopdf_document(&PdfSaveOptions::default(), warnings);
    let page_box = lopdf::Object::Array(vec![
        lopdf::Object::Integer(0),
        lopdf::Object::Integer(0),
        lopdf::Object::Real(width_pt),
        lopdf::Object::Real(height_pt),
    ]);
    for (_, page_id) in document.get_pages() {
        let page = document
            .get_object_mut(page_id)
            .map_err(|error| format!("Could not access generated PDF page: {error}"))?
            .as_dict_mut()
            .map_err(|error| format!("Generated PDF page was invalid: {error}"))?;
        page.set("MediaBox", page_box.clone());
        page.set("TrimBox", page_box.clone());
        page.set("CropBox", page_box.clone());
    }
    let mut bytes = Vec::new();
    document
        .save_to(&mut bytes)
        .map_err(|error| format!("Could not serialize generated PDF: {error}"))?;
    Ok(bytes)
}

fn add_pdf_fonts(pdf: &mut PdfDocument, format: DocumentFormat) -> Result<PdfFonts, String> {
    let mut add = |variant| {
        let bytes = pdf_font_bytes(format, variant);
        let mut font_warnings = Vec::new();
        let parsed = ParsedFont::from_bytes(bytes, 0, &mut font_warnings)
            .ok_or_else(|| format!("Could not parse the bundled {:?} PDF font.", variant))?;
        Ok::<_, String>(pdf.add_font(&parsed))
    };
    Ok(PdfFonts {
        regular: add(FontVariant::Regular)?,
        bold: add(FontVariant::Bold)?,
        italic: add(FontVariant::Italic)?,
        bold_italic: add(FontVariant::BoldItalic)?,
    })
}

fn pdf_font_bytes(format: DocumentFormat, variant: FontVariant) -> &'static [u8] {
    match (format, variant) {
        (DocumentFormat::Fountain, FontVariant::Regular) => COURIER_REGULAR,
        (DocumentFormat::Fountain, FontVariant::Bold) => COURIER_BOLD,
        (DocumentFormat::Fountain, FontVariant::Italic) => COURIER_ITALIC,
        (DocumentFormat::Fountain, FontVariant::BoldItalic) => COURIER_BOLD_ITALIC,
        (_, FontVariant::Regular) => SEGOE_REGULAR,
        (_, FontVariant::Bold) => SEGOE_BOLD,
        (_, FontVariant::Italic) => SEGOE_ITALIC,
        (_, FontVariant::BoldItalic) => SEGOE_BOLD_ITALIC,
    }
}

fn push_pdf_text_line(
    ops: &mut Vec<Op>,
    snapshot: &PdfExportSnapshot,
    state: &EditorState,
    fonts: &PdfFonts,
    line: &PdfExportLine,
) {
    let font_size = FONT_SIZE * line.style.font_scale;
    let baseline_from_top = font_ascender_pt(
        pdf_font_bytes(snapshot.format, line.style.font_variant),
        font_size,
    );
    let baseline_y = snapshot.page_height_pt - line.top_pt - baseline_from_top;
    let mut x = snapshot.text_left_pt;

    for fragment in &line.visual.fragments {
        if fragment.text.is_empty() {
            continue;
        }
        let variant = font_variant_for_processed_fragment(line.style.font_variant, fragment);
        let bytes = pdf_font_bytes(snapshot.format, variant);
        let width = font_text_width_pt(bytes, &fragment.text, font_size);
        let color = if line.allow_link_color
            && fragment.is_link
            && state.processed_link_color_mode == ProcessedLinkColorMode::Colored
        {
            state.processed_link_color_for_target(fragment.link_target.as_deref())
        } else {
            line.style.color
        };

        ops.extend([
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point {
                    x: Pt(x),
                    y: Pt(baseline_y),
                },
            },
            Op::SetFont {
                font: PdfFontHandle::External(fonts.id(variant)),
                size: Pt(font_size),
            },
            Op::SetFillColor {
                col: pdf_color(color),
            },
            Op::ShowText {
                items: vec![TextItem::Text(fragment.text.clone())],
            },
            Op::EndTextSection,
        ]);

        if fragment.is_link
            && fragment.link_target.as_deref().is_some_and(|target| {
                target.starts_with("http://") || target.starts_with("https://")
            })
        {
            let target = fragment.link_target.clone().unwrap_or_default();
            ops.push(Op::LinkAnnotation {
                link: LinkAnnotation::new(
                    Rect {
                        x: Pt(x),
                        y: Pt(baseline_y - font_size * 0.2),
                        width: Pt(width.max(0.1)),
                        height: Pt(font_size),
                        mode: None,
                        winding_order: None,
                    },
                    Actions::uri(target),
                    None,
                    Some(printpdf::ColorArray::Transparent),
                    None,
                ),
            });
        }
        x += width;
    }
}

fn push_pdf_checklist(ops: &mut Vec<Op>, snapshot: &PdfExportSnapshot, top_pt: f32, checked: bool) {
    let icon_size = (LINE_HEIGHT * 0.72).clamp(8.0, 16.0);
    let icon_gap = (LINE_HEIGHT * 0.20).clamp(2.0, 4.0);
    let size = icon_size;
    let x = (snapshot.text_left_pt - icon_size - icon_gap).max(0.0);
    let top = top_pt + ((LINE_HEIGHT - icon_size) * 0.5).max(0.0);
    let y = snapshot.page_height_pt - top - icon_size;
    ops.extend([
        Op::SaveGraphicsState,
        Op::SetOutlineColor {
            col: pdf_color(COLOR_ACTION),
        },
        Op::SetOutlineThickness { pt: Pt(0.75) },
        Op::DrawRectangle {
            rectangle: Rect {
                x: Pt(x),
                y: Pt(y),
                width: Pt(size),
                height: Pt(size),
                mode: Some(PaintMode::Stroke),
                winding_order: None,
            },
        },
    ]);
    if checked {
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    pdf_line_point(x + 1.2, y + 3.4),
                    pdf_line_point(x + 3.0, y + 1.5),
                    pdf_line_point(x + 6.2, y + 5.8),
                ],
                is_closed: false,
            },
        });
    }
    ops.push(Op::RestoreGraphicsState);
}

fn pdf_line_point(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point { x: Pt(x), y: Pt(y) },
        bezier: false,
    }
}

fn push_pdf_image(
    pdf: &mut PdfDocument,
    ops: &mut Vec<Op>,
    warnings: &mut Vec<String>,
    snapshot: &PdfExportSnapshot,
    state: &EditorState,
    top_pt: f32,
    image: &ProcessedImageBlock,
) {
    let reserved_height =
        (image.reserved_lines.max(1) as f32 * LINE_HEIGHT - PROCESSED_IMAGE_BLOCK_GAP).max(1.0);
    let target = image.target.trim();
    if is_remote_image_target(target) {
        warnings.push(format!("Remote image was not embedded: {target}"));
        push_pdf_image_placeholder(ops, snapshot, top_pt, reserved_height);
        return;
    }
    let path = match resolve_processed_image_path(state, target) {
        Ok(path) => path,
        Err(error) => {
            warnings.push(format!(
                "Image path could not be resolved ({target}): {error}"
            ));
            push_pdf_image_placeholder(ops, snapshot, top_pt, reserved_height);
            return;
        }
    };
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(format!(
                "Image could not be read ({}): {error}",
                path.display()
            ));
            push_pdf_image_placeholder(ops, snapshot, top_pt, reserved_height);
            return;
        }
    };
    let mut image_warnings = Vec::new();
    let raw = match RawImage::decode_from_bytes(&bytes, &mut image_warnings) {
        Ok(raw) => raw,
        Err(error) => {
            warnings.push(format!(
                "Image could not be decoded ({}): {error}",
                path.display()
            ));
            push_pdf_image_placeholder(ops, snapshot, top_pt, reserved_height);
            return;
        }
    };

    let (display_width, display_height) = fit_image_box(
        raw.width as f32,
        raw.height as f32,
        snapshot.text_width_pt,
        reserved_height,
    );
    let x = snapshot.text_left_pt + (snapshot.text_width_pt - display_width) * 0.5;
    let top = top_pt + (reserved_height - display_height) * 0.5;
    let y = snapshot.page_height_pt - top - display_height;
    let id = pdf.add_image(&raw);
    ops.push(Op::UseXobject {
        id,
        transform: XObjectTransform {
            translate_x: Some(Pt(x)),
            translate_y: Some(Pt(y)),
            scale_x: Some(display_width / raw.width.max(1) as f32),
            scale_y: Some(display_height / raw.height.max(1) as f32),
            dpi: Some(72.0),
            ..default()
        },
    });
}

fn fit_image_box(
    image_width: f32,
    image_height: f32,
    max_width: f32,
    max_height: f32,
) -> (f32, f32) {
    let image_width = image_width.max(1.0);
    let image_height = image_height.max(1.0);
    let natural_height = max_width.max(1.0) * image_height / image_width;
    let height = natural_height.min(max_height.max(1.0)).max(1.0);
    let width = (height * image_width / image_height)
        .min(max_width.max(1.0))
        .max(1.0);
    (width, height)
}

fn push_pdf_image_placeholder(
    ops: &mut Vec<Op>,
    snapshot: &PdfExportSnapshot,
    top_pt: f32,
    reserved_height: f32,
) {
    let (width, height) = fit_image_box(16.0, 9.0, snapshot.text_width_pt, reserved_height);
    let x = snapshot.text_left_pt + (snapshot.text_width_pt - width) * 0.5;
    let y = snapshot.page_height_pt - top_pt - (reserved_height + height) * 0.5;
    ops.extend([
        Op::SaveGraphicsState,
        Op::SetFillColor {
            col: pdf_color(COLOR_IMAGE_PLACEHOLDER),
        },
        Op::DrawRectangle {
            rectangle: Rect {
                x: Pt(x),
                y: Pt(y),
                width: Pt(width),
                height: Pt(height),
                mode: Some(PaintMode::Fill),
                winding_order: None,
            },
        },
        Op::RestoreGraphicsState,
    ]);
}

fn font_ascender_pt(bytes: &[u8], font_size: f32) -> f32 {
    ttf_parser::Face::parse(bytes, 0)
        .map(|face| face.ascender() as f32 / face.units_per_em() as f32 * font_size)
        .unwrap_or(font_size * 0.8)
}

fn font_text_width_pt(bytes: &[u8], text: &str, font_size: f32) -> f32 {
    let Ok(face) = ttf_parser::Face::parse(bytes, 0) else {
        return text.chars().count() as f32 * font_size * 0.6;
    };
    let units = text
        .chars()
        .filter_map(|ch| face.glyph_index(ch))
        .filter_map(|glyph| face.glyph_hor_advance(glyph))
        .map(u32::from)
        .sum::<u32>();
    units as f32 / face.units_per_em() as f32 * font_size
}

fn pdf_color(color: Color) -> PdfColor {
    let rgba = color.to_srgba();
    PdfColor::Rgb(Rgb::new(rgba.red, rgba.green, rgba.blue, None))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create PDF destination directory: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("export.pdf");
    let temp = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::write(&temp, bytes).map_err(|error| format!("Could not write temporary PDF: {error}"))?;
    if let Err(error) = fs::rename(&temp, path) {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_error| {
                format!("Could not replace existing PDF: {remove_error}")
            })?;
            fs::rename(&temp, path)
                .map_err(|rename_error| format!("Could not finalize PDF: {rename_error}"))?;
        } else {
            let _ = fs::remove_file(&temp);
            return Err(format!("Could not finalize PDF: {error}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_point_geometry_is_exact() {
        assert!((A4_WIDTH_POINTS - 595.2756).abs() < 0.001);
        assert!((A4_HEIGHT_POINTS - 841.8898).abs() < 0.001);
    }

    #[test]
    fn appends_pdf_only_when_extension_is_missing() {
        assert_eq!(
            normalize_pdf_path(PathBuf::from("draft")),
            PathBuf::from("draft.pdf")
        );
        assert_eq!(
            normalize_pdf_path(PathBuf::from("draft.PDF")),
            PathBuf::from("draft.PDF")
        );
    }

    #[test]
    fn image_fit_preserves_aspect_ratio_and_bounds() {
        let (width, height) = fit_image_box(1920.0, 1080.0, 400.0, 120.0);
        assert!(width <= 400.0 && height <= 120.0);
        assert!((width / height - 16.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn courier_prime_twelve_point_advance_matches_screenplay_cell() {
        let width = font_text_width_pt(COURIER_REGULAR, "MMMMMMMMMM", 12.0) / 10.0;
        assert!((width - DEFAULT_CHAR_WIDTH).abs() < 0.02);
    }

    #[test]
    fn markdown_upright_fonts_are_static_and_cover_editor_symbols() {
        for bytes in [SEGOE_REGULAR, SEGOE_BOLD] {
            let face = ttf_parser::Face::parse(bytes, 0).expect("bundled Segoe font should parse");
            assert!(
                !face.is_variable(),
                "runtime font must be a static instance"
            );

            for symbol in ['\u{2022}', '\u{20ac}', '\u{2192}', '\u{2500}'] {
                assert!(
                    face.glyph_index(symbol).is_some(),
                    "runtime font should contain {symbol:?}"
                );
            }
        }
    }

    #[test]
    fn snapshot_is_invariant_under_zoom_scroll_and_continuous_view() {
        let mut world = World::new();
        let mut state = EditorState::from_world(&mut world);
        state.document = Document::from_text("INT. ROOM - DAY\n\nALICE\nHello there.");
        state.document_format = DocumentFormat::Fountain;
        state.reparse();
        state.zoom = ZOOM_MIN;
        state.processed_top_visual = 7;
        state.processed_paginated = false;
        state.display_mode = DisplayMode::ProcessedRawCurrentLine;
        let first = build_pdf_export_snapshot(&mut state).expect("first snapshot");
        assert!(!state.processed_paginated);
        assert_eq!(state.display_mode, DisplayMode::ProcessedRawCurrentLine);

        state.zoom = ZOOM_MAX;
        state.processed_top_visual = 0;
        state.processed_paginated = true;
        state.display_mode = DisplayMode::Split;
        let second = build_pdf_export_snapshot(&mut state).expect("second snapshot");

        let signature = |snapshot: &PdfExportSnapshot| {
            snapshot
                .pages
                .iter()
                .flat_map(|page| page.lines.iter())
                .map(|line| (line.top_pt, line.visual.text.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(first.pages.len(), second.pages.len());
        assert_eq!(signature(&first), signature(&second));
        assert!(state.processed_paginated);
        assert_eq!(state.display_mode, DisplayMode::Split);
    }

    #[test]
    fn writes_parseable_searchable_a4_pdf() {
        let mut world = World::new();
        let mut state = EditorState::from_world(&mut world);
        state.document = Document::from_text("INT. ROOM - DAY\n\nALICE\nSearchable dialogue.");
        state.document_format = DocumentFormat::Fountain;
        state.reparse();
        let snapshot = build_pdf_export_snapshot(&mut state).expect("snapshot");
        let path = std::env::temp_dir().join(format!(
            "basscript-pdf-smoke-{}-{}.pdf",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let outcome = write_pdf_snapshot(&snapshot, &state, &path).expect("write PDF");
        let bytes = fs::read(&path).expect("read PDF");
        let _ = fs::remove_file(&path);

        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(outcome.page_count, 1);
        let mut parse_warnings = Vec::new();
        let parsed = PdfDocument::parse(
            &bytes,
            &printpdf::PdfParseOptions::default(),
            &mut parse_warnings,
        )
        .expect("parse generated PDF");
        assert_eq!(parsed.page_count(), 1);
        let media_box = &parsed.pages[0].media_box;
        assert!(
            (media_box.width.0 - A4_WIDTH_POINTS).abs() < 0.01,
            "width was {} instead of {}",
            media_box.width.0,
            A4_WIDTH_POINTS
        );
        assert!(
            (media_box.height.0 - A4_HEIGHT_POINTS).abs() < 0.01,
            "height was {} instead of {}",
            media_box.height.0,
            A4_HEIGHT_POINTS
        );
        assert!(
            parsed
                .extract_text()
                .iter()
                .flatten()
                .any(|text| text.contains("Searchable dialogue"))
        );
    }
}
