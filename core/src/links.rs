use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs, io,
    ops::{Range, RangeInclusive},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptLinkSyntax {
    TargetOnly,
    LabelledTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptLink {
    pub span: Range<usize>,
    pub label: String,
    pub target: String,
    pub syntax: ScriptLinkSyntax,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkDisplayText {
    pub text: String,
    pub display_to_raw: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFrontMatter {
    pub id: String,
    pub target: String,
    pub entity_type: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityDocument {
    pub metadata: EntityFrontMatter,
    pub path: PathBuf,
    pub body: String,
}

#[derive(Clone, Debug, Default)]
pub struct EntityCatalog {
    entities: BTreeMap<String, EntityDocument>,
    alias_index: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionSource {
    ExplicitLink,
    Alias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEntity {
    pub source: ResolutionSource,
    pub mention: String,
    pub target: String,
    pub path: PathBuf,
    pub entity: EntityFrontMatter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionOrigin {
    EntityMatch,
    AmbiguousAlias,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntitySuggestion {
    pub target: String,
    pub name: String,
    pub score: u16,
    pub origin: SuggestionOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestedEntityResolution {
    pub mention: String,
    pub suggestions: Vec<EntitySuggestion>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnresolvedReason {
    MissingTargetFile,
    InvalidTarget,
    UnknownMention,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityScaffold {
    pub target: String,
    pub path: PathBuf,
    pub markdown: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedEntityResolution {
    pub mention: String,
    pub explicit_target: Option<String>,
    pub reason: UnresolvedReason,
    pub scaffold: Option<EntityScaffold>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionResolution {
    Resolved(ResolvedEntity),
    Suggested(SuggestedEntityResolution),
    Unresolved(UnresolvedEntityResolution),
}

#[derive(Debug)]
pub enum LinkError {
    Io(io::Error),
    MissingFrontMatter {
        path: PathBuf,
    },
    UnterminatedFrontMatter {
        path: PathBuf,
    },
    MissingField {
        path: PathBuf,
        field: &'static str,
    },
    MalformedFrontMatterLine {
        path: PathBuf,
        line: usize,
        content: String,
    },
    InvalidTargetKey {
        path: Option<PathBuf>,
        target: String,
    },
    FilenameTargetMismatch {
        path: PathBuf,
        file_stem: String,
        target: String,
    },
    NonUtf8FileStem {
        path: PathBuf,
    },
    DuplicateTarget {
        target: String,
        first: PathBuf,
        second: PathBuf,
    },
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::MissingFrontMatter { path } => {
                write!(f, "{} is missing YAML front matter", path.display())
            }
            Self::UnterminatedFrontMatter { path } => {
                write!(f, "{} has unterminated YAML front matter", path.display())
            }
            Self::MissingField { path, field } => {
                write!(f, "{} is missing required field `{field}`", path.display())
            }
            Self::MalformedFrontMatterLine {
                path,
                line,
                content,
            } => write!(
                f,
                "{} has malformed YAML front matter on line {}: {}",
                path.display(),
                line,
                content
            ),
            Self::InvalidTargetKey { path, target } => match path {
                Some(path) => write!(
                    f,
                    "{} declares invalid target key `{target}`",
                    path.display()
                ),
                None => write!(f, "invalid target key `{target}`"),
            },
            Self::FilenameTargetMismatch {
                path,
                file_stem,
                target,
            } => write!(
                f,
                "{} uses file stem `{file_stem}` but front matter target is `{target}`",
                path.display()
            ),
            Self::NonUtf8FileStem { path } => {
                write!(f, "{} does not have a UTF-8 file stem", path.display())
            }
            Self::DuplicateTarget {
                target,
                first,
                second,
            } => write!(
                f,
                "duplicate target `{target}` found in {} and {}",
                first.display(),
                second.display()
            ),
        }
    }
}

impl Error for LinkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for LinkError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn extract_script_links(input: &str) -> Vec<ScriptLink> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut links = Vec::<ScriptLink>::new();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '[' {
            index += 1;
            continue;
        }

        let Some(label_end) = chars[index + 1..]
            .iter()
            .position(|ch| *ch == ']')
            .map(|offset| index + 1 + offset)
        else {
            break;
        };

        let label = chars[index + 1..label_end].iter().collect::<String>();
        if label.is_empty() {
            index += 1;
            continue;
        }

        if chars.get(label_end + 1) == Some(&'(') {
            let Some(target_end) = chars[label_end + 2..]
                .iter()
                .position(|ch| *ch == ')')
                .map(|offset| label_end + 2 + offset)
            else {
                index += 1;
                continue;
            };
            let target = chars[label_end + 2..target_end].iter().collect::<String>();
            if is_valid_target_key(&target) {
                links.push(ScriptLink {
                    span: index..target_end + 1,
                    label,
                    target,
                    syntax: ScriptLinkSyntax::LabelledTarget,
                });
                index = target_end + 1;
                continue;
            }

            index += 1;
            continue;
        }

        if is_valid_target_key(&label) {
            links.push(ScriptLink {
                span: index..label_end + 1,
                label: label.clone(),
                target: label,
                syntax: ScriptLinkSyntax::TargetOnly,
            });
            index = label_end + 1;
            continue;
        }

        index += 1;
    }

    links
}

pub fn render_script_link_text(input: &str) -> LinkDisplayText {
    let chars = input.chars().collect::<Vec<_>>();
    let links = extract_script_links(input);
    let mut rendered = String::new();
    let mut display_to_raw = vec![0usize];
    let mut cursor = 0usize;

    for link in &links {
        while cursor < link.span.start {
            rendered.push(chars[cursor]);
            display_to_raw.push(cursor + 1);
            cursor += 1;
        }

        let label_raw_start = link.span.start + 1;
        for (offset, ch) in link.label.chars().enumerate() {
            rendered.push(ch);
            display_to_raw.push(label_raw_start + offset + 1);
        }

        if let Some(last) = display_to_raw.last_mut() {
            *last = link.span.end;
        }

        cursor = link.span.end;
    }

    while cursor < chars.len() {
        rendered.push(chars[cursor]);
        display_to_raw.push(cursor + 1);
        cursor += 1;
    }

    LinkDisplayText {
        text: rendered,
        display_to_raw,
    }
}

pub fn is_valid_target_key(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }

    let parts = target.split('-').collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return false;
    }

    parts.iter().all(|part| {
        part.chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    })
}

pub fn script_link_visible_column_range(link: &ScriptLink) -> RangeInclusive<usize> {
    let start = link.span.start.saturating_add(1);
    let end = match link.syntax {
        ScriptLinkSyntax::TargetOnly => link.span.end.saturating_sub(1),
        ScriptLinkSyntax::LabelledTarget => start.saturating_add(link.label.chars().count()),
    };
    start..=end
}

pub fn script_link_contains_visible_column(link: &ScriptLink, column: usize) -> bool {
    script_link_visible_column_range(link).contains(&column)
}

impl EntityDocument {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LinkError> {
        let path = path.as_ref().to_path_buf();
        let markdown = fs::read_to_string(&path)?;
        Self::from_markdown(path, &markdown)
    }

    pub fn from_markdown(path: impl AsRef<Path>, markdown: &str) -> Result<Self, LinkError> {
        let path = path.as_ref().to_path_buf();
        let (front_matter, body) = split_front_matter(markdown, &path)?;
        let metadata = parse_front_matter(&path, &front_matter)?;
        validate_target_against_path(&path, &metadata.target)?;

        Ok(Self {
            metadata,
            path,
            body,
        })
    }
}

impl EntityCatalog {
    pub fn load_from_dir(root: impl AsRef<Path>) -> Result<Self, LinkError> {
        let mut files = Vec::<PathBuf>::new();
        collect_markdown_files(root.as_ref(), &mut files)?;
        let mut documents = Vec::<EntityDocument>::with_capacity(files.len());
        for path in files {
            documents.push(EntityDocument::load(path)?);
        }
        Self::from_documents(documents)
    }

    pub fn from_documents<I>(documents: I) -> Result<Self, LinkError>
    where
        I: IntoIterator<Item = EntityDocument>,
    {
        let mut entities = BTreeMap::<String, EntityDocument>::new();
        let mut alias_index = BTreeMap::<String, BTreeSet<String>>::new();

        for document in documents {
            let target = document.metadata.target.clone();
            if let Some(existing) = entities.get(&target) {
                return Err(LinkError::DuplicateTarget {
                    target,
                    first: existing.path.clone(),
                    second: document.path.clone(),
                });
            }

            let lookup_terms = document
                .metadata
                .aliases
                .iter()
                .cloned()
                .chain(std::iter::once(document.metadata.name.clone()))
                .collect::<Vec<_>>();

            for term in lookup_terms {
                let normalized = normalize_lookup(&term);
                if normalized.is_empty() {
                    continue;
                }
                alias_index
                    .entry(normalized)
                    .or_default()
                    .insert(document.metadata.target.clone());
            }

            entities.insert(document.metadata.target.clone(), document);
        }

        Ok(Self {
            entities,
            alias_index,
        })
    }

    pub fn entity(&self, target: &str) -> Option<&EntityDocument> {
        self.entities.get(target)
    }

    pub fn resolve_script_link(
        &self,
        link: &ScriptLink,
        entity_root: impl AsRef<Path>,
    ) -> MentionResolution {
        self.resolve_mention(&link.label, Some(&link.target), entity_root)
    }

    pub fn resolve_mention(
        &self,
        mention: &str,
        explicit_target: Option<&str>,
        entity_root: impl AsRef<Path>,
    ) -> MentionResolution {
        let mention = mention.trim().to_owned();

        if let Some(target) = explicit_target {
            if !is_valid_target_key(target) {
                return MentionResolution::Unresolved(UnresolvedEntityResolution {
                    mention,
                    explicit_target: Some(target.to_owned()),
                    reason: UnresolvedReason::InvalidTarget,
                    scaffold: None,
                });
            }

            if let Some(entity) = self.entity(target) {
                return MentionResolution::Resolved(ResolvedEntity {
                    source: ResolutionSource::ExplicitLink,
                    mention,
                    target: target.to_owned(),
                    path: entity.path.clone(),
                    entity: entity.metadata.clone(),
                });
            }

            return MentionResolution::Unresolved(UnresolvedEntityResolution {
                mention,
                explicit_target: Some(target.to_owned()),
                reason: UnresolvedReason::MissingTargetFile,
                scaffold: self.scaffold_for_target(entity_root, target).ok(),
            });
        }

        let lookup = normalize_lookup(&mention);
        if let Some(targets) = self.alias_index.get(&lookup) {
            if targets.len() == 1 {
                if let Some(target) = targets.iter().next() {
                    if let Some(entity) = self.entity(target) {
                        return MentionResolution::Resolved(ResolvedEntity {
                            source: ResolutionSource::Alias,
                            mention,
                            target: target.clone(),
                            path: entity.path.clone(),
                            entity: entity.metadata.clone(),
                        });
                    }
                }
            }
        }

        let mut suggestions = Vec::<EntitySuggestion>::new();
        if let Some(targets) = self.alias_index.get(&lookup) {
            for target in targets {
                if let Some(entity) = self.entity(target) {
                    suggestions.push(EntitySuggestion {
                        target: target.clone(),
                        name: entity.metadata.name.clone(),
                        score: 100,
                        origin: SuggestionOrigin::AmbiguousAlias,
                    });
                }
            }
        }

        for suggestion in self.suggest_entities(&mention) {
            if suggestions
                .iter()
                .any(|existing| existing.target == suggestion.target)
            {
                continue;
            }
            suggestions.push(suggestion);
        }

        suggestions.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.target.cmp(&right.target))
        });

        if !suggestions.is_empty() {
            return MentionResolution::Suggested(SuggestedEntityResolution {
                mention,
                suggestions,
            });
        }

        MentionResolution::Unresolved(UnresolvedEntityResolution {
            mention,
            explicit_target: None,
            reason: UnresolvedReason::UnknownMention,
            scaffold: None,
        })
    }

    pub fn suggest_entities(&self, mention: &str) -> Vec<EntitySuggestion> {
        let normalized_mention = normalize_lookup(mention);
        if normalized_mention.is_empty() {
            return Vec::new();
        }

        let mut suggestions = Vec::<EntitySuggestion>::new();
        for entity in self.entities.values() {
            let mut best_score = score_candidate(
                &normalized_mention,
                &normalize_lookup(&entity.metadata.target.replace('-', " ")),
            );

            best_score = best_score.max(score_candidate(
                &normalized_mention,
                &normalize_lookup(&entity.metadata.name),
            ));
            for alias in &entity.metadata.aliases {
                best_score = best_score.max(score_candidate(
                    &normalized_mention,
                    &normalize_lookup(alias),
                ));
            }

            if best_score >= 50 {
                suggestions.push(EntitySuggestion {
                    target: entity.metadata.target.clone(),
                    name: entity.metadata.name.clone(),
                    score: best_score,
                    origin: SuggestionOrigin::EntityMatch,
                });
            }
        }

        suggestions.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.target.cmp(&right.target))
        });
        suggestions
    }

    pub fn scaffold_for_target(
        &self,
        entity_root: impl AsRef<Path>,
        target: &str,
    ) -> Result<EntityScaffold, LinkError> {
        scaffold_entity(entity_root, target)
    }
}

pub fn scaffold_entity(
    entity_root: impl AsRef<Path>,
    target: &str,
) -> Result<EntityScaffold, LinkError> {
    if !is_valid_target_key(target) {
        return Err(LinkError::InvalidTargetKey {
            path: None,
            target: target.to_owned(),
        });
    }

    let path = entity_root.as_ref().join(format!("{target}.md"));
    let id = format!("entity_{}_001", target.replace('-', "_"));
    let markdown = format!(
        "---\nid: {id}\ntarget: {target}\ntype: unknown\nname: {}\naliases: []\nstatus: draft\n---\n",
        humanize_target(target)
    );

    Ok(EntityScaffold {
        target: target.to_owned(),
        path,
        markdown,
    })
}

fn collect_markdown_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), LinkError> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
            continue;
        }

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase());
        if matches!(extension.as_deref(), Some("md") | Some("markdown")) {
            out.push(path);
        }
    }

    Ok(())
}

fn split_front_matter(markdown: &str, path: &Path) -> Result<(Vec<String>, String), LinkError> {
    let lines = markdown
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_owned())
        .collect::<Vec<_>>();

    if lines.first().map(String::as_str) != Some("---") {
        return Err(LinkError::MissingFrontMatter {
            path: path.to_path_buf(),
        });
    }

    let Some(end_index) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line == "---").then_some(index))
    else {
        return Err(LinkError::UnterminatedFrontMatter {
            path: path.to_path_buf(),
        });
    };

    let front_matter = lines[1..end_index].to_vec();
    let body = if end_index + 1 < lines.len() {
        lines[end_index + 1..].join("\n")
    } else {
        String::new()
    };

    Ok((front_matter, body))
}

fn parse_front_matter(path: &Path, lines: &[String]) -> Result<EntityFrontMatter, LinkError> {
    let mut id = None::<String>;
    let mut target = None::<String>;
    let mut entity_type = None::<String>;
    let mut name = None::<String>;
    let mut aliases = Vec::<String>::new();
    let mut aliases_seen = false;
    let mut status = None::<String>;
    let mut reading_alias_block = false;

    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 2;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if reading_alias_block && trimmed.starts_with("- ") {
            let alias = parse_yaml_scalar(&trimmed[2..]);
            if !alias.is_empty() {
                aliases.push(alias);
            }
            continue;
        }
        reading_alias_block = false;

        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(LinkError::MalformedFrontMatterLine {
                path: path.to_path_buf(),
                line: line_number,
                content: trimmed.to_owned(),
            });
        };
        let key = key.trim();
        let value = value.trim();

        match key {
            "id" => id = Some(parse_yaml_scalar(value)),
            "target" => target = Some(parse_yaml_scalar(value)),
            "type" => entity_type = Some(parse_yaml_scalar(value)),
            "name" => name = Some(parse_yaml_scalar(value)),
            "aliases" => {
                aliases_seen = true;
                if value.is_empty() {
                    reading_alias_block = true;
                } else {
                    aliases.extend(parse_alias_value(path, line_number, value)?);
                }
            }
            "status" => status = Some(parse_yaml_scalar(value)),
            _ => {}
        }
    }

    let id = required_field(path, "id", id)?;
    let target = required_field(path, "target", target)?;
    let entity_type = required_field(path, "type", entity_type)?;
    let name = required_field(path, "name", name)?;
    if !aliases_seen {
        return Err(LinkError::MissingField {
            path: path.to_path_buf(),
            field: "aliases",
        });
    }

    if !is_valid_target_key(&target) {
        return Err(LinkError::InvalidTargetKey {
            path: Some(path.to_path_buf()),
            target,
        });
    }

    let mut seen_aliases = BTreeSet::<String>::new();
    aliases.retain(|alias| {
        let normalized = normalize_lookup(alias);
        !normalized.is_empty() && seen_aliases.insert(normalized)
    });

    Ok(EntityFrontMatter {
        id,
        target,
        entity_type,
        name,
        aliases,
        status,
    })
}

fn required_field(
    path: &Path,
    field: &'static str,
    value: Option<String>,
) -> Result<String, LinkError> {
    let value = value.ok_or_else(|| LinkError::MissingField {
        path: path.to_path_buf(),
        field,
    })?;
    if value.trim().is_empty() {
        return Err(LinkError::MissingField {
            path: path.to_path_buf(),
            field,
        });
    }
    Ok(value)
}

fn parse_alias_value(path: &Path, line: usize, value: &str) -> Result<Vec<String>, LinkError> {
    if value == "[]" {
        return Ok(Vec::new());
    }

    if !(value.starts_with('[') && value.ends_with(']')) {
        return Err(LinkError::MalformedFrontMatterLine {
            path: path.to_path_buf(),
            line,
            content: format!("aliases: {value}"),
        });
    }

    let inner = &value[1..value.len().saturating_sub(1)];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    Ok(inner
        .split(',')
        .map(parse_yaml_scalar)
        .filter(|alias| !alias.is_empty())
        .collect())
}

fn parse_yaml_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let quoted_with_double = trimmed.starts_with('"') && trimmed.ends_with('"');
        let quoted_with_single = trimmed.starts_with('\'') && trimmed.ends_with('\'');
        if quoted_with_double || quoted_with_single {
            return trimmed[1..trimmed.len() - 1].to_owned();
        }
    }
    trimmed.to_owned()
}

fn validate_target_against_path(path: &Path, target: &str) -> Result<(), LinkError> {
    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| LinkError::NonUtf8FileStem {
            path: path.to_path_buf(),
        })?;

    if file_stem != target {
        return Err(LinkError::FilenameTargetMismatch {
            path: path.to_path_buf(),
            file_stem: file_stem.to_owned(),
            target: target.to_owned(),
        });
    }

    Ok(())
}

fn normalize_lookup(input: &str) -> String {
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

fn score_candidate(mention: &str, candidate: &str) -> u16 {
    if mention.is_empty() || candidate.is_empty() {
        return 0;
    }
    if mention == candidate {
        return 100;
    }

    let mention_tokens = mention.split_whitespace().collect::<Vec<_>>();
    let candidate_tokens = candidate.split_whitespace().collect::<Vec<_>>();
    if mention_tokens.is_empty() || candidate_tokens.is_empty() {
        return 0;
    }

    let mention_set = mention_tokens.iter().copied().collect::<BTreeSet<_>>();
    let candidate_set = candidate_tokens.iter().copied().collect::<BTreeSet<_>>();
    let overlap = mention_set.intersection(&candidate_set).count();

    let mut score = if overlap == 0 {
        0
    } else {
        ((overlap * 100) / mention_set.len().max(candidate_set.len())) as u16
    };

    if candidate.contains(mention) || mention.contains(candidate) {
        score = score.max(70);
    }
    if mention_tokens.first() == candidate_tokens.first() {
        score = score.saturating_add(10);
    }

    score.min(99)
}

fn humanize_target(target: &str) -> String {
    target
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path =
                env::temp_dir().join(format!("basscript-links-{}-{}", std::process::id(), unique));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::write(path, contents).expect("write file");
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn kitchen_door_markdown() -> &'static str {
        "---\nid: obj_door_kitchen_main_001\ntarget: door-kitchen-main\ntype: prop\nname: Kitchen main door\naliases:\n  - kitchen door\n  - main kitchen door\nstatus: draft\n---\nCanonical notes.\n"
    }

    #[test]
    fn extracts_direct_and_labelled_links() {
        let links = extract_script_links(
            "He opens [door-kitchen-main] and points at [that door](door-kitchen-main).",
        );

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].label, "door-kitchen-main");
        assert_eq!(links[0].target, "door-kitchen-main");
        assert_eq!(links[0].syntax, ScriptLinkSyntax::TargetOnly);
        assert_eq!(links[1].label, "that door");
        assert_eq!(links[1].target, "door-kitchen-main");
        assert_eq!(links[1].syntax, ScriptLinkSyntax::LabelledTarget);
    }

    #[test]
    fn ignores_non_slug_targets() {
        let links = extract_script_links("Ignore [site](https://example.com) and [not a target].");
        assert!(links.is_empty());
    }

    #[test]
    fn renders_labelled_links_as_visible_text() {
        let rendered = render_script_link_text("Open [that door](door-kitchen-main) now.");

        assert_eq!(rendered.text, "Open that door now.");
        assert_eq!(rendered.display_to_raw.last().copied(), Some(40));
    }

    #[test]
    fn computes_visible_click_range_for_labelled_links() {
        let link = extract_script_links("[that door](door-kitchen-main)")
            .into_iter()
            .next()
            .unwrap();

        assert!(script_link_contains_visible_column(&link, 1));
        assert!(script_link_contains_visible_column(&link, 9));
        assert!(script_link_contains_visible_column(&link, 10));
        assert!(!script_link_contains_visible_column(&link, 11));
    }

    #[test]
    fn loads_entity_document_from_yaml_front_matter() {
        let document =
            EntityDocument::from_markdown("door-kitchen-main.md", kitchen_door_markdown()).unwrap();

        assert_eq!(document.metadata.id, "obj_door_kitchen_main_001");
        assert_eq!(document.metadata.target, "door-kitchen-main");
        assert_eq!(document.metadata.entity_type, "prop");
        assert_eq!(document.metadata.name, "Kitchen main door");
        assert_eq!(
            document.metadata.aliases,
            vec!["kitchen door".to_owned(), "main kitchen door".to_owned()]
        );
        assert_eq!(document.metadata.status.as_deref(), Some("draft"));
    }

    #[test]
    fn rejects_target_filename_mismatch() {
        let error = EntityDocument::from_markdown("other.md", kitchen_door_markdown()).unwrap_err();
        assert!(matches!(error, LinkError::FilenameTargetMismatch { .. }));
    }

    #[test]
    fn resolves_explicit_links_by_target_and_does_not_promote_local_labels() {
        let root = TestDir::new();
        root.write("door-kitchen-main.md", kitchen_door_markdown());
        let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();
        let link = extract_script_links("[that door](door-kitchen-main)")
            .into_iter()
            .next()
            .unwrap();

        let resolved = catalog.resolve_script_link(&link, root.path());
        match resolved {
            MentionResolution::Resolved(entity) => {
                assert_eq!(entity.source, ResolutionSource::ExplicitLink);
                assert_eq!(entity.target, "door-kitchen-main");
            }
            other => panic!("expected explicit resolution, got {other:?}"),
        }

        let alias_lookup = catalog.resolve_mention("that door", None, root.path());
        assert!(!matches!(alias_lookup, MentionResolution::Resolved(_)));
    }

    #[test]
    fn resolves_confirmed_alias_before_suggestions() {
        let root = TestDir::new();
        root.write("door-kitchen-main.md", kitchen_door_markdown());
        let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();

        let resolved = catalog.resolve_mention("kitchen door", None, root.path());
        match resolved {
            MentionResolution::Resolved(entity) => {
                assert_eq!(entity.source, ResolutionSource::Alias);
                assert_eq!(entity.target, "door-kitchen-main");
            }
            other => panic!("expected alias resolution, got {other:?}"),
        }
    }

    #[test]
    fn returns_entity_matching_suggestions_after_alias_lookup() {
        let root = TestDir::new();
        root.write("door-kitchen-main.md", kitchen_door_markdown());
        let catalog = EntityCatalog::load_from_dir(root.path()).unwrap();

        let resolved = catalog.resolve_mention("main door", None, root.path());
        match resolved {
            MentionResolution::Suggested(suggested) => {
                assert_eq!(suggested.suggestions[0].target, "door-kitchen-main");
                assert_eq!(
                    suggested.suggestions[0].origin,
                    SuggestionOrigin::EntityMatch
                );
            }
            other => panic!("expected suggestion resolution, got {other:?}"),
        }
    }

    #[test]
    fn returns_unresolved_with_scaffold_for_missing_explicit_target() {
        let root = TestDir::new();
        let catalog = EntityCatalog::default();

        let resolved = catalog.resolve_mention("that door", Some("door-kitchen-main"), root.path());
        match resolved {
            MentionResolution::Unresolved(unresolved) => {
                assert_eq!(unresolved.reason, UnresolvedReason::MissingTargetFile);
                let scaffold = unresolved.scaffold.expect("expected scaffold");
                assert!(scaffold.path.ends_with("door-kitchen-main.md"));
                assert!(scaffold.markdown.contains("target: door-kitchen-main"));
            }
            other => panic!("expected unresolved resolution, got {other:?}"),
        }
    }
}
