use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant, SystemTime},
};

use basscript_core::{
    CanvasDocument, CanvasNodeKind, Cursor, Document, DocumentFormat, DocumentPath, ImageEmbed,
    LineKind, LinkDisplayText, ParsedLine, Position, ScriptLink, StoryIndexDatabase,
    parse_canvas_document, parse_document_with_format, update_canvas_node_position,
    update_canvas_text_node_content,
};
use bevy::{
    app::AppExit,
    asset::{LoadState, RenderAssetUsages},
    image::{CompressedImageFormats, ImageSampler, ImageType},
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::{MouseScrollUnit, MouseWheel},
    },
    log::{info, warn},
    prelude::*,
    text::{ComputedTextBlock, FontStyle, FontWeight, LineHeight},
    ui::{RelativeCursorPosition, UiGlobalTransform, UiTransform, Val2},
    window::{PrimaryWindow, RawHandleWrapper},
};
use parley::{Affinity, Cursor as ParleyCursor};
use rfd::FileDialog;

mod core;
use core::*;

mod canvas;
use canvas::*;
mod story_index;
use story_index::*;
mod story_query_sheet;
use story_query_sheet::*;
mod status_line;
use status_line::*;
mod command_menu;
use command_menu::*;
mod autocomplete;
use autocomplete::*;
mod processed;
use processed::*;
mod formatting_marks;
use formatting_marks::*;
mod metadata_controls;
use metadata_controls::*;
mod caret;
use caret::*;
mod ui_setup;
use ui_setup::*;
mod splitters;
use splitters::*;
mod settings;
use settings::*;
mod linking;
use linking::*;
mod selection;
use selection::*;

#[path = "../pannels/text/explorer.rs"]
mod explorer;
use explorer::*;
#[path = "../pannels/text/explorer_actions.rs"]
mod explorer_actions;
use explorer_actions::*;
#[path = "../pannels/text/plain.rs"]
mod plain;
use plain::*;
#[path = "../pannels/text/processed.rs"]
mod processed_panel;
use processed_panel::*;

#[path = "../pannels/text/scrolling/modes/shared.rs"]
mod scrolling_shared;
use scrolling_shared::*;
#[path = "../pannels/text/scrolling/modes/wheel.rs"]
mod scrolling_wheel;
use scrolling_wheel::*;
#[path = "../pannels/text/scrolling/modes/ctrl_left_drag.rs"]
mod scrolling_ctrl_left_drag;
use scrolling_ctrl_left_drag::*;
#[path = "../pannels/text/scrolling/modes/middle_autoscroll.rs"]
mod scrolling_middle_autoscroll;
use scrolling_middle_autoscroll::*;

mod dialogs;
use dialogs::*;
mod clipboard;
use clipboard::*;
mod vim;
use vim::*;
mod editing;
use editing::*;
mod rendering;
use rendering::*;

pub use core::UiPlugin;
