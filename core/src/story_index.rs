use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::links::EntityDocument;
use rusqlite::{Connection, OptionalExtension, params};

mod appearance_records;
mod query;
mod scene_records;
use appearance_records::build_appearance_index;
pub use appearance_records::{StoryIndexAppearanceRecord, StoryIndexAppearanceRole};
pub use query::{StoryIndexEntityRecord, StoryIndexPlaceVisit};
pub use scene_records::StoryIndexSceneRecord;
use scene_records::build_scene_index;

pub const STORY_INDEX_SCHEMA_VERSION: i64 = 5;
pub const STORY_INDEX_DIR_NAME: &str = ".basscript";
pub const STORY_INDEX_DATABASE_NAME: &str = "story-index.sqlite3";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexDatabase {
    workspace_root: PathBuf,
    database_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexOpenReport {
    pub database: StoryIndexDatabase,
    pub status: StoryIndexOpenStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryIndexOpenStatus {
    Created,
    Ready,
    Recreated {
        reason: StoryIndexRecoveryReason,
        previous_database_path: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoryIndexRecoveryReason {
    CorruptOrUnreadable(String),
    IncompatibleSchemaVersion { found: i64, expected: i64 },
}

#[derive(Debug)]
pub enum StoryIndexError {
    Io(io::Error),
    Sqlite(rusqlite::Error),
    PathOutsideWorkspace {
        workspace_root: PathBuf,
        path: PathBuf,
    },
    IncompatibleSchemaVersion {
        found: i64,
        expected: i64,
    },
    RecoveryFailed {
        original_error: String,
        recovery_error: io::Error,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexedFileKind {
    Fountain,
    Markdown,
    Canvas,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexScanReport {
    pub database_path: PathBuf,
    pub workspace_root: PathBuf,
    pub file_count: usize,
    pub inserted_count: usize,
    pub updated_count: usize,
    pub removed_count: usize,
    pub entity_count: usize,
    pub entity_alias_count: usize,
    pub entity_error_count: usize,
    pub duplicate_entity_target_count: usize,
    pub scene_count: usize,
    pub appearance_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndexedWorkspaceFile {
    path: PathBuf,
    relative_path: String,
    kind: IndexedFileKind,
    modified_unix_millis: Option<i64>,
    byte_len: u64,
    content_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredIndexedFile {
    relative_path: String,
    kind: IndexedFileKind,
    modified_unix_millis: Option<i64>,
    byte_len: u64,
    content_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndexedEntityRecord {
    document: EntityDocument,
    relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntityIndexErrorRecord {
    path: PathBuf,
    relative_path: String,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntityIndexBuild {
    entities: Vec<IndexedEntityRecord>,
    errors: Vec<EntityIndexErrorRecord>,
    alias_count: usize,
    duplicate_target_count: usize,
}

impl StoryIndexDatabase {
    pub fn open_workspace(
        workspace_root: impl AsRef<Path>,
    ) -> Result<StoryIndexOpenReport, StoryIndexError> {
        let workspace_root = normalize_workspace_root(workspace_root.as_ref());
        let database_path = story_index_database_path(&workspace_root);
        let existed_before_open = database_path.exists();

        match initialize_database(&workspace_root, &database_path) {
            Ok(()) => {
                let status = if existed_before_open {
                    StoryIndexOpenStatus::Ready
                } else {
                    StoryIndexOpenStatus::Created
                };
                Ok(StoryIndexOpenReport {
                    database: Self {
                        workspace_root,
                        database_path,
                    },
                    status,
                })
            }
            Err(error) if existed_before_open => {
                let reason = recovery_reason_from_error(&error);
                let previous_database_path = quarantine_database(&database_path, &error)?;
                initialize_database(&workspace_root, &database_path)?;
                Ok(StoryIndexOpenReport {
                    database: Self {
                        workspace_root,
                        database_path,
                    },
                    status: StoryIndexOpenStatus::Recreated {
                        reason,
                        previous_database_path: Some(previous_database_path),
                    },
                })
            }
            Err(error) => Err(error),
        }
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn scan_workspace_files(&self) -> Result<StoryIndexScanReport, StoryIndexError> {
        initialize_database(&self.workspace_root, &self.database_path)?;
        let files = collect_indexable_workspace_files(&self.workspace_root)?;
        let entity_index = build_entity_index(&files)?;
        let scene_index = build_scene_index(&files)?;
        let appearance_index = build_appearance_index(&files, &scene_index, &entity_index)?;
        let mut connection = Connection::open(&self.database_path)?;
        let previous = load_stored_indexed_files(&connection)?;
        let now = current_unix_seconds() as i64;
        let mut seen_paths = BTreeSet::<String>::new();
        let mut inserted_count = 0usize;
        let mut updated_count = 0usize;

        let transaction = connection.transaction()?;
        for file in &files {
            let path_key = file.path.to_string_lossy().to_string();
            seen_paths.insert(path_key.clone());
            let stored = StoredIndexedFile {
                relative_path: file.relative_path.clone(),
                kind: file.kind,
                modified_unix_millis: file.modified_unix_millis,
                byte_len: file.byte_len,
                content_hash: file.content_hash.clone(),
            };

            match previous.get(&path_key) {
                None => {
                    inserted_count = inserted_count.saturating_add(1);
                    upsert_indexed_file(&transaction, &path_key, &stored, now)?;
                }
                Some(existing) if existing != &stored => {
                    updated_count = updated_count.saturating_add(1);
                    upsert_indexed_file(&transaction, &path_key, &stored, now)?;
                }
                Some(_) => {}
            }
        }

        let mut removed_count = 0usize;
        for previous_path in previous.keys() {
            if seen_paths.contains(previous_path) {
                continue;
            }
            removed_count = removed_count.saturating_add(1);
            transaction.execute(
                "DELETE FROM story_index_files WHERE path = ?1;",
                params![previous_path],
            )?;
        }

        replace_entity_index(&transaction, &entity_index, now)?;
        replace_scene_index(&transaction, &scene_index, now)?;
        replace_appearance_index(&transaction, &appearance_index, now)?;
        transaction.execute(
            "INSERT OR REPLACE INTO story_index_meta (key, value) VALUES ('last_file_scan_at_unix_seconds', ?1);",
            params![now.to_string()],
        )?;
        transaction.execute(
            "INSERT OR REPLACE INTO story_index_meta (key, value) VALUES ('last_entity_scan_at_unix_seconds', ?1);",
            params![now.to_string()],
        )?;
        transaction.execute(
            "INSERT OR REPLACE INTO story_index_meta (key, value) VALUES ('last_scene_scan_at_unix_seconds', ?1);",
            params![now.to_string()],
        )?;
        transaction.execute(
            "INSERT OR REPLACE INTO story_index_meta (key, value) VALUES ('last_appearance_scan_at_unix_seconds', ?1);",
            params![now.to_string()],
        )?;
        transaction.commit()?;

        Ok(StoryIndexScanReport {
            database_path: self.database_path.clone(),
            workspace_root: self.workspace_root.clone(),
            file_count: files.len(),
            inserted_count,
            updated_count,
            removed_count,
            entity_count: entity_index.entities.len(),
            entity_alias_count: entity_index.alias_count,
            entity_error_count: entity_index.errors.len(),
            duplicate_entity_target_count: entity_index.duplicate_target_count,
            scene_count: scene_index.len(),
            appearance_count: appearance_index.len(),
        })
    }

    pub fn scenes_for_file(
        &self,
        source_path: impl AsRef<Path>,
    ) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError> {
        initialize_database(&self.workspace_root, &self.database_path)?;
        let source_path = normalize_workspace_file_path(&self.workspace_root, source_path.as_ref());
        let connection = Connection::open(&self.database_path)?;
        let mut statement = connection.prepare(
            "SELECT scene_key, source_path, relative_path, scene_ordinal, heading_text,
                    normalized_heading, start_line, end_line, script_order, location_text,
                    time_of_day_text
             FROM story_index_scenes
             WHERE source_path = ?1
             ORDER BY scene_ordinal;",
        )?;
        let rows = statement
            .query_map(params![source_path.to_string_lossy().to_string()], |row| {
                scene_record_from_row(row)
            })?;

        let mut scenes = Vec::<StoryIndexSceneRecord>::new();
        for row in rows {
            scenes.push(row?);
        }
        Ok(scenes)
    }

    pub fn scene_at_line(
        &self,
        source_path: impl AsRef<Path>,
        line: usize,
    ) -> Result<Option<StoryIndexSceneRecord>, StoryIndexError> {
        initialize_database(&self.workspace_root, &self.database_path)?;
        let source_path = normalize_workspace_file_path(&self.workspace_root, source_path.as_ref());
        let connection = Connection::open(&self.database_path)?;
        connection
            .query_row(
                "SELECT scene_key, source_path, relative_path, scene_ordinal, heading_text,
                        normalized_heading, start_line, end_line, script_order, location_text,
                        time_of_day_text
                 FROM story_index_scenes
                 WHERE source_path = ?1 AND start_line <= ?2 AND end_line >= ?2
                 ORDER BY scene_ordinal
                 LIMIT 1;",
                params![source_path.to_string_lossy().to_string(), line as i64],
                scene_record_from_row,
            )
            .optional()
            .map_err(StoryIndexError::Sqlite)
    }
}

pub fn story_index_database_path(workspace_root: impl AsRef<Path>) -> PathBuf {
    workspace_root
        .as_ref()
        .join(STORY_INDEX_DIR_NAME)
        .join(STORY_INDEX_DATABASE_NAME)
}

fn normalize_workspace_root(workspace_root: &Path) -> PathBuf {
    workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf())
}

fn normalize_workspace_file_path(workspace_root: &Path, path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };
    path.canonicalize().unwrap_or(path)
}

fn initialize_database(workspace_root: &Path, database_path: &Path) -> Result<(), StoryIndexError> {
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut connection = Connection::open(database_path)?;
    let user_version = sqlite_user_version(&connection)?;
    if user_version != 0 && user_version != STORY_INDEX_SCHEMA_VERSION {
        return Err(StoryIndexError::IncompatibleSchemaVersion {
            found: user_version,
            expected: STORY_INDEX_SCHEMA_VERSION,
        });
    }

    let now = current_unix_seconds();
    let workspace_root = workspace_root.to_string_lossy().to_string();
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS story_index_meta (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS story_index_files (
            path TEXT PRIMARY KEY NOT NULL,
            relative_path TEXT NOT NULL,
            kind TEXT NOT NULL,
            modified_unix_millis INTEGER,
            byte_len INTEGER NOT NULL,
            content_hash TEXT NOT NULL,
            indexed_at_unix_seconds INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS story_index_files_kind_idx
            ON story_index_files (kind);
        CREATE INDEX IF NOT EXISTS story_index_files_relative_path_idx
            ON story_index_files (relative_path);
        CREATE TABLE IF NOT EXISTS story_index_entities (
            target TEXT PRIMARY KEY NOT NULL,
            id TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            name TEXT NOT NULL,
            status TEXT,
            path TEXT NOT NULL UNIQUE,
            relative_path TEXT NOT NULL,
            source_file_path TEXT NOT NULL,
            indexed_at_unix_seconds INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS story_index_entities_type_idx
            ON story_index_entities (entity_type);
        CREATE INDEX IF NOT EXISTS story_index_entities_name_idx
            ON story_index_entities (name);
        CREATE INDEX IF NOT EXISTS story_index_entities_relative_path_idx
            ON story_index_entities (relative_path);
        CREATE TABLE IF NOT EXISTS story_index_entity_aliases (
            target TEXT NOT NULL,
            alias TEXT NOT NULL,
            normalized_alias TEXT NOT NULL,
            alias_source TEXT NOT NULL,
            indexed_at_unix_seconds INTEGER NOT NULL,
            PRIMARY KEY (target, alias, alias_source)
        );
        CREATE INDEX IF NOT EXISTS story_index_entity_aliases_lookup_idx
            ON story_index_entity_aliases (normalized_alias);
        CREATE INDEX IF NOT EXISTS story_index_entity_aliases_target_idx
            ON story_index_entity_aliases (target);
        CREATE TABLE IF NOT EXISTS story_index_entity_errors (
            path TEXT PRIMARY KEY NOT NULL,
            relative_path TEXT NOT NULL,
            message TEXT NOT NULL,
            indexed_at_unix_seconds INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS story_index_scenes (
            scene_key TEXT PRIMARY KEY NOT NULL,
            source_path TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            scene_ordinal INTEGER NOT NULL,
            heading_text TEXT NOT NULL,
            normalized_heading TEXT NOT NULL,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            script_order INTEGER NOT NULL,
            location_text TEXT,
            time_of_day_text TEXT,
            indexed_at_unix_seconds INTEGER NOT NULL,
            UNIQUE(source_path, scene_ordinal)
        );
        CREATE INDEX IF NOT EXISTS story_index_scenes_source_path_idx
            ON story_index_scenes (source_path);
        CREATE INDEX IF NOT EXISTS story_index_scenes_normalized_heading_idx
            ON story_index_scenes (normalized_heading);
        CREATE INDEX IF NOT EXISTS story_index_scenes_script_order_idx
            ON story_index_scenes (script_order);
        CREATE TABLE IF NOT EXISTS story_index_appearances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            target TEXT NOT NULL,
            entity_type TEXT,
            entity_name TEXT,
            source_path TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            scene_key TEXT,
            line INTEGER NOT NULL,
            column INTEGER NOT NULL,
            line_kind TEXT NOT NULL,
            appearance_role TEXT NOT NULL,
            raw_snippet TEXT NOT NULL,
            indexed_at_unix_seconds INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS story_index_appearances_target_idx
            ON story_index_appearances (target);
        CREATE INDEX IF NOT EXISTS story_index_appearances_scene_key_idx
            ON story_index_appearances (scene_key);
        CREATE INDEX IF NOT EXISTS story_index_appearances_role_idx
            ON story_index_appearances (appearance_role);
        CREATE INDEX IF NOT EXISTS story_index_appearances_source_line_idx
            ON story_index_appearances (source_path, line);",
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO story_index_meta (key, value) VALUES ('schema_version', ?1);",
        params![STORY_INDEX_SCHEMA_VERSION.to_string()],
    )?;
    transaction.execute(
        "UPDATE story_index_meta SET value = ?1 WHERE key = 'schema_version';",
        params![STORY_INDEX_SCHEMA_VERSION.to_string()],
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO story_index_meta (key, value) VALUES ('workspace_root', ?1);",
        params![workspace_root],
    )?;
    transaction.execute(
        "UPDATE story_index_meta SET value = ?1 WHERE key = 'workspace_root';",
        params![workspace_root],
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO story_index_meta (key, value) VALUES ('created_at_unix_seconds', ?1);",
        params![now.to_string()],
    )?;
    transaction.execute(
        "INSERT OR REPLACE INTO story_index_meta (key, value) VALUES ('last_opened_at_unix_seconds', ?1);",
        params![now.to_string()],
    )?;
    transaction.pragma_update(None, "user_version", STORY_INDEX_SCHEMA_VERSION)?;
    transaction.commit()?;

    Ok(())
}

fn collect_indexable_workspace_files(
    workspace_root: &Path,
) -> Result<Vec<IndexedWorkspaceFile>, StoryIndexError> {
    let mut files = Vec::<IndexedWorkspaceFile>::new();
    let mut stack = vec![workspace_root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                if should_skip_story_index_dir(&path) {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let Some(kind) = IndexedFileKind::from_path(&path) else {
                continue;
            };
            files.push(index_workspace_file(workspace_root, path, kind)?);
        }
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn build_entity_index(files: &[IndexedWorkspaceFile]) -> Result<EntityIndexBuild, StoryIndexError> {
    let mut parsed = Vec::<IndexedEntityRecord>::new();
    let mut errors = Vec::<EntityIndexErrorRecord>::new();

    for file in files
        .iter()
        .filter(|file| file.kind == IndexedFileKind::Markdown)
    {
        let markdown = fs::read_to_string(&file.path)?;
        if !looks_like_yaml_front_matter(&markdown) {
            continue;
        }

        match EntityDocument::from_markdown_for_index(&file.path, &markdown) {
            Ok(document) => parsed.push(IndexedEntityRecord {
                document,
                relative_path: file.relative_path.clone(),
            }),
            Err(error) => errors.push(EntityIndexErrorRecord {
                path: file.path.clone(),
                relative_path: file.relative_path.clone(),
                message: error.to_string(),
            }),
        }
    }

    let mut by_target = BTreeMap::<String, Vec<IndexedEntityRecord>>::new();
    for record in parsed {
        by_target
            .entry(record.document.metadata.target.clone())
            .or_default()
            .push(record);
    }

    let mut entities = Vec::<IndexedEntityRecord>::new();
    let mut duplicate_target_count = 0usize;
    for (target, mut records) in by_target {
        records.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        if records.len() == 1 {
            entities.push(records.remove(0));
            continue;
        }

        duplicate_target_count = duplicate_target_count.saturating_add(1);
        let paths = records
            .iter()
            .map(|record| record.relative_path.clone())
            .collect::<Vec<_>>()
            .join(", ");
        for record in records {
            errors.push(EntityIndexErrorRecord {
                path: record.document.path.clone(),
                relative_path: record.relative_path,
                message: format!("Duplicate entity target `{target}` also declared by {paths}."),
            });
        }
    }

    let alias_count = entities
        .iter()
        .map(|entity| entity_alias_terms(entity).len())
        .sum();
    entities.sort_by(|left, right| {
        left.document
            .metadata
            .target
            .cmp(&right.document.metadata.target)
    });
    errors.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(EntityIndexBuild {
        entities,
        errors,
        alias_count,
        duplicate_target_count,
    })
}

fn looks_like_yaml_front_matter(markdown: &str) -> bool {
    markdown
        .lines()
        .next()
        .map(|line| line.trim_end_matches('\r') == "---")
        .unwrap_or(false)
}

fn index_workspace_file(
    workspace_root: &Path,
    path: PathBuf,
    kind: IndexedFileKind,
) -> Result<IndexedWorkspaceFile, StoryIndexError> {
    let metadata = fs::metadata(&path)?;
    let bytes = fs::read(&path)?;
    let relative_path = path
        .strip_prefix(workspace_root)
        .map_err(|_| StoryIndexError::PathOutsideWorkspace {
            workspace_root: workspace_root.to_path_buf(),
            path: path.clone(),
        })?
        .to_string_lossy()
        .replace('\\', "/");

    Ok(IndexedWorkspaceFile {
        path,
        relative_path,
        kind,
        modified_unix_millis: metadata
            .modified()
            .ok()
            .and_then(system_time_to_unix_millis),
        byte_len: metadata.len(),
        content_hash: stable_content_hash(&bytes),
    })
}

fn load_stored_indexed_files(
    connection: &Connection,
) -> Result<BTreeMap<String, StoredIndexedFile>, StoryIndexError> {
    let mut statement = connection.prepare(
        "SELECT path, relative_path, kind, modified_unix_millis, byte_len, content_hash
         FROM story_index_files;",
    )?;
    let rows = statement.query_map([], |row| {
        let kind_text: String = row.get(2)?;
        Ok((
            row.get::<_, String>(0)?,
            StoredIndexedFile {
                relative_path: row.get(1)?,
                kind: IndexedFileKind::from_database_value(&kind_text)
                    .unwrap_or(IndexedFileKind::Fountain),
                modified_unix_millis: row.get(3)?,
                byte_len: row.get::<_, i64>(4)?.max(0) as u64,
                content_hash: row.get(5)?,
            },
        ))
    })?;

    let mut files = BTreeMap::<String, StoredIndexedFile>::new();
    for row in rows {
        let (path, file) = row?;
        files.insert(path, file);
    }
    Ok(files)
}

fn upsert_indexed_file(
    connection: &Connection,
    path: &str,
    file: &StoredIndexedFile,
    indexed_at_unix_seconds: i64,
) -> Result<(), StoryIndexError> {
    connection.execute(
        "INSERT OR REPLACE INTO story_index_files (
            path,
            relative_path,
            kind,
            modified_unix_millis,
            byte_len,
            content_hash,
            indexed_at_unix_seconds
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
        params![
            path,
            file.relative_path,
            file.kind.as_database_value(),
            file.modified_unix_millis,
            file.byte_len as i64,
            file.content_hash,
            indexed_at_unix_seconds,
        ],
    )?;
    Ok(())
}

fn replace_entity_index(
    connection: &Connection,
    entity_index: &EntityIndexBuild,
    indexed_at_unix_seconds: i64,
) -> Result<(), StoryIndexError> {
    connection.execute("DELETE FROM story_index_entity_aliases;", [])?;
    connection.execute("DELETE FROM story_index_entities;", [])?;
    connection.execute("DELETE FROM story_index_entity_errors;", [])?;

    for entity in &entity_index.entities {
        insert_entity_record(connection, entity, indexed_at_unix_seconds)?;
        for alias in entity_alias_terms(entity) {
            connection.execute(
                "INSERT OR IGNORE INTO story_index_entity_aliases (
                    target,
                    alias,
                    normalized_alias,
                    alias_source,
                    indexed_at_unix_seconds
                ) VALUES (?1, ?2, ?3, ?4, ?5);",
                params![
                    entity.document.metadata.target.as_str(),
                    alias.text.as_str(),
                    normalize_lookup_text(&alias.text),
                    alias.source,
                    indexed_at_unix_seconds,
                ],
            )?;
        }
    }

    for error in &entity_index.errors {
        connection.execute(
            "INSERT OR REPLACE INTO story_index_entity_errors (
                path,
                relative_path,
                message,
                indexed_at_unix_seconds
        ) VALUES (?1, ?2, ?3, ?4);",
            params![
                error.path.to_string_lossy().to_string(),
                error.relative_path.as_str(),
                error.message.as_str(),
                indexed_at_unix_seconds,
            ],
        )?;
    }

    Ok(())
}

fn replace_scene_index(
    connection: &Connection,
    scenes: &[StoryIndexSceneRecord],
    indexed_at_unix_seconds: i64,
) -> Result<(), StoryIndexError> {
    connection.execute("DELETE FROM story_index_scenes;", [])?;

    for scene in scenes {
        connection.execute(
            "INSERT OR REPLACE INTO story_index_scenes (
                scene_key,
                source_path,
                relative_path,
                scene_ordinal,
                heading_text,
                normalized_heading,
                start_line,
                end_line,
                script_order,
                location_text,
                time_of_day_text,
                indexed_at_unix_seconds
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);",
            params![
                scene.scene_key.as_str(),
                scene.source_path.to_string_lossy().to_string(),
                scene.relative_path.as_str(),
                scene.scene_ordinal as i64,
                scene.heading_text.as_str(),
                scene.normalized_heading.as_str(),
                scene.start_line as i64,
                scene.end_line as i64,
                scene.script_order as i64,
                scene.location_text.as_deref(),
                scene.time_of_day_text.as_deref(),
                indexed_at_unix_seconds,
            ],
        )?;
    }

    Ok(())
}

fn scene_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoryIndexSceneRecord> {
    Ok(StoryIndexSceneRecord {
        scene_key: row.get(0)?,
        source_path: PathBuf::from(row.get::<_, String>(1)?),
        relative_path: row.get(2)?,
        scene_ordinal: row.get::<_, i64>(3)?.max(0) as usize,
        heading_text: row.get(4)?,
        normalized_heading: row.get(5)?,
        start_line: row.get::<_, i64>(6)?.max(0) as usize,
        end_line: row.get::<_, i64>(7)?.max(0) as usize,
        script_order: row.get::<_, i64>(8)?.max(0) as usize,
        location_text: row.get(9)?,
        time_of_day_text: row.get(10)?,
    })
}

fn replace_appearance_index(
    connection: &Connection,
    appearances: &[StoryIndexAppearanceRecord],
    indexed_at_unix_seconds: i64,
) -> Result<(), StoryIndexError> {
    connection.execute("DELETE FROM story_index_appearances;", [])?;

    for appearance in appearances {
        connection.execute(
            "INSERT INTO story_index_appearances (
                target,
                entity_type,
                entity_name,
                source_path,
                relative_path,
                scene_key,
                line,
                column,
                line_kind,
                appearance_role,
                raw_snippet,
                indexed_at_unix_seconds
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);",
            params![
                appearance.target.as_str(),
                appearance.entity_type.as_deref(),
                appearance.entity_name.as_deref(),
                appearance.source_path.to_string_lossy().to_string(),
                appearance.relative_path.as_str(),
                appearance.scene_key.as_deref(),
                appearance.line as i64,
                appearance.column as i64,
                appearance.line_kind.as_str(),
                appearance.role.as_database_value(),
                appearance.raw_snippet.as_str(),
                indexed_at_unix_seconds,
            ],
        )?;
    }

    Ok(())
}

fn appearance_record_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StoryIndexAppearanceRecord> {
    let role_text: String = row.get(10)?;
    Ok(StoryIndexAppearanceRecord {
        target: row.get(0)?,
        entity_type: row.get(1)?,
        entity_name: row.get(2)?,
        source_path: PathBuf::from(row.get::<_, String>(3)?),
        relative_path: row.get(4)?,
        scene_key: row.get(5)?,
        line: row.get::<_, i64>(6)?.max(0) as usize,
        column: row.get::<_, i64>(7)?.max(0) as usize,
        line_kind: row.get(8)?,
        role: StoryIndexAppearanceRole::from_database_value(&role_text)
            .unwrap_or(StoryIndexAppearanceRole::ActionMention),
        raw_snippet: row.get(9)?,
    })
}

fn insert_entity_record(
    connection: &Connection,
    entity: &IndexedEntityRecord,
    indexed_at_unix_seconds: i64,
) -> Result<(), StoryIndexError> {
    connection.execute(
        "INSERT OR REPLACE INTO story_index_entities (
            target,
            id,
            entity_type,
            name,
            status,
            path,
            relative_path,
            source_file_path,
            indexed_at_unix_seconds
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            entity.document.metadata.target.as_str(),
            entity.document.metadata.id.as_str(),
            entity.document.metadata.entity_type.as_str(),
            entity.document.metadata.name.as_str(),
            entity.document.metadata.status.as_deref(),
            entity.document.path.to_string_lossy().to_string(),
            entity.relative_path.as_str(),
            entity.document.path.to_string_lossy().to_string(),
            indexed_at_unix_seconds,
        ],
    )?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EntityAliasTerm {
    text: String,
    source: &'static str,
}

fn entity_alias_terms(entity: &IndexedEntityRecord) -> Vec<EntityAliasTerm> {
    let mut terms = BTreeSet::<EntityAliasTerm>::new();
    terms.insert(EntityAliasTerm {
        text: entity.document.metadata.target.clone(),
        source: "target",
    });
    terms.insert(EntityAliasTerm {
        text: entity.document.metadata.name.clone(),
        source: "name",
    });
    for alias in &entity.document.metadata.aliases {
        terms.insert(EntityAliasTerm {
            text: alias.clone(),
            source: "alias",
        });
    }
    terms.into_iter().collect()
}

fn normalize_lookup_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn should_skip_story_index_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.starts_with('.') || matches!(name, "target" | "node_modules")
}

fn stable_content_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn system_time_to_unix_millis(time: SystemTime) -> Option<i64> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(duration.as_millis()).ok()
}

fn sqlite_user_version(connection: &Connection) -> Result<i64, StoryIndexError> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(StoryIndexError::Sqlite)
}

fn recovery_reason_from_error(error: &StoryIndexError) -> StoryIndexRecoveryReason {
    match error {
        StoryIndexError::IncompatibleSchemaVersion { found, expected } => {
            StoryIndexRecoveryReason::IncompatibleSchemaVersion {
                found: *found,
                expected: *expected,
            }
        }
        other => StoryIndexRecoveryReason::CorruptOrUnreadable(other.to_string()),
    }
}

fn quarantine_database(
    database_path: &Path,
    error: &StoryIndexError,
) -> Result<PathBuf, StoryIndexError> {
    let quarantined_path = database_path.with_file_name(format!(
        "story-index.corrupt-{}.sqlite3",
        current_unix_nanos()
    ));
    fs::rename(database_path, &quarantined_path).map_err(|recovery_error| {
        StoryIndexError::RecoveryFailed {
            original_error: error.to_string(),
            recovery_error,
        }
    })?;
    Ok(quarantined_path)
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn current_unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

impl fmt::Display for StoryIndexOpenStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Ready => write!(f, "ready"),
            Self::Recreated { reason, .. } => write!(f, "recreated after {reason}"),
        }
    }
}

impl fmt::Display for StoryIndexRecoveryReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CorruptOrUnreadable(error) => write!(f, "unreadable database: {error}"),
            Self::IncompatibleSchemaVersion { found, expected } => {
                write!(f, "schema version {found}, expected {expected}")
            }
        }
    }
}

impl fmt::Display for StoryIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Sqlite(error) => write!(f, "{error}"),
            Self::PathOutsideWorkspace {
                workspace_root,
                path,
            } => write!(
                f,
                "{} is outside workspace {}",
                path.display(),
                workspace_root.display()
            ),
            Self::IncompatibleSchemaVersion { found, expected } => {
                write!(f, "story index schema version {found}, expected {expected}")
            }
            Self::RecoveryFailed {
                original_error,
                recovery_error,
            } => write!(
                f,
                "could not quarantine invalid story index ({original_error}): {recovery_error}"
            ),
        }
    }
}

impl Error for StoryIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::RecoveryFailed { recovery_error, .. } => Some(recovery_error),
            Self::PathOutsideWorkspace { .. } | Self::IncompatibleSchemaVersion { .. } => None,
        }
    }
}

impl IndexedFileKind {
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase());
        match extension.as_deref() {
            Some("fountain") => Some(Self::Fountain),
            Some("md") | Some("markdown") => Some(Self::Markdown),
            Some("canvas") => Some(Self::Canvas),
            _ => None,
        }
    }

    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Fountain => "fountain",
            Self::Markdown => "markdown",
            Self::Canvas => "canvas",
        }
    }

    fn from_database_value(value: &str) -> Option<Self> {
        match value {
            "fountain" => Some(Self::Fountain),
            "markdown" => Some(Self::Markdown),
            "canvas" => Some(Self::Canvas),
            _ => None,
        }
    }
}

impl From<io::Error> for StoryIndexError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for StoryIndexError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEST_DIR_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique_id = NEXT_TEST_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = env::temp_dir().join(format!(
                "basscript-story-index-{}-{}-{}",
                std::process::id(),
                current_unix_nanos(),
                unique_id
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write(&self, relative: &str, contents: impl AsRef<[u8]>) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::write(path, contents).expect("write file");
        }

        fn remove(&self, relative: &str) {
            fs::remove_file(self.path.join(relative)).expect("remove file");
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn creates_story_index_database_for_workspace() {
        let root = TestDir::new();

        let report = StoryIndexDatabase::open_workspace(root.path()).expect("open story index");

        assert_eq!(report.status, StoryIndexOpenStatus::Created);
        assert_eq!(
            report.database.database_path(),
            &story_index_database_path(report.database.workspace_root())
        );
        assert!(report.database.database_path().exists());
    }

    #[test]
    fn reuses_existing_story_index_database() {
        let root = TestDir::new();

        StoryIndexDatabase::open_workspace(root.path()).expect("first open");
        let report = StoryIndexDatabase::open_workspace(root.path()).expect("second open");

        assert_eq!(report.status, StoryIndexOpenStatus::Ready);
    }

    #[test]
    fn recreates_corrupt_story_index_database_without_deleting_it() {
        let root = TestDir::new();
        let path = story_index_database_path(root.path());
        fs::create_dir_all(path.parent().expect("database parent")).expect("create db parent");
        fs::write(&path, "not sqlite").expect("write corrupt database");

        let report = StoryIndexDatabase::open_workspace(root.path()).expect("recover database");

        match report.status {
            StoryIndexOpenStatus::Recreated {
                reason: StoryIndexRecoveryReason::CorruptOrUnreadable(_),
                previous_database_path: Some(previous_path),
            } => {
                assert!(previous_path.exists());
                assert!(report.database.database_path().exists());
            }
            other => panic!("unexpected status: {other:?}"),
        }
    }

    #[test]
    fn recreates_incompatible_schema_database() {
        let root = TestDir::new();
        let path = story_index_database_path(root.path());
        fs::create_dir_all(path.parent().expect("database parent")).expect("create db parent");
        let connection = Connection::open(&path).expect("open database");
        connection
            .pragma_update(None, "user_version", STORY_INDEX_SCHEMA_VERSION + 100)
            .expect("set user version");
        drop(connection);

        let report = StoryIndexDatabase::open_workspace(root.path()).expect("recover database");

        match report.status {
            StoryIndexOpenStatus::Recreated {
                reason: StoryIndexRecoveryReason::IncompatibleSchemaVersion { found, expected },
                previous_database_path: Some(previous_path),
            } => {
                assert_eq!(found, STORY_INDEX_SCHEMA_VERSION + 100);
                assert_eq!(expected, STORY_INDEX_SCHEMA_VERSION);
                assert!(previous_path.exists());
                assert!(report.database.database_path().exists());
            }
            other => panic!("unexpected status: {other:?}"),
        }
    }

    #[test]
    fn scans_supported_workspace_files_into_database() {
        let root = TestDir::new();
        root.write("script.fountain", "INT. KITCHEN - DAY\n");
        root.write("notes.md", "---\ntarget: notes\n---\n");
        root.write("board.canvas", "{}");
        root.write("ignored.txt", "not indexed by story index");
        root.write(
            ".basscript/ignored.md",
            "generated db dir should be ignored",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");

        assert_eq!(report.file_count, 3);
        assert_eq!(report.inserted_count, 3);
        assert_eq!(report.updated_count, 0);
        assert_eq!(report.removed_count, 0);

        let connection = Connection::open(database.database_path()).expect("open database");
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM story_index_files;", [], |row| {
                row.get(0)
            })
            .expect("count indexed files");
        assert_eq!(count, 3);
    }

    #[test]
    fn scan_reports_updated_and_removed_files() {
        let root = TestDir::new();
        root.write("script.fountain", "INT. KITCHEN - DAY\n");
        root.write("notes.md", "old");
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let first = database.scan_workspace_files().expect("first scan");
        assert_eq!(first.inserted_count, 2);

        root.write("notes.md", "new");
        root.remove("script.fountain");
        let second = database.scan_workspace_files().expect("second scan");

        assert_eq!(second.file_count, 1);
        assert_eq!(second.inserted_count, 0);
        assert_eq!(second.updated_count, 1);
        assert_eq!(second.removed_count, 1);
    }

    #[test]
    fn indexes_entity_records_and_aliases_from_markdown_front_matter() {
        let root = TestDir::new();
        root.write(
            "characters/eoghan.md",
            entity_markdown(
                "entity_eoghan_001",
                "eoghan",
                "character",
                "Eoghan",
                &["Eo", "EOG"],
            ),
        );
        root.write(
            "items/ember-sigil.md",
            entity_markdown(
                "entity_ember_sigil_001",
                "ember-sigil",
                "artifact",
                "Ember Sigil",
                &[],
            ),
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");

        assert_eq!(report.entity_count, 2);
        assert_eq!(report.entity_alias_count, 6);
        assert_eq!(report.entity_error_count, 0);

        let connection = Connection::open(database.database_path()).expect("open database");
        let entity_type: String = connection
            .query_row(
                "SELECT entity_type FROM story_index_entities WHERE target = 'ember-sigil';",
                [],
                |row| row.get(0),
            )
            .expect("entity type");
        assert_eq!(entity_type, "artifact");

        let eo_alias_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM story_index_entity_aliases
                 WHERE target = 'eoghan' AND normalized_alias IN ('eo', 'eog');",
                [],
                |row| row.get(0),
            )
            .expect("alias count");
        assert_eq!(eo_alias_count, 2);
    }

    #[test]
    fn indexes_entity_front_matter_without_id_aliases_or_matching_filename() {
        let root = TestDir::new();
        root.write(
            "Characters/Eoghan Profile.md",
            "---\ntarget: eoghan\ntype: Character\nname: Eoghan\n---\nNotes.\n",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");
        let characters = database
            .entities_of_type("character")
            .expect("query character entities");

        assert_eq!(report.entity_count, 1);
        assert_eq!(report.entity_error_count, 0);
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].target, "eoghan");
        assert_eq!(characters[0].entity_type, "Character");
    }

    #[test]
    fn reports_invalid_and_duplicate_entities_without_indexing_them() {
        let root = TestDir::new();
        root.write("plain.md", "ordinary Markdown note");
        root.write("broken.md", "---\ntarget: broken\n---\n");
        root.write(
            "left/eoghan.md",
            entity_markdown(
                "entity_eoghan_left_001",
                "eoghan",
                "character",
                "Eoghan",
                &[],
            ),
        );
        root.write(
            "right/eoghan.md",
            entity_markdown(
                "entity_eoghan_right_001",
                "eoghan",
                "character",
                "Eoghan Duplicate",
                &[],
            ),
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");

        assert_eq!(report.file_count, 4);
        assert_eq!(report.entity_count, 0);
        assert_eq!(report.duplicate_entity_target_count, 1);
        assert_eq!(report.entity_error_count, 3);

        let connection = Connection::open(database.database_path()).expect("open database");
        let entity_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM story_index_entities;", [], |row| {
                row.get(0)
            })
            .expect("entity count");
        let error_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM story_index_entity_errors;",
                [],
                |row| row.get(0),
            )
            .expect("error count");
        assert_eq!(entity_count, 0);
        assert_eq!(error_count, 3);
    }

    #[test]
    fn indexes_fountain_scene_records() {
        let root = TestDir::new();
        root.write(
            "pilot.fountain",
            "Title Page\nINT. KITCHEN - NIGHT\nA kettle screams.\nEXT. FOREST ROAD - DAWN\nBirdsong.\n",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");
        let scenes = database
            .scenes_for_file(root.path().join("pilot.fountain"))
            .expect("scenes for file");

        assert_eq!(report.scene_count, 2);
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].scene_ordinal, 1);
        assert_eq!(scenes[0].heading_text, "INT. KITCHEN - NIGHT");
        assert_eq!(scenes[0].normalized_heading, "int kitchen night");
        assert_eq!(scenes[0].start_line, 1);
        assert_eq!(scenes[0].end_line, 2);
        assert_eq!(scenes[0].location_text.as_deref(), Some("KITCHEN"));
        assert_eq!(scenes[0].time_of_day_text.as_deref(), Some("NIGHT"));
        assert_eq!(scenes[1].script_order, 1);
        assert_eq!(scenes[1].location_text.as_deref(), Some("FOREST ROAD"));
        assert_eq!(scenes[1].time_of_day_text.as_deref(), Some("DAWN"));
    }

    #[test]
    fn scene_at_line_maps_to_containing_scene() {
        let root = TestDir::new();
        root.write(
            "episode.fountain",
            "Cold open\nINT. HALLWAY - DAY\nStep.\nINT. HALLWAY - DAY\nAgain.\n",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;
        database.scan_workspace_files().expect("scan workspace");
        let path = root.path().join("episode.fountain");

        assert_eq!(
            database
                .scene_at_line(&path, 0)
                .expect("scene before first heading"),
            None
        );
        let first = database
            .scene_at_line(&path, 2)
            .expect("scene at line")
            .expect("first scene");
        let second = database
            .scene_at_line("episode.fountain", 4)
            .expect("scene at relative line")
            .expect("second scene");

        assert_eq!(first.scene_ordinal, 1);
        assert_eq!(first.start_line, 1);
        assert_eq!(first.end_line, 2);
        assert_eq!(second.scene_ordinal, 2);
        assert_eq!(second.heading_text, "INT. HALLWAY - DAY");
        assert_ne!(first.scene_key, second.scene_key);
    }

    #[test]
    fn indexes_fountain_appearances_and_dialogue_speakers() {
        let root = TestDir::new();
        write_story_entities(&root);
        root.write(
            "episode.fountain",
            "INT. KITCHEN - NIGHT\nThe room is [Kitchen](kitchen).\n[knife](knife) rests on the table.\n[EOGHAN](eoghan)\nHello [knife](knife).\n(holding [knife](knife))\n",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");
        let eoghan = database
            .appearances_of_entity("eoghan")
            .expect("eoghan appearances");
        let knife = database
            .appearances_of_entity("knife")
            .expect("knife appearances");

        assert_eq!(report.appearance_count, 6);
        assert!(eoghan.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::CharacterCue && appearance.line == 3
        }));
        assert!(eoghan.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::DialogueSpeaker && appearance.line == 4
        }));
        assert!(knife.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::ActionMention && appearance.line == 2
        }));
        assert!(knife.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::DialogueMention && appearance.line == 4
        }));
        assert!(knife.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::ParentheticalMention
                && appearance.line == 5
        }));
    }

    #[test]
    fn indexes_canvas_appearances_from_text_file_and_link_nodes() {
        let root = TestDir::new();
        write_story_entities(&root);
        root.write(
            "board.canvas",
            r#"{
                "nodes": [
                    {"id":"text","type":"text","text":"Plan [Eoghan](eoghan) with the [knife](knife).","x":0,"y":0},
                    {"id":"file","type":"file","file":"characters/eoghan.md","x":0,"y":120},
                    {"id":"link","type":"link","url":"props/knife","x":0,"y":240}
                ],
                "edges": []
            }"#,
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;

        let report = database.scan_workspace_files().expect("scan workspace");
        let eoghan = database
            .appearances_of_entity("eoghan")
            .expect("eoghan appearances");
        let knife = database
            .appearances_of_entity("knife")
            .expect("knife appearances");

        assert_eq!(report.appearance_count, 4);
        assert!(eoghan.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::CanvasText
                && appearance.raw_snippet.contains("[Eoghan](eoghan)")
        }));
        assert!(eoghan.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::CanvasFile
                && appearance.raw_snippet == "characters/eoghan.md"
        }));
        assert!(knife.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::CanvasText
                && appearance.raw_snippet.contains("[knife](knife)")
        }));
        assert!(knife.iter().any(|appearance| {
            appearance.role == StoryIndexAppearanceRole::CanvasLink
                && appearance.raw_snippet == "props/knife"
        }));
    }

    #[test]
    fn query_api_returns_entities_scenes_places_and_backlinks() {
        let root = TestDir::new();
        write_story_entities(&root);
        root.write(
            "episode.fountain",
            "INT. KITCHEN - NIGHT\nThe room is [Kitchen](kitchen).\n[knife](knife) rests on the table.\n[EOGHAN](eoghan)\nHello [knife](knife).\nEXT. ROAD - DAY\n[ELIZAH](elizah)\n",
        );
        let database = StoryIndexDatabase::open_workspace(root.path())
            .expect("open story index")
            .database;
        database.scan_workspace_files().expect("scan workspace");
        let kitchen_scene = database
            .scene_at_line("episode.fountain", 2)
            .expect("scene lookup")
            .expect("kitchen scene");

        let suggestions = database
            .entities_matching_text("eo", 10)
            .expect("entity search");
        let props = database
            .props_in_scene(&kitchen_scene.scene_key)
            .expect("props in scene");
        let characters = database
            .characters_in_current_scene("episode.fountain", 4)
            .expect("characters in current scene");
        let shared_scenes = database
            .scenes_containing_all_entities(["eoghan", "knife"])
            .expect("shared scenes");
        let places = database
            .places_for_character("eoghan")
            .expect("places for character");
        let knife_backlinks = database
            .backlinks_to_entity("knife")
            .expect("knife backlinks");

        assert_eq!(suggestions[0].target, "eoghan");
        assert_eq!(
            props
                .iter()
                .map(|entity| entity.target.as_str())
                .collect::<Vec<_>>(),
            vec!["knife"]
        );
        assert_eq!(
            characters
                .iter()
                .map(|entity| entity.target.as_str())
                .collect::<Vec<_>>(),
            vec!["eoghan"]
        );
        assert_eq!(shared_scenes.len(), 1);
        assert_eq!(shared_scenes[0].heading_text, "INT. KITCHEN - NIGHT");
        assert_eq!(places.len(), 1);
        assert_eq!(
            places[0]
                .place
                .as_ref()
                .map(|entity| entity.target.as_str()),
            Some("kitchen")
        );
        assert!(places[0].raw_location.is_none());
        assert_eq!(knife_backlinks.len(), 2);
    }

    fn entity_markdown(
        id: &str,
        target: &str,
        entity_type: &str,
        name: &str,
        aliases: &[&str],
    ) -> String {
        let aliases = if aliases.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "\n{}",
                aliases
                    .iter()
                    .map(|alias| format!("  - {alias}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        format!(
            "---\nid: {id}\ntarget: {target}\ntype: {entity_type}\nname: {name}\naliases: {aliases}\nstatus: draft\n---\n"
        )
    }

    fn write_story_entities(root: &TestDir) {
        root.write(
            "characters/eoghan.md",
            entity_markdown(
                "entity_eoghan_001",
                "eoghan",
                "character",
                "Eoghan",
                &["Eo"],
            ),
        );
        root.write(
            "characters/elizah.md",
            entity_markdown("entity_elizah_001", "elizah", "character", "Elizah", &[]),
        );
        root.write(
            "props/knife.md",
            entity_markdown("entity_knife_001", "knife", "prop", "Knife", &[]),
        );
        root.write(
            "places/kitchen.md",
            entity_markdown("entity_kitchen_001", "kitchen", "place", "Kitchen", &[]),
        );
    }
}
