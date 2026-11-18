//! Bounded account-catalog index prototype. This module is wired to the
//! terminal registry only after account-level guardian election exists.
#![allow(dead_code)]

use std::{
    collections::{BTreeMap, VecDeque},
    time::{Duration, Instant},
};
use uuid::Uuid;

pub(crate) const DEFAULT_PAGE: u16 = 16;
pub(crate) const MAX_PAGE: u16 = 32;
pub(crate) const MAX_SESSIONS: usize = 32;
pub(crate) const MAX_WATCH_EVENTS: usize = 64;
pub(crate) const MAX_TEXT: usize = 256;
pub(crate) const MAX_LABEL: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionKind {
    Agent,
    Shell,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionState {
    Live,
    Exited,
    Interrupted,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SessionMetadata {
    pub session_id: String,
    pub run_id: String,
    pub scope_id: String,
    pub project_id: String,
    pub kind: SessionKind,
    pub label: Option<String>,
    pub state: SessionState,
    pub processes_active: bool,
    pub exit_code: Option<i32>,
    pub output_start: u64,
    pub output_end: u64,
    pub history_truncated: bool,
    pub attachment_generation: u64,
    pub attached: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogEntry {
    metadata: SessionMetadata,
    created_sequence: u64,
    active_sequence: u64,
    state_since: Instant,
}

impl CatalogEntry {
    pub(crate) fn metadata(&self) -> &SessionMetadata {
        &self.metadata
    }
    pub(crate) fn created_sequence(&self) -> u64 {
        self.created_sequence
    }
    pub(crate) fn active_sequence(&self) -> u64 {
        self.active_sequence
    }
    pub(crate) fn state_age(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.state_since)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatalogChange {
    Upsert,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogEvent {
    pub revision: u64,
    pub change: CatalogChange,
    pub session_id: String,
    pub run_id: String,
    pub state: SessionState,
    pub active_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogCursor {
    pub boot_id: String,
    pub revision: u64,
    pub scope_id: String,
    pub project_id: Option<String>,
    pub after_created: u64,
    pub after_session: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WatchCursor {
    pub boot_id: String,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogPage {
    pub entries: Vec<CatalogEntry>,
    pub next: Option<CatalogCursor>,
    pub total: usize,
    pub revision: u64,
    pub boot_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WatchPage {
    pub events: Vec<CatalogEvent>,
    pub next: WatchCursor,
    pub gap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Inspection {
    pub entry: CatalogEntry,
    pub revision: u64,
    pub boot_id: String,
    pub state_age: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatalogError {
    Invalid,
    Capacity,
    NotFound,
    NotLive,
    Gap,
    Exhausted,
}

#[derive(Clone, Debug)]
pub(crate) struct PageQuery {
    pub scope_id: String,
    pub project_id: Option<String>,
    pub cursor: Option<CatalogCursor>,
    pub limit: Option<u16>,
}

#[derive(Clone, Debug)]
pub(crate) struct InspectQuery {
    pub scope_id: String,
    pub project_id: String,
    pub run_id: Option<String>,
    pub session_id: String,
}

pub(crate) struct Catalog {
    boot_id: String,
    entries: BTreeMap<String, CatalogEntry>,
    events: VecDeque<CatalogEvent>,
    created_sequence: u64,
    active_sequence: u64,
    revision: u64,
}

impl Catalog {
    pub(crate) fn new(boot_id: String) -> Result<Self, CatalogError> {
        if !valid_id(&boot_id) {
            return Err(CatalogError::Invalid);
        }
        Ok(Self {
            boot_id,
            entries: BTreeMap::new(),
            events: VecDeque::new(),
            created_sequence: 0,
            active_sequence: 0,
            revision: 0,
        })
    }

    pub(crate) fn boot_id(&self) -> &str {
        &self.boot_id
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn upsert(
        &mut self,
        metadata: SessionMetadata,
        now: Instant,
    ) -> Result<u64, CatalogError> {
        validate(&metadata)?;
        if let Some(entry) = self.entries.get(&metadata.session_id) {
            if entry.metadata == metadata {
                return Ok(self.revision);
            }
        }
        if self.entries.contains_key(&metadata.session_id) {
            let revision = self.next_revision()?;
            let entry = self
                .entries
                .get_mut(&metadata.session_id)
                .expect("existing catalog entry");
            if entry.metadata.state != metadata.state {
                entry.state_since = now;
            }
            entry.metadata = metadata;
            let event = entry.metadata.clone();
            let active_sequence = entry.active_sequence;
            self.push_event(revision, CatalogChange::Upsert, &event, active_sequence);
            return Ok(revision);
        }
        if self.entries.len() >= MAX_SESSIONS {
            return Err(CatalogError::Capacity);
        }
        let created_sequence = self
            .created_sequence
            .checked_add(1)
            .ok_or(CatalogError::Exhausted)?;
        self.created_sequence = created_sequence;
        let revision = self.next_revision()?;
        let entry = CatalogEntry {
            metadata,
            created_sequence,
            active_sequence: 0,
            state_since: now,
        };
        self.push_event(
            revision,
            CatalogChange::Upsert,
            &entry.metadata,
            entry.active_sequence,
        );
        self.entries
            .insert(entry.metadata.session_id.clone(), entry);
        Ok(revision)
    }

    /// Foreground attach or explicit selection only; read/list never call this.
    pub(crate) fn touch(&mut self, session_id: &str) -> Result<u64, CatalogError> {
        if self
            .entries
            .get(session_id)
            .ok_or(CatalogError::NotFound)?
            .metadata
            .state
            != SessionState::Live
        {
            return Err(CatalogError::NotLive);
        }
        let active_sequence = self
            .active_sequence
            .checked_add(1)
            .ok_or(CatalogError::Exhausted)?;
        self.active_sequence = active_sequence;
        let entry = self
            .entries
            .get_mut(session_id)
            .expect("checked catalog entry");
        entry.active_sequence = active_sequence;
        let event = entry.metadata.clone();
        let revision = self.next_revision()?;
        self.push_event(revision, CatalogChange::Upsert, &event, active_sequence);
        Ok(revision)
    }

    pub(crate) fn remove(&mut self, session_id: &str) -> Result<u64, CatalogError> {
        let entry = self
            .entries
            .remove(session_id)
            .ok_or(CatalogError::NotFound)?;
        let revision = self.next_revision()?;
        self.push_event(
            revision,
            CatalogChange::Remove,
            &entry.metadata,
            entry.active_sequence,
        );
        Ok(revision)
    }

    pub(crate) fn page(&self, query: &PageQuery) -> Result<CatalogPage, CatalogError> {
        if !valid_text(&query.scope_id, MAX_TEXT)
            || query
                .project_id
                .as_deref()
                .is_some_and(|project| !valid_id(project))
        {
            return Err(CatalogError::Invalid);
        }
        if let Some(cursor) = &query.cursor {
            if cursor.boot_id != self.boot_id
                || cursor.scope_id != query.scope_id
                || cursor.project_id != query.project_id
                || !valid_id(&cursor.after_session)
            {
                return Err(CatalogError::Gap);
            }
        }
        let limit = query.limit.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE) as usize;
        let mut entries: Vec<CatalogEntry> = self
            .entries
            .values()
            .filter(|entry| {
                entry.metadata.scope_id == query.scope_id
                    && query
                        .project_id
                        .as_ref()
                        .is_none_or(|project| &entry.metadata.project_id == project)
            })
            .cloned()
            .collect();
        entries.sort_by(|left, right| {
            right
                .created_sequence
                .cmp(&left.created_sequence)
                .then_with(|| right.metadata.session_id.cmp(&left.metadata.session_id))
        });
        let total = entries.len();
        if let Some(cursor) = &query.cursor {
            entries.retain(|entry| {
                entry.created_sequence < cursor.after_created
                    || (entry.created_sequence == cursor.after_created
                        && entry.metadata.session_id < cursor.after_session)
            });
        }
        let more = entries.len() > limit;
        entries.truncate(limit);
        let next = more.then(|| {
            let last = entries.last().expect("non-empty page");
            CatalogCursor {
                boot_id: self.boot_id.clone(),
                revision: self.revision,
                scope_id: query.scope_id.clone(),
                project_id: query.project_id.clone(),
                after_created: last.created_sequence,
                after_session: last.metadata.session_id.clone(),
            }
        });
        Ok(CatalogPage {
            entries,
            next,
            total,
            revision: self.revision,
            boot_id: self.boot_id.clone(),
        })
    }

    pub(crate) fn inspect(
        &self,
        query: &InspectQuery,
        now: Instant,
    ) -> Result<Inspection, CatalogError> {
        if !valid_text(&query.scope_id, MAX_TEXT)
            || !valid_id(&query.project_id)
            || !valid_id(&query.session_id)
            || query.run_id.as_deref().is_some_and(|run| !valid_id(run))
        {
            return Err(CatalogError::Invalid);
        }
        let entry = self
            .entries
            .get(&query.session_id)
            .ok_or(CatalogError::NotFound)?;
        if entry.metadata.scope_id != query.scope_id
            || entry.metadata.project_id != query.project_id
            || query
                .run_id
                .as_ref()
                .is_some_and(|run| &entry.metadata.run_id != run)
        {
            return Err(CatalogError::NotFound);
        }
        Ok(Inspection {
            state_age: entry.state_age(now),
            entry: entry.clone(),
            revision: self.revision,
            boot_id: self.boot_id.clone(),
        })
    }

    pub(crate) fn watch(&self, cursor: Option<&WatchCursor>, limit: Option<u16>) -> WatchPage {
        let Some(cursor) = cursor else {
            return WatchPage {
                events: Vec::new(),
                next: WatchCursor {
                    boot_id: self.boot_id.clone(),
                    revision: self.revision,
                },
                gap: false,
            };
        };
        if cursor.boot_id != self.boot_id || cursor.revision > self.revision {
            return WatchPage {
                events: Vec::new(),
                next: WatchCursor {
                    boot_id: self.boot_id.clone(),
                    revision: self.revision,
                },
                gap: true,
            };
        }
        let Some(first) = self.events.front() else {
            return WatchPage {
                events: Vec::new(),
                next: WatchCursor {
                    boot_id: self.boot_id.clone(),
                    revision: self.revision,
                },
                gap: false,
            };
        };
        if cursor
            .revision
            .checked_add(1)
            .is_none_or(|next| next < first.revision)
        {
            return WatchPage {
                events: Vec::new(),
                next: WatchCursor {
                    boot_id: self.boot_id.clone(),
                    revision: self.revision,
                },
                gap: true,
            };
        }
        let limit = limit.unwrap_or(DEFAULT_PAGE).clamp(1, MAX_PAGE) as usize;
        let events: Vec<CatalogEvent> = self
            .events
            .iter()
            .filter(|event| event.revision > cursor.revision)
            .take(limit)
            .cloned()
            .collect();
        let revision = events
            .last()
            .map_or(cursor.revision, |event| event.revision);
        WatchPage {
            events,
            next: WatchCursor {
                boot_id: self.boot_id.clone(),
                revision,
            },
            gap: false,
        }
    }

    /// Resolves the bare-resume candidate without attaching or changing MRU.
    pub(crate) fn resume(
        &self,
        scope_id: &str,
        project_id: Option<&str>,
    ) -> Result<SessionMetadata, CatalogError> {
        if !valid_text(scope_id, MAX_TEXT) || project_id.is_some_and(|project| !valid_id(project)) {
            return Err(CatalogError::Invalid);
        }
        self.entries
            .values()
            .filter(|entry| {
                entry.metadata.scope_id == scope_id
                    && entry.metadata.state == SessionState::Live
                    && entry.metadata.processes_active
                    && project_id.is_none_or(|project| entry.metadata.project_id == project)
            })
            .max_by(|left, right| {
                left.active_sequence
                    .cmp(&right.active_sequence)
                    .then_with(|| left.created_sequence.cmp(&right.created_sequence))
                    .then_with(|| left.metadata.session_id.cmp(&right.metadata.session_id))
            })
            .map(|entry| entry.metadata.clone())
            .ok_or(CatalogError::NotFound)
    }

    fn next_revision(&mut self) -> Result<u64, CatalogError> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(CatalogError::Exhausted)?;
        self.revision = revision;
        Ok(revision)
    }

    fn push_event(
        &mut self,
        revision: u64,
        change: CatalogChange,
        metadata: &SessionMetadata,
        active_sequence: u64,
    ) {
        if self.events.len() >= MAX_WATCH_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(CatalogEvent {
            revision,
            change,
            session_id: metadata.session_id.clone(),
            run_id: metadata.run_id.clone(),
            state: metadata.state,
            active_sequence,
        });
    }
}

fn valid_id(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| !id.is_nil())
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn validate(metadata: &SessionMetadata) -> Result<(), CatalogError> {
    if !valid_id(&metadata.session_id)
        || !valid_id(&metadata.run_id)
        || !valid_id(&metadata.project_id)
        || !valid_text(&metadata.scope_id, MAX_TEXT)
        || metadata
            .label
            .as_deref()
            .is_some_and(|label| !valid_text(label, MAX_LABEL))
        || metadata.output_start > metadata.output_end
    {
        return Err(CatalogError::Invalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(session: &str, project: &str, state: SessionState) -> SessionMetadata {
        SessionMetadata {
            session_id: session.into(),
            run_id: Uuid::new_v4().to_string(),
            scope_id: "scope:one".into(),
            project_id: project.into(),
            kind: SessionKind::Agent,
            label: Some("build".into()),
            state,
            processes_active: state == SessionState::Live,
            exit_code: (state == SessionState::Exited).then_some(0),
            output_start: 0,
            output_end: 4,
            history_truncated: false,
            attachment_generation: 1,
            attached: true,
        }
    }

    fn catalog() -> Catalog {
        Catalog::new(Uuid::new_v4().to_string()).unwrap()
    }

    #[test]
    fn pages_are_stable_across_insertions_and_bound_the_page() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        let now = Instant::now();
        let ids: Vec<_> = (0..3).map(|_| Uuid::new_v4().to_string()).collect();
        for id in &ids {
            catalog
                .upsert(metadata(id, &project, SessionState::Live), now)
                .unwrap();
        }
        let query = PageQuery {
            scope_id: "scope:one".into(),
            project_id: Some(project.clone()),
            cursor: None,
            limit: Some(2),
        };
        let first = catalog.page(&query).unwrap();
        assert_eq!(first.total, 3);
        assert_eq!(first.entries.len(), 2);
        assert_eq!(first.entries[0].metadata().session_id, ids[2]);
        let cursor = first.next.clone().unwrap();
        let second = catalog
            .page(&PageQuery {
                cursor: Some(cursor),
                ..query.clone()
            })
            .unwrap();
        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].metadata().session_id, ids[0]);
        assert!(second.next.is_none());

        let new = Uuid::new_v4().to_string();
        catalog
            .upsert(metadata(&new, &project, SessionState::Live), now)
            .unwrap();
        let after = catalog
            .page(&PageQuery {
                cursor: first.next,
                ..query
            })
            .unwrap();
        assert_eq!(after.entries.len(), 1);
        assert_eq!(after.entries[0].metadata().session_id, ids[0]);
    }

    #[test]
    fn cursors_are_bound_to_boot_and_filter() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        for _ in 0..3 {
            let id = Uuid::new_v4().to_string();
            catalog
                .upsert(metadata(&id, &project, SessionState::Live), Instant::now())
                .unwrap();
        }
        let page = catalog
            .page(&PageQuery {
                scope_id: "scope:one".into(),
                project_id: Some(project),
                cursor: None,
                limit: Some(1),
            })
            .unwrap();
        let cursor = page.next.unwrap();
        let wrong_scope = catalog
            .page(&PageQuery {
                scope_id: "scope:two".into(),
                project_id: None,
                cursor: Some(cursor.clone()),
                limit: None,
            })
            .unwrap_err();
        assert_eq!(wrong_scope, CatalogError::Gap);
        let mut stale = cursor.clone();
        stale.boot_id = Uuid::new_v4().to_string();
        assert_eq!(
            catalog
                .page(&PageQuery {
                    scope_id: "scope:one".into(),
                    project_id: None,
                    cursor: Some(stale),
                    limit: None,
                })
                .unwrap_err(),
            CatalogError::Gap
        );
    }

    #[test]
    fn watch_is_revision_scoped_and_reports_ring_gaps() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        let start = catalog.watch(None, None);
        assert!(!start.gap && start.events.is_empty());
        let first = Uuid::new_v4().to_string();
        catalog
            .upsert(
                metadata(&first, &project, SessionState::Live),
                Instant::now(),
            )
            .unwrap();
        let page = catalog.watch(Some(&start.next), None);
        assert!(!page.gap);
        assert_eq!(page.events.len(), 1);
        assert_eq!(page.events[0].change, CatalogChange::Upsert);
        assert_eq!(page.next.revision, catalog.revision());

        for _ in 0..(MAX_WATCH_EVENTS + 1) {
            catalog.touch(&first).unwrap();
        }
        let gap = catalog.watch(Some(&page.next), None);
        assert!(gap.gap && gap.events.is_empty());
    }

    #[test]
    fn mru_tracks_foreground_activity_and_resume_excludes_ended_runs() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        let first = Uuid::new_v4().to_string();
        let second = Uuid::new_v4().to_string();
        let now = Instant::now();
        catalog
            .upsert(metadata(&first, &project, SessionState::Live), now)
            .unwrap();
        catalog
            .upsert(metadata(&second, &project, SessionState::Live), now)
            .unwrap();
        assert_eq!(
            catalog
                .resume("scope:one", Some(&project))
                .unwrap()
                .session_id,
            second
        );
        catalog.touch(&first).unwrap();
        assert_eq!(
            catalog
                .resume("scope:one", Some(&project))
                .unwrap()
                .session_id,
            first
        );
        let mut ended = metadata(&first, &project, SessionState::Exited);
        ended.processes_active = false;
        ended.exit_code = Some(0);
        catalog.upsert(ended, now).unwrap();
        assert_eq!(
            catalog
                .resume("scope:one", Some(&project))
                .unwrap()
                .session_id,
            second
        );
        assert_eq!(catalog.touch(&first), Err(CatalogError::NotLive));
        assert_eq!(
            catalog.resume("other-scope", None),
            Err(CatalogError::NotFound)
        );
    }

    #[test]
    fn state_age_changes_only_when_state_changes() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        let start = Instant::now();
        catalog
            .upsert(metadata(&id, &project, SessionState::Live), start)
            .unwrap();
        let later = start + Duration::from_secs(5);
        let mut live = metadata(&id, &project, SessionState::Live);
        live.label = Some("renamed".into());
        catalog.upsert(live, later).unwrap();
        let inspection = catalog
            .inspect(
                &InspectQuery {
                    scope_id: "scope:one".into(),
                    project_id: project.clone(),
                    run_id: None,
                    session_id: id.clone(),
                },
                later,
            )
            .unwrap();
        assert_eq!(inspection.state_age, Duration::from_secs(5));
        let mut ended = metadata(&id, &project, SessionState::Exited);
        ended.processes_active = false;
        catalog.upsert(ended, later).unwrap();
        let inspection = catalog
            .inspect(
                &InspectQuery {
                    scope_id: "scope:one".into(),
                    project_id: project,
                    run_id: None,
                    session_id: id,
                },
                later + Duration::from_secs(2),
            )
            .unwrap();
        assert_eq!(inspection.state_age, Duration::from_secs(2));
    }

    #[test]
    fn entries_and_events_are_bounded() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        for _ in 0..MAX_SESSIONS {
            let id = Uuid::new_v4().to_string();
            catalog
                .upsert(metadata(&id, &project, SessionState::Live), Instant::now())
                .unwrap();
        }
        assert_eq!(
            catalog.upsert(
                metadata(&Uuid::new_v4().to_string(), &project, SessionState::Live),
                Instant::now()
            ),
            Err(CatalogError::Capacity)
        );
        assert!(catalog.events.len() <= MAX_WATCH_EVENTS);
    }

    #[test]
    fn foreign_scope_and_mismatched_runs_are_not_inspectable() {
        let mut catalog = catalog();
        let project = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        catalog
            .upsert(metadata(&id, &project, SessionState::Live), Instant::now())
            .unwrap();
        let mut query = InspectQuery {
            scope_id: "scope:one".into(),
            project_id: project,
            run_id: None,
            session_id: id,
        };
        assert!(catalog.inspect(&query, Instant::now()).is_ok());
        query.scope_id = "scope:two".into();
        assert_eq!(
            catalog.inspect(&query, Instant::now()).unwrap_err(),
            CatalogError::NotFound
        );
        query.scope_id = "scope:one".into();
        query.run_id = Some(Uuid::new_v4().to_string());
        assert_eq!(
            catalog.inspect(&query, Instant::now()).unwrap_err(),
            CatalogError::NotFound
        );
    }
}
