#[derive(Component)]
struct CommandMenuRoot;

#[derive(Component)]
struct CommandMenuInputText;

#[derive(Component)]
struct CommandMenuHintText;

fn command_menu_bundle(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            bottom: px(0.0),
            width: percent(100.0),
            display: Display::None,
            padding: UiRect::axes(px(16.0), px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.92)),
        ZIndex(96),
        CommandMenuRoot,
        children![(
            Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            },
            children![
                (
                    Text::new(":"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.96, 0.97, 0.99)),
                    CommandMenuInputText,
                ),
                (
                    Text::new("Enter runs. Esc cancels. First command: w"),
                    TextFont {
                        font,
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.78, 0.80, 0.84, 0.92)),
                    CommandMenuHintText,
                ),
            ],
        )],
    )
}

fn sync_command_menu_ui(
    state: Res<EditorState>,
    mut root_query: Query<&mut Node, With<CommandMenuRoot>>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<CommandMenuInputText>>,
        Query<&mut Text, With<CommandMenuHintText>>,
    )>,
) {
    let Some(command_menu) = state.command_menu.as_ref() else {
        if let Ok(mut root) = root_query.single_mut() {
            root.display = Display::None;
        }
        return;
    };

    if let Ok(mut root) = root_query.single_mut() {
        root.display = Display::Flex;
    }
    if let Ok(mut input) = text_queries.p0().single_mut() {
        **input = format!(":{}_", command_menu.input);
    }
    if let Ok(mut hint) = text_queries.p1().single_mut() {
        **hint = "Enter runs. Esc cancels. First command: w".to_string();
    }
}

fn handle_command_menu_open_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    if state.command_menu.is_some()
        || state.workspace_prompt.is_some()
        || state.workspace_focused
    {
        return;
    }

    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }
        if keys.pressed(KeyCode::Space) && keyboard_input_text_is(input, ":") {
            state.open_command_menu();
            state.pending_space_insert = false;
            state.pending_space_combo_canceled = true;
            break;
        }
    }
}

fn handle_command_menu_input(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
) {
    let keybinds = state.keybinds.clone();
    let Some(command_menu) = state.command_menu.as_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        state.command_menu = None;
        state.status_message = "Command canceled.".to_string();
        return;
    }

    if paste_shortcut_just_pressed(&keys) {
        let status_message;
        if let Some(text) = read_system_clipboard_text() {
            command_menu.input.push_str(&text.replace('\n', " "));
            status_message = "Pasted clipboard into command.".to_string();
        } else {
            status_message = "Clipboard is empty or unavailable.".to_string();
        }
        state.status_message = status_message;
        for _ in keyboard_inputs.read() {}
        return;
    }

    enum CommandMenuAction {
        Run(String),
    }

    let mut action = None::<CommandMenuAction>;
    for input in keyboard_inputs.read() {
        if !input.state.is_pressed() {
            continue;
        }

        match &input.logical_key {
            Key::Enter => {
                action = Some(CommandMenuAction::Run(command_menu.input.clone()));
                break;
            }
            Key::Backspace => {
                command_menu.input.pop();
            }
            Key::Delete => {}
            _ if text_input_should_skip_for_shortcut(&keys, input, &keybinds) => {}
            _ => {
                if let Some(inserted_text) = &input.text {
                    if !inserted_text.is_empty() && inserted_text.chars().all(is_printable_char) {
                        command_menu.input.push_str(inserted_text);
                    }
                }
            }
        }
    }

    if let Some(CommandMenuAction::Run(input)) = action {
        state.command_menu = None;
        run_command_menu_command(&mut state, input.trim());
    }
}

fn run_command_menu_command(state: &mut EditorState, command: &str) {
    match command {
        "w" => state.save_current(),
        "" => state.status_message = "No command entered.".to_string(),
        other => state.status_message = format!("Unknown command: {other}"),
    }
}

fn keyboard_input_text_is(input: &KeyboardInput, expected: &str) -> bool {
    input.text.as_ref().is_some_and(|text| text == expected)
}

impl EditorState {
    fn open_command_menu(&mut self) {
        self.command_menu = Some(CommandMenu {
            input: String::new(),
        });
        self.vim_pending_operator = None;
        self.status_message = "Command menu.".to_string();
    }
}
