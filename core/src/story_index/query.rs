use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use crate::story_index::{
    StoryIndexAppearanceRecord, StoryIndexDatabase, StoryIndexError, StoryIndexSceneRecord,
    appearance_record_from_row, normalize_lookup_text, normalize_workspace_file_path,
    scene_record_from_row,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexEntityRecord {
    pub target: String,
    pub entity_type: String,
    pub name: String,
    pub status: Option<String>,
    pub path: PathBuf,
    pub relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryIndexPlaceVisit {
    pub scene: StoryIndexSceneRecord,
    pub place: Option<StoryIndexEntityRecord>,
    pub raw_location: Option<String>,
}

impl StoryIndexDatabase {
    pub fn all_entities(&self) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_by_type(None)
    }

    pub fn entities_of_type(
        &self,
        entity_type: &str,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_by_type(Some(entity_type))
    }

    pub fn character_appearance_entities(
        &self,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut statement = connection.prepare(
            "SELECT target,
                    COALESCE(MAX(entity_type), 'character') AS entity_type,
                    COALESCE(MAX(entity_name), target) AS name,
                    NULL AS status,
                    MIN(source_path) AS path,
                    MIN(relative_path) AS relative_path
             FROM story_index_appearances
             WHERE appearance_role IN ('character_cue', 'dialogue_speaker')
             GROUP BY target
             ORDER BY name, target;",
        )?;
        let rows = statement.query_map([], entity_record_from_row)?;
        collect_rows(rows)
    }

    pub fn all_scenes(&self) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut statement = connection.prepare(
            "SELECT scene_key, source_path, relative_path, scene_ordinal, heading_text,
                    normalized_heading, start_line, end_line, script_order, location_text,
                    time_of_day_text
             FROM story_index_scenes
             ORDER BY script_order, source_path, scene_ordinal;",
        )?;
        let rows = statement.query_map([], scene_record_from_row)?;
        collect_rows(rows)
    }

    pub fn entities_matching_text(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let normalized = normalize_lookup_text(query);
        if normalized.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let connection = self.open_connection()?;
        let contains = format!("%{normalized}%");
        let prefix = format!("{normalized}%");
        let mut statement = connection.prepare(
            "SELECT DISTINCT e.target, e.entity_type, e.name, e.status, e.path, e.relative_path,
                    CASE
                        WHEN a.normalized_alias = ?1 THEN 0
                        WHEN a.normalized_alias LIKE ?2 THEN 1
                        WHEN e.target LIKE ?2 THEN 2
                        WHEN a.normalized_alias LIKE ?3 THEN 3
                        ELSE 4
                    END AS rank
             FROM story_index_entities e
             LEFT JOIN story_index_entity_aliases a ON a.target = e.target
             WHERE e.target LIKE ?3 OR e.name LIKE ?3 OR a.normalized_alias LIKE ?3
             ORDER BY rank, e.name, e.target
             LIMIT ?4;",
        )?;
        let rows = statement.query_map(
            params![normalized, prefix, contains, limit as i64],
            entity_record_from_row,
        )?;
        collect_rows(rows)
    }

    pub fn appearances_of_entity(
        &self,
        target: &str,
    ) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
        self.appearances_for_target(target)
    }

    pub fn appearances_in_scene(
        &self,
        scene_key: &str,
    ) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut statement = connection.prepare(
            "SELECT target, entity_type, entity_name, source_path, relative_path, scene_key,
                    line, column, line_kind, raw_snippet, appearance_role
             FROM story_index_appearances
             WHERE scene_key = ?1
             ORDER BY source_path, line, column, appearance_role;",
        )?;
        let rows = statement.query_map(params![scene_key], appearance_record_from_row)?;
        collect_rows(rows)
    }

    pub fn backlinks_to_entity(
        &self,
        target: &str,
    ) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
        self.appearances_for_target(target)
    }

    pub fn entities_in_scene(
        &self,
        scene_key: &str,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_in_scene_by_type(scene_key, None)
    }

    pub fn props_in_scene(
        &self,
        scene_key: &str,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_in_scene_by_type(scene_key, Some("prop"))
    }

    pub fn characters_in_scene(
        &self,
        scene_key: &str,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_in_scene_by_type(scene_key, Some("character"))
    }

    pub fn entities_in_current_scene(
        &self,
        source_path: impl AsRef<Path>,
        line: usize,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        let Some(scene) = self.scene_at_line(source_path, line)? else {
            return Ok(Vec::new());
        };
        self.entities_in_scene(&scene.scene_key)
    }

    pub fn props_in_current_scene(
        &self,
        source_path: impl AsRef<Path>,
        line: usize,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        let Some(scene) = self.scene_at_line(source_path, line)? else {
            return Ok(Vec::new());
        };
        self.props_in_scene(&scene.scene_key)
    }

    pub fn characters_in_current_scene(
        &self,
        source_path: impl AsRef<Path>,
        line: usize,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        let Some(scene) = self.scene_at_line(source_path, line)? else {
            return Ok(Vec::new());
        };
        self.characters_in_scene(&scene.scene_key)
    }

    pub fn scenes_containing_entity(
        &self,
        target: &str,
    ) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut statement = connection.prepare(
            "SELECT DISTINCT s.scene_key, s.source_path, s.relative_path, s.scene_ordinal,
                    s.heading_text, s.normalized_heading, s.start_line, s.end_line,
                    s.script_order, s.location_text, s.time_of_day_text
             FROM story_index_scenes s
             JOIN story_index_appearances a ON a.scene_key = s.scene_key
             WHERE a.target = ?1
             ORDER BY s.script_order, s.source_path, s.scene_ordinal;",
        )?;
        let rows = statement.query_map(params![target], scene_record_from_row)?;
        collect_rows(rows)
    }

    pub fn scenes_containing_all_entities<I, S>(
        &self,
        targets: I,
    ) -> Result<Vec<StoryIndexSceneRecord>, StoryIndexError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let targets = targets
            .into_iter()
            .map(|target| target.as_ref().to_string())
            .filter(|target| !target.is_empty())
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let mut scenes = self.scenes_containing_entity(&targets[0])?;
        scenes.retain(|scene| {
            targets[1..].iter().all(|target| {
                self.scene_contains_target(&scene.scene_key, target)
                    .unwrap_or(false)
            })
        });
        Ok(scenes)
    }

    pub fn places_for_character(
        &self,
        character_target: &str,
    ) -> Result<Vec<StoryIndexPlaceVisit>, StoryIndexError> {
        let character_scenes = self.scenes_containing_entity(character_target)?;
        let mut visits = Vec::<StoryIndexPlaceVisit>::new();

        for scene in character_scenes {
            let explicit_places = self.props_or_places_in_scene(&scene.scene_key, "place")?;
            if explicit_places.is_empty() {
                visits.push(StoryIndexPlaceVisit {
                    raw_location: scene.location_text.clone(),
                    scene,
                    place: None,
                });
                continue;
            }

            for place in explicit_places {
                visits.push(StoryIndexPlaceVisit {
                    scene: scene.clone(),
                    raw_location: None,
                    place: Some(place),
                });
            }
        }

        Ok(visits)
    }

    fn ensure_ready(&self) -> Result<(), StoryIndexError> {
        super::initialize_database(&self.workspace_root, &self.database_path)
    }

    fn open_connection(&self) -> Result<Connection, StoryIndexError> {
        Connection::open(&self.database_path).map_err(StoryIndexError::Sqlite)
    }

    fn entities_by_type(
        &self,
        entity_type: Option<&str>,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        match entity_type {
            Some(entity_type) => {
                let mut statement = connection.prepare(
                    "SELECT target, entity_type, name, status, path, relative_path
                     FROM story_index_entities
                     WHERE LOWER(entity_type) = LOWER(?1)
                     ORDER BY name, target;",
                )?;
                let rows = statement.query_map(params![entity_type], entity_record_from_row)?;
                collect_rows(rows)
            }
            None => {
                let mut statement = connection.prepare(
                    "SELECT target, entity_type, name, status, path, relative_path
                     FROM story_index_entities
                     ORDER BY entity_type, name, target;",
                )?;
                let rows = statement.query_map([], entity_record_from_row)?;
                collect_rows(rows)
            }
        }
    }

    fn appearances_for_target(
        &self,
        target: &str,
    ) -> Result<Vec<StoryIndexAppearanceRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut statement = connection.prepare(
            "SELECT target, entity_type, entity_name, source_path, relative_path, scene_key,
                    line, column, line_kind, raw_snippet, appearance_role
             FROM story_index_appearances
             WHERE target = ?1
             ORDER BY source_path, line, column, appearance_role;",
        )?;
        let rows = statement.query_map(params![target], appearance_record_from_row)?;
        collect_rows(rows)
    }

    fn entities_in_scene_by_type(
        &self,
        scene_key: &str,
        entity_type: Option<&str>,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let mut sql = String::from(
            "SELECT DISTINCT e.target, e.entity_type, e.name, e.status, e.path, e.relative_path
             FROM story_index_entities e
             JOIN story_index_appearances a ON a.target = e.target
             WHERE a.scene_key = ?1",
        );
        if entity_type.is_some() {
            sql.push_str(" AND LOWER(e.entity_type) = LOWER(?2)");
        }
        sql.push_str(" ORDER BY e.name, e.target;");

        let mut statement = connection.prepare(&sql)?;
        let rows = match entity_type {
            Some(entity_type) => {
                statement.query_map(params![scene_key, entity_type], entity_record_from_row)?
            }
            None => statement.query_map(params![scene_key], entity_record_from_row)?,
        };
        collect_rows(rows)
    }

    fn props_or_places_in_scene(
        &self,
        scene_key: &str,
        entity_type: &str,
    ) -> Result<Vec<StoryIndexEntityRecord>, StoryIndexError> {
        self.entities_in_scene_by_type(scene_key, Some(entity_type))
    }

    fn scene_contains_target(
        &self,
        scene_key: &str,
        target: &str,
    ) -> Result<bool, StoryIndexError> {
        self.ensure_ready()?;
        let connection = self.open_connection()?;
        let exists = connection
            .query_row(
                "SELECT 1 FROM story_index_appearances
                 WHERE scene_key = ?1 AND target = ?2
                 LIMIT 1;",
                params![scene_key, target],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        Ok(exists)
    }
}

fn entity_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoryIndexEntityRecord> {
    Ok(StoryIndexEntityRecord {
        target: row.get(0)?,
        entity_type: row.get(1)?,
        name: row.get(2)?,
        status: row.get(3)?,
        path: PathBuf::from(row.get::<_, String>(4)?),
        relative_path: row.get(5)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>, StoryIndexError> {
    let mut out = Vec::<T>::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[allow(dead_code)]
fn normalized_source_path(database: &StoryIndexDatabase, source_path: impl AsRef<Path>) -> PathBuf {
    normalize_workspace_file_path(&database.workspace_root, source_path.as_ref())
}
