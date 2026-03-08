use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    EntityCatalog, EntityDocument, EntityFrontMatter, EntityScaffold, EntitySuggestion, LinkError,
    MentionResolution, ResolutionSource, ResolvedEntity, ScriptLink, SuggestedEntityResolution,
    SuggestionOrigin, UnresolvedEntityResolution, UnresolvedReason, is_valid_target_key,
};

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
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FrontMatterBlock {
        Aliases,
        Ignore,
    }

    let mut id = None::<String>;
    let mut target = None::<String>;
    let mut entity_type = None::<String>;
    let mut name = None::<String>;
    let mut aliases = Vec::<String>::new();
    let mut aliases_seen = false;
    let mut status = None::<String>;
    let mut active_block = None::<(usize, FrontMatterBlock)>;

    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 2;
        let indent = line.chars().take_while(|ch| ch.is_ascii_whitespace()).count();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((block_indent, block_kind)) = active_block {
            if indent > block_indent {
                match block_kind {
                    FrontMatterBlock::Aliases => {
                        if let Some(alias_value) = trimmed.strip_prefix("- ") {
                            let alias = parse_yaml_scalar(alias_value);
                            if !alias.is_empty() {
                                aliases.push(alias);
                            }
                        }
                        continue;
                    }
                    FrontMatterBlock::Ignore => continue,
                }
            }
            active_block = None;
        }

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
                    active_block = Some((indent, FrontMatterBlock::Aliases));
                } else {
                    aliases.extend(parse_alias_value(path, line_number, value)?);
                }
            }
            "status" => status = Some(parse_yaml_scalar(value)),
            _ => {
                if value.is_empty()
                    || matches!(value.chars().next(), Some('|') | Some('>'))
                {
                    active_block = Some((indent, FrontMatterBlock::Ignore));
                }
            }
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
