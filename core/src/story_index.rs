use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, params};

pub const STORY_INDEX_SCHEMA_VERSION: i64 = 1;
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
    IncompatibleSchemaVersion {
        found: i64,
        expected: i64,
    },
    RecoveryFailed {
        original_error: String,
        recovery_error: io::Error,
    },
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
        );",
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
            Self::IncompatibleSchemaVersion { .. } => None,
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
}
