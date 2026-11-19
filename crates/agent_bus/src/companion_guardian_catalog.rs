//! Durable metadata for guarded sessions, opened only under the guardian lock.
use super::{terminals, unix_millis, CatalogHandle};
use diesel::{
    Connection, QueryableByName, RunQueryDsl,
    connection::SimpleConnection,
    sql_query,
    sql_types::{BigInt, Integer, Nullable, Text},
    sqlite::SqliteConnection,
};
use std::{collections::VecDeque, path::Path};

pub const APPLICATION_ID: i32 = 0x5747_4341;
pub const USER_VERSION: i32 = 1;
const MAX_ROWS: i64 = 64;
const MAX_EVENTS: usize = 64;
/// Resolved ended rows are catalog history, never retained work.
pub const ENDED_TTL_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Upserted,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub revision: u64,
    pub row: Record,
    pub change: Change,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchBatch {
    pub events: Vec<(Change, Record)>,
    pub next: u64,
    /// Events below the retained window were lost; the caller must relist.
    pub gap: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Live,
    Ended,
    Unknown,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Ended => "ended",
            Self::Unknown => "unknown",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "live" => Some(Self::Live),
            "ended" => Some(Self::Ended),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub session_id: String,
    pub run_id: String,
    pub boot_id: String,
    pub service_id: String,
    pub scope_id: String,
    pub project_id: String,
    pub policy: String,
    pub kind: String,
    pub label: Option<String>,
    pub created_seq: u64,
    pub mru_seq: u64,
    pub state: State,
    pub observed_ms: i64,
    pub exit_status: Option<i32>,
    /// Last observed writer epoch; takeover presents this exact generation.
    pub attachment_generation: u64,
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

impl Record {
    fn valid(&self) -> bool {
        valid_text(&self.session_id, 64)
            && valid_text(&self.run_id, 64)
            && valid_text(&self.boot_id, 64)
            && valid_text(&self.service_id, 64)
            && valid_text(&self.scope_id, 64)
            && valid_text(&self.project_id, 256)
            && valid_text(&self.policy, 32)
            && valid_text(&self.kind, 32)
            && self
                .label
                .as_deref()
                .is_none_or(|label| valid_text(label, 128))
    }
}

pub struct Catalog {
    connection: SqliteConnection,
    next_seq: i64,
    events: VecDeque<Event>,
    event_floor: u64,
}

#[derive(QueryableByName)]
struct SessionId {
    #[diesel(sql_type = Text)]
    session_id: String,
}

#[derive(QueryableByName)]
struct PragmaValue {
    #[diesel(sql_type = Integer)]
    value: i32,
}

#[derive(QueryableByName)]
struct RowCount {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct StoredRow {
    #[diesel(sql_type = Text)]
    session_id: String,
    #[diesel(sql_type = Text)]
    run_id: String,
    #[diesel(sql_type = Text)]
    boot_id: String,
    #[diesel(sql_type = Text)]
    service_id: String,
    #[diesel(sql_type = Text)]
    scope_id: String,
    #[diesel(sql_type = Text)]
    project_id: String,
    #[diesel(sql_type = Text)]
    policy: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    label: Option<String>,
    #[diesel(sql_type = BigInt)]
    created_seq: i64,
    #[diesel(sql_type = BigInt)]
    mru_seq: i64,
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = BigInt)]
    observed_ms: i64,
    #[diesel(sql_type = Nullable<Integer>)]
    exit_status: Option<i32>,
    #[diesel(sql_type = BigInt)]
    attachment_generation: i64,
}

fn unavailable(_: diesel::result::Error) -> String {
    "Guardian catalog unavailable".into()
}

fn pragma(connection: &mut SqliteConnection, name: &str) -> Result<i32, diesel::result::Error> {
    let query = format!("SELECT {name} AS value FROM pragma_{name}");
    Ok(sql_query(&query)
        .get_result::<PragmaValue>(connection)?
        .value)
}

impl Catalog {
    /// Opens an existing catalog only after validating its identity; never overwrites.
    pub fn open(path: &Path) -> Result<Self, String> {
        let existing = path.is_file();
        let mut connection = SqliteConnection::establish(&path.to_string_lossy())
            .map_err(|_| "Guardian catalog unavailable".to_owned())?;
        if existing {
            if pragma(&mut connection, "application_id").map_err(unavailable)? != APPLICATION_ID
                || pragma(&mut connection, "user_version").map_err(unavailable)? != USER_VERSION
            {
                return Err("Guardian catalog identity mismatch".into());
            }
            let columns = sql_query(
                "SELECT COUNT(*) AS count FROM pragma_table_info('guardian_catalog_sessions') \
                 WHERE name IN ('session_id','run_id','boot_id','service_id','scope_id','project_id',\
                   'policy','kind','label','created_seq','mru_seq','state','observed_ms',\
                   'exit_status','attachment_generation')",
            )
            .get_result::<RowCount>(&mut connection)
            .map_err(unavailable)?;
            if columns.count != 15 {
                return Err("Guardian catalog schema mismatch".into());
            }
        } else {
            // Identity, version and schema land together or not at all.
            connection
                .batch_execute(&format!(
                    "BEGIN IMMEDIATE;\n\
                     PRAGMA application_id = {APPLICATION_ID};\n\
                     PRAGMA user_version = {USER_VERSION};\n\
                     CREATE TABLE guardian_catalog_sessions(\
                       session_id TEXT PRIMARY KEY NOT NULL,\
                       run_id TEXT NOT NULL,\
                       boot_id TEXT NOT NULL,\
                       service_id TEXT NOT NULL,\
                       scope_id TEXT NOT NULL,\
                       project_id TEXT NOT NULL,\
                       policy TEXT NOT NULL,\
                       kind TEXT NOT NULL,\
                       label TEXT,\
                       created_seq INTEGER NOT NULL,\
                       mru_seq INTEGER NOT NULL,\
                       state TEXT NOT NULL,\
                       observed_ms INTEGER NOT NULL,\
                       exit_status INTEGER,\
                       attachment_generation INTEGER NOT NULL);\n\
                     COMMIT"
                ))
                .map_err(|_| "Guardian catalog initialization failed".to_owned())?;
        }
        let seeded = sql_query(
            "SELECT COALESCE(MAX(created_seq), 0) AS count FROM guardian_catalog_sessions",
        )
        .get_result::<RowCount>(&mut connection)
        .map_err(unavailable)?
        .count;
        let mru = sql_query(
            "SELECT COALESCE(MAX(mru_seq), 0) AS count FROM guardian_catalog_sessions",
        )
        .get_result::<RowCount>(&mut connection)
        .map_err(unavailable)?
        .count;
        let mut catalog = Self {
            connection,
            next_seq: seeded.max(mru),
            events: VecDeque::new(),
            event_floor: seeded.max(mru).max(0) as u64,
        };
        // Existing rows must satisfy the same invariants before any write.
        catalog.list()?;
        Ok(catalog)
    }

    /// Serially allocated by the single account owner; never derived from wall clocks.
    pub fn allocate_seq(&mut self) -> Result<u64, String> {
        let next = self
            .next_seq
            .checked_add(1)
            .filter(|next| *next > 0)
            .ok_or_else(|| "Guardian catalog sequence exhausted".to_owned())?;
        self.next_seq = next;
        u64::try_from(next).map_err(|_| "Guardian catalog sequence exhausted".to_owned())
    }

    /// Mutations allocate a revision even when they do not consume a session sequence.
    fn bump_revision(&mut self) -> u64 {
        if let Ok(revision) = self.allocate_seq() {
            return revision;
        }
        self.next_seq = i64::MAX;
        self.revision()
    }

    fn record_event(&mut self, row: Record, change: Change) {
        let revision = self.bump_revision();
        self.events.push_back(Event { revision, row, change });
        if self.events.len() > MAX_EVENTS {
            self.event_floor = self.events.pop_front().expect("event ring exceeds its capacity").revision;
        }
    }

    fn record_upsert(&mut self, session_id: &str) {
        match self.find_session(session_id) {
            Ok(Some(row)) => self.record_event(row, Change::Upserted),
            _ => {
                self.events.clear();
                self.event_floor = self.bump_revision();
            }
        }
    }

    /// Prior live rows are interruption evidence, never auto-resumed work.
    pub fn begin_boot(&mut self, observed_ms: i64) -> Result<usize, diesel::result::Error> {
        sql_query(
            "UPDATE guardian_catalog_sessions SET state='unknown', observed_ms=? \
             WHERE state='live'",
        )
        .bind::<BigInt, _>(observed_ms)
        .execute(&mut self.connection)
    }

    fn count(&mut self) -> Result<i64, diesel::result::Error> {
        Ok(
            sql_query("SELECT COUNT(*) AS count FROM guardian_catalog_sessions")
                .get_result::<RowCount>(&mut self.connection)?
                .count,
        )
    }

    /// Bounds durable rows; unresolved uncertainty fails admission instead of eviction.
    pub fn insert(&mut self, record: &Record) -> Result<(), String> {
        if !record.valid() {
            return Err("Invalid guardian catalog record".into());
        }
        let created: i64 = record
            .created_seq
            .try_into()
            .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
        let mru: i64 = record
            .mru_seq
            .try_into()
            .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
        if self.count().map_err(unavailable)? >= MAX_ROWS {
            let overflow = self.count().map_err(unavailable)? - MAX_ROWS + 1;
            let mut pruned = self.list()?;
            pruned.retain(|row| row.state == State::Ended);
            pruned.sort_by(|a, b| a.created_seq.cmp(&b.created_seq).then(a.session_id.cmp(&b.session_id)));
            pruned.truncate(overflow as usize);
            let removed = sql_query(
                "DELETE FROM guardian_catalog_sessions WHERE session_id IN (\
                   SELECT session_id FROM guardian_catalog_sessions WHERE state='ended' \
                   ORDER BY created_seq, session_id LIMIT ?)",
            )
            .bind::<BigInt, _>(overflow)
            .execute(&mut self.connection)
            .map_err(unavailable)?;
            if removed == 0 && self.count().map_err(unavailable)? >= MAX_ROWS {
                return Err("Guardian catalog capacity reached".into());
            }
            for row in pruned {
                self.record_event(row, Change::Removed);
            }
        }
        sql_query(
            "INSERT INTO guardian_catalog_sessions(\
               session_id, run_id, boot_id, service_id, scope_id, project_id, policy, kind, label,\
               created_seq, mru_seq, state, observed_ms, exit_status, attachment_generation\
             ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind::<Text, _>(&record.session_id)
        .bind::<Text, _>(&record.run_id)
        .bind::<Text, _>(&record.boot_id)
        .bind::<Text, _>(&record.service_id)
        .bind::<Text, _>(&record.scope_id)
        .bind::<Text, _>(&record.project_id)
        .bind::<Text, _>(&record.policy)
        .bind::<Text, _>(&record.kind)
        .bind::<Nullable<Text>, _>(record.label.as_deref())
        .bind::<BigInt, _>(created)
        .bind::<BigInt, _>(mru)
        .bind::<Text, _>(record.state.as_str())
        .bind::<BigInt, _>(record.observed_ms)
        .bind::<Nullable<Integer>, _>(record.exit_status)
        .bind::<BigInt, _>(
            i64::try_from(record.attachment_generation)
                .map_err(|_| "Invalid guardian catalog sequence".to_owned())?,
        )
        .execute(&mut self.connection)
        .map_err(unavailable)?;
        self.record_upsert(&record.session_id);
        Ok(())
    }

    fn contains(&mut self, session_id: &str) -> Result<bool, diesel::result::Error> {
        Ok(sql_query(
            "SELECT COUNT(*) AS count FROM guardian_catalog_sessions WHERE session_id=?",
        )
        .bind::<Text, _>(session_id)
        .get_result::<RowCount>(&mut self.connection)?
        .count
            > 0)
    }

    /// Launch recording is retry-safe: an existing session ID updates instead of duplicating.
    pub fn upsert(&mut self, record: &Record) -> Result<(), String> {
        if !record.valid() {
            return Err("Invalid guardian catalog record".into());
        }
        if self.contains(&record.session_id).map_err(unavailable)? {
            let matches = sql_query(
                "SELECT COUNT(*) AS count FROM guardian_catalog_sessions \
                 WHERE session_id=? AND run_id=? AND boot_id=? AND service_id=? \
                   AND scope_id=? AND project_id=? AND policy=? AND kind=?",
            )
            .bind::<Text, _>(&record.session_id)
            .bind::<Text, _>(&record.run_id)
            .bind::<Text, _>(&record.boot_id)
            .bind::<Text, _>(&record.service_id)
            .bind::<Text, _>(&record.scope_id)
            .bind::<Text, _>(&record.project_id)
            .bind::<Text, _>(&record.policy)
            .bind::<Text, _>(&record.kind)
            .get_result::<RowCount>(&mut self.connection)
            .map_err(unavailable)?
            .count;
            if matches != 1 {
                return Err("Guardian catalog identity conflict".into());
            }
            let mru: i64 = record
                .mru_seq
                .try_into()
                .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
            let updated = sql_query(
                "UPDATE guardian_catalog_sessions SET label=?, \
                 state=CASE WHEN state='ended' THEN state ELSE ? END, observed_ms=?, \
                 mru_seq=MAX(mru_seq, ?), \
                 exit_status=CASE WHEN state='ended' THEN exit_status ELSE ? END, \
                 attachment_generation=? \
                 WHERE session_id=?",
            )
            .bind::<Nullable<Text>, _>(record.label.as_deref())
            .bind::<Text, _>(record.state.as_str())
            .bind::<BigInt, _>(record.observed_ms)
            .bind::<BigInt, _>(mru)
            .bind::<Nullable<Integer>, _>(record.exit_status)
            .bind::<BigInt, _>(
                i64::try_from(record.attachment_generation)
                    .map_err(|_| "Invalid guardian catalog sequence".to_owned())?,
            )
            .bind::<Text, _>(&record.session_id)
            .execute(&mut self.connection)
            .map_err(unavailable)?;
            if updated > 0 {
                self.record_upsert(&record.session_id);
            }
            return Ok(());
        }
        self.insert(record)
    }

    /// Exact identity prevents a reused session ID or stale run from mutating another run.
    pub fn update_state(
        &mut self,
        session_id: &str,
        run_id: &str,
        boot_id: &str,
        service_id: &str,
        state: State,
        observed_ms: i64,
        exit_status: Option<i32>,
    ) -> Result<usize, diesel::result::Error> {
        let updated = sql_query(
            "UPDATE guardian_catalog_sessions SET state=?, observed_ms=?, exit_status=? \
             WHERE session_id=? AND run_id=? AND boot_id=? AND service_id=? AND state<>'ended'",
        )
        .bind::<Text, _>(state.as_str())
        .bind::<BigInt, _>(observed_ms)
        .bind::<Nullable<Integer>, _>(exit_status)
        .bind::<Text, _>(session_id)
        .bind::<Text, _>(run_id)
        .bind::<Text, _>(boot_id)
        .bind::<Text, _>(service_id)
        .execute(&mut self.connection)?;
        if updated > 0 {
            self.record_upsert(session_id);
        }
        Ok(updated)
    }

    /// An older or replayed writer cannot reorder MRU or roll back the observed epoch.
    pub fn touch_attachment(
        &mut self,
        session_id: &str,
        run_id: &str,
        boot_id: &str,
        service_id: &str,
        generation: u64,
    ) -> Result<bool, String> {
        let Some(row) = self.find(session_id, run_id)? else {
            return Ok(false);
        };
        if row.boot_id != boot_id || row.service_id != service_id {
            return Ok(false);
        }
        if row.state == State::Ended || row.attachment_generation >= generation {
            return Ok(true);
        }
        let generation = i64::try_from(generation)
            .map_err(|_| "Invalid guardian attachment generation".to_owned())?;
        let mru = i64::try_from(self.allocate_seq()?)
            .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
        let updated = sql_query(
            "UPDATE guardian_catalog_sessions SET attachment_generation=?, mru_seq=?, state='live' \
             WHERE session_id=? AND run_id=? AND boot_id=? AND service_id=? \
               AND state<>'ended' AND attachment_generation<?",
        )
        .bind::<BigInt, _>(generation)
        .bind::<BigInt, _>(mru)
        .bind::<Text, _>(session_id)
        .bind::<Text, _>(run_id)
        .bind::<Text, _>(boot_id)
        .bind::<Text, _>(service_id)
        .bind::<BigInt, _>(generation)
        .execute(&mut self.connection)
        .map_err(unavailable)?;
        if updated > 0 {
            self.record_upsert(session_id);
        }
        Ok(updated == 1)
    }

    /// Foreground acquisition/selection only; listing and observation never call this.
    #[cfg(test)]
    fn touch_mru(&mut self, session_id: &str) -> Result<u64, String> {
        let mru = self.allocate_seq()?;
        let mru: i64 = mru
            .try_into()
            .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
        let updated = sql_query(
            "UPDATE guardian_catalog_sessions SET mru_seq=? \
             WHERE session_id=? AND state<>'ended'",
        )
        .bind::<BigInt, _>(mru)
        .bind::<Text, _>(session_id)
        .execute(&mut self.connection)
        .map_err(unavailable)?;
        if updated > 0 {
            self.record_upsert(session_id);
        }
        Ok(mru as u64)
    }

    pub fn remove(&mut self, session_id: &str) -> Result<usize, diesel::result::Error> {
        let row = self.find_session(session_id).map_err(|error| {
            diesel::result::Error::DeserializationError(error.into())
        })?;
        let removed = sql_query("DELETE FROM guardian_catalog_sessions WHERE session_id=?")
            .bind::<Text, _>(session_id)
            .execute(&mut self.connection)?;
        if let Some(row) = row {
            self.record_event(row, Change::Removed);
        }
        Ok(removed)
    }

    /// Only resolved ended rows expire; unresolved uncertainty stays until resolved.
    pub fn prune_ended(&mut self, cutoff_ms: i64) -> Result<usize, diesel::result::Error> {
        let expired = sql_query(
            "SELECT session_id FROM guardian_catalog_sessions \
             WHERE state='ended' AND observed_ms<?",
        )
        .bind::<BigInt, _>(cutoff_ms)
        .get_results::<SessionId>(&mut self.connection)?;
        let mut removed = 0;
        for row in expired {
            removed += self.remove(&row.session_id)?;
        }
        Ok(removed)
    }

    /// Cursor-fenced change replay; a lost window demands a fresh listing.
    pub fn watch(&self, after: u64, limit: usize, matches: impl Fn(&Record) -> bool) -> Result<WatchBatch, String> {
        let current = self.revision();
        if after > current {
            return Err("Guardian catalog cursor ahead".into());
        }
        if after == current {
            return Ok(WatchBatch {
                events: Vec::new(),
                next: current,
                gap: false,
            });
        }
        if after < self.event_floor {
            return Ok(WatchBatch {
                events: Vec::new(),
                next: current,
                gap: true,
            });
        }
        let mut events = Vec::new();
        let mut next = current;
        for event in self.events.iter().filter(|event| event.revision > after) {
            if matches(&event.row) {
                events.push((event.change, event.row.clone()));
                if events.len() == limit {
                    next = event.revision;
                    break;
                }
            }
        }
        Ok(WatchBatch {
            next,
            events,
            gap: false,
        })
    }

    #[cfg(test)]
    pub(super) fn set_query_only(&mut self, readonly: bool) {
        self.connection.batch_execute(if readonly {
            "PRAGMA query_only=ON"
        } else {
            "PRAGMA query_only=OFF"
        }).unwrap();
    }

    /// Monotonic account revision; pages bound to it become stale after any mutation.
    pub fn revision(&self) -> u64 {
        self.next_seq.max(0) as u64
    }

    pub fn find(
        &mut self,
        session_id: &str,
        run_id: &str,
    ) -> Result<Option<Record>, String> {
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.session_id == session_id && record.run_id == run_id))
    }

    fn find_session(&mut self, session_id: &str) -> Result<Option<Record>, String> {
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.session_id == session_id))
    }

    pub fn list(&mut self) -> Result<Vec<Record>, String> {
        let rows = sql_query(
            "SELECT session_id, run_id, boot_id, service_id, scope_id, project_id, policy, kind, label,\
             created_seq, mru_seq, state, observed_ms, exit_status, attachment_generation \
             FROM guardian_catalog_sessions \
             ORDER BY mru_seq DESC, created_seq DESC, session_id ASC",
        )
        .get_results::<StoredRow>(&mut self.connection)
        .map_err(unavailable)?;
        rows.into_iter()
            .map(|row| {
                let record = Record {
                    session_id: row.session_id,
                    run_id: row.run_id,
                    boot_id: row.boot_id,
                    service_id: row.service_id,
                    scope_id: row.scope_id,
                    project_id: row.project_id,
                    policy: row.policy,
                    kind: row.kind,
                    label: row.label,
                    created_seq: row.created_seq.try_into().ok()?,
                    mru_seq: row.mru_seq.try_into().ok()?,
                    state: State::parse(&row.state)?,
                    observed_ms: row.observed_ms,
                    exit_status: row.exit_status,
                    attachment_generation: row.attachment_generation.try_into().ok()?,
                };
                record.valid().then_some(record)
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| "Invalid guardian catalog row".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, state: State) -> Record {
        Record {
            session_id: id.into(),
            run_id: format!("run-{id}"),
            boot_id: "boot".into(),
            service_id: "service".into(),
            scope_id: "ssh".into(),
            project_id: "/project".into(),
            policy: "guarded".into(),
            kind: "shell".into(),
            label: Some("label".into()),
            created_seq: 1,
            mru_seq: 1,
            state,
            observed_ms: 1,
            exit_status: None,
            attachment_generation: 0,
        }
    }

    #[test]
    fn creates_validates_and_marks_prior_live_rows_unknown() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        catalog.insert(&record("a", State::Live)).unwrap();
        catalog.insert(&record("b", State::Ended)).unwrap();
        drop(catalog);
        let mut catalog = Catalog::open(&path).unwrap();
        assert_eq!(catalog.begin_boot(9).unwrap(), 1);
        let rows = catalog.list().unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| row.state == State::Unknown));
        assert!(rows.iter().any(|row| row.state == State::Ended));
        catalog.touch_mru("a").unwrap();
        assert_eq!(catalog.list().unwrap()[0].mru_seq, 2);
    }

    #[test]
    fn sequence_is_serial_and_identity_checked() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        assert_eq!(catalog.allocate_seq().unwrap(), 1);
        assert_eq!(catalog.allocate_seq().unwrap(), 2);
        let mut first = record("a", State::Live);
        first.created_seq = 2;
        first.mru_seq = 2;
        catalog.insert(&first).unwrap();
        drop(catalog);
        let mut catalog = Catalog::open(&path).unwrap();
        assert_eq!(catalog.allocate_seq().unwrap(), 3);

        let mut reused = record("a", State::Live);
        reused.run_id = "other-run".into();
        assert!(catalog.upsert(&reused).is_err());
        assert_eq!(
            catalog
                .update_state("a", "other-run", "boot", "service", State::Ended, 5, Some(0))
                .unwrap(),
            0
        );
        assert_eq!(
            catalog
                .update_state("a", "run-a", "boot", "service", State::Ended, 5, Some(0))
                .unwrap(),
            1
        );
        assert_eq!(
            catalog
                .update_state("a", "run-a", "boot", "service", State::Live, 6, None)
                .unwrap(),
            0
        );
    }

    #[test]
    fn upsert_updates_without_duplicating_and_keeps_peak_mru() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        let mut first = record("a", State::Live);
        first.mru_seq = 5;
        catalog.upsert(&first).unwrap();
        let mut retry = record("a", State::Ended);
        retry.mru_seq = 2;
        retry.exit_status = Some(0);
        catalog.upsert(&retry).unwrap();
        let rows = catalog.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].state, State::Ended);
        assert_eq!(rows[0].mru_seq, 5);
        assert_eq!(rows[0].exit_status, Some(0));
    }

    #[test]
    fn refuses_unknown_identity_without_overwriting() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut connection = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        connection
            .batch_execute("CREATE TABLE foreign_table(id INTEGER)")
            .unwrap();
        drop(connection);
        assert!(Catalog::open(&path).is_err());
        let mut connection = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        assert_eq!(
            sql_query("SELECT COUNT(*) AS count FROM foreign_table")
                .get_result::<RowCount>(&mut connection)
                .unwrap()
                .count,
            0
        );
    }

    #[test]
    fn bounds_rows_and_refuses_unresolved_capacity() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        for index in 0..MAX_ROWS {
            catalog
                .insert(&record(&format!("live-{index}"), State::Live))
                .unwrap();
        }
        assert!(catalog.insert(&record("overflow", State::Live)).is_err());
        catalog
            .update_state("live-0", "run-live-0", "boot", "service", State::Ended, 2, Some(0))
            .unwrap();
        catalog.insert(&record("new", State::Live)).unwrap();
        assert!(
            !catalog
                .list()
                .unwrap()
                .iter()
                .any(|row| row.session_id == "live-0")
        );
    }

    #[test]
    fn watch_replays_bounded_events_and_reports_gaps() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        let mut first = record("a", State::Live);
        first.created_seq = 1;
        first.mru_seq = 1;
        catalog.insert(&first).unwrap();
        let baseline = catalog.revision();
        catalog.allocate_seq().unwrap();
        assert!(catalog.watch(baseline, 16, |_| true).unwrap().events.is_empty());
        catalog
            .update_state("a", "run-a", "boot", "service", State::Ended, 2, Some(0))
            .unwrap();
        let batch = catalog.watch(baseline, 16, |_| true).unwrap();
        assert!(!batch.gap);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].0, Change::Upserted);
        assert_eq!(
            batch.events[0].1.state,
            State::Ended
        );
        assert_eq!(batch.next, catalog.revision());
        assert!(catalog.watch(batch.next, 16, |_| true).unwrap().events.is_empty());

        catalog.remove("a").unwrap();
        let removal = catalog.watch(batch.next, 16, |_| true).unwrap();
        assert!(!removal.gap);
        assert!(catalog.watch(batch.next, 16, |row| row.project_id == "other")
            .unwrap().events.is_empty());
        assert_eq!(removal.events.len(), 1);
        assert_eq!(removal.events[0].0, Change::Removed);
        assert_eq!(removal.events[0].1.session_id, "a");
        catalog.insert(&record("c", State::Live)).unwrap();
        let stale = catalog.revision();
        for observed in 0..=MAX_EVENTS as i64 {
            catalog
                .update_state("c", "run-c", "boot", "service", State::Live, observed, None)
                .unwrap();
        }
        assert!(catalog.watch(stale, 16, |_| true).unwrap().gap);
        assert!(catalog.watch(catalog.revision() + 1, 16, |_| true).is_err());
    }

    #[test]
    fn expires_only_resolved_ended_rows() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        let mut live = record("live", State::Live);
        live.observed_ms = 1;
        catalog.insert(&live).unwrap();
        let mut ended = record("ended", State::Ended);
        ended.observed_ms = 1;
        catalog.insert(&ended).unwrap();
        assert_eq!(catalog.prune_ended(10).unwrap(), 1);
        let rows = catalog.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session_id, "live");
    }

    #[test]
    fn invalid_rows_and_write_faults_surface_as_errors() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        let mut catalog = Catalog::open(&path).unwrap();
        let mut invalid = record("a", State::Live);
        invalid.scope_id.clear();
        assert!(catalog.insert(&invalid).is_err());
        catalog
            .connection
            .batch_execute("PRAGMA query_only = ON")
            .unwrap();
        assert!(catalog.insert(&record("b", State::Live)).is_err());
        catalog
            .connection
            .batch_execute("PRAGMA query_only = OFF")
            .unwrap();
    }

    #[test]
    fn schema_identity_mismatch_refuses_open() {
        let state = tempfile::tempdir().unwrap();
        let path = state.path().join("catalog.sqlite3");
        drop(Catalog::open(&path).unwrap());
        let mut connection = SqliteConnection::establish(&path.to_string_lossy()).unwrap();
        connection
            .batch_execute("PRAGMA user_version = 9")
            .unwrap();
        drop(connection);
        assert!(Catalog::open(&path).is_err());
    }
}

pub(super) fn sync_attachment(
    terminals: &terminals::Terminals,
    catalog: &CatalogHandle,
    attachment: &terminals::PendingAttachment,
) {
    let recorded = catalog.try_lock().is_ok_and(|mut catalog| {
        catalog.touch_attachment(
            &attachment.session_id,
            &attachment.run_id,
            &attachment.boot_id,
            &attachment.service_id,
            attachment.generation,
        ).unwrap_or(false)
    });
    if recorded {
        terminals.mark_attachment_reported(
            &attachment.session_id, &attachment.run_id, attachment.generation,
        );
    }
}

/// Sole lifecycle authority reports observed exit once, independent of client reads.
pub(super) fn sync_ended(terminals: &terminals::Terminals, catalog: &CatalogHandle) {
    let ended = terminals.ended_guarded();
    if ended.is_empty() {
        return;
    }
    let observed = unix_millis();
    for session in ended {
        let recorded = catalog.lock().is_ok_and(|mut catalog| {
            catalog
                .update_state(
                    &session.session_id,
                    &session.run_id,
                    &session.boot_id,
                    &session.service_id,
                    State::Ended,
                    observed,
                    session.exit_code,
                )
                .is_ok()
        });
        if recorded {
            terminals.mark_end_reported(&session.session_id, &session.run_id);
        }
    }
}
