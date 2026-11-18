//! Durable metadata for guarded sessions, opened only under the guardian lock.
#![cfg(unix)]
use diesel::{
    Connection, QueryableByName, RunQueryDsl,
    connection::SimpleConnection,
    sql_query,
    sql_types::{BigInt, Integer, Nullable, Text},
    sqlite::SqliteConnection,
};
use std::path::Path;

pub const APPLICATION_ID: i32 = 0x5747_4341;
pub const USER_VERSION: i32 = 1;
const MAX_ROWS: i64 = 64;

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
    pub project_id: String,
    pub policy: String,
    pub kind: String,
    pub label: Option<String>,
    pub created_seq: u64,
    pub mru_seq: u64,
    pub state: State,
    pub observed_ms: i64,
    pub exit_status: Option<i32>,
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
            let tables = sql_query(
                "SELECT COUNT(*) AS count FROM sqlite_master \
                 WHERE type='table' AND name='guardian_catalog_sessions'",
            )
            .get_result::<RowCount>(&mut connection)
            .map_err(unavailable)?;
            if tables.count != 1 {
                return Err("Guardian catalog schema mismatch".into());
            }
        } else {
            connection
                .batch_execute(&format!(
                    "PRAGMA application_id = {APPLICATION_ID};\n\
                     PRAGMA user_version = {USER_VERSION};\n\
                     CREATE TABLE guardian_catalog_sessions(\
                       session_id TEXT PRIMARY KEY NOT NULL,\
                       run_id TEXT NOT NULL,\
                       boot_id TEXT NOT NULL,\
                       service_id TEXT NOT NULL,\
                       project_id TEXT NOT NULL,\
                       policy TEXT NOT NULL,\
                       kind TEXT NOT NULL,\
                       label TEXT,\
                       created_seq INTEGER NOT NULL,\
                       mru_seq INTEGER NOT NULL,\
                       state TEXT NOT NULL,\
                       observed_ms INTEGER NOT NULL,\
                       exit_status INTEGER)"
                ))
                .map_err(|_| "Guardian catalog initialization failed".to_owned())?;
        }
        Ok(Self { connection })
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
            let removed = sql_query(
                "DELETE FROM guardian_catalog_sessions WHERE session_id IN (\
                   SELECT session_id FROM guardian_catalog_sessions WHERE state='ended' \
                   ORDER BY created_seq LIMIT ?)",
            )
            .bind::<BigInt, _>(self.count().map_err(unavailable)? - MAX_ROWS + 1)
            .execute(&mut self.connection)
            .map_err(unavailable)?;
            if removed == 0 && self.count().map_err(unavailable)? >= MAX_ROWS {
                return Err("Guardian catalog capacity reached".into());
            }
        }
        sql_query(
            "INSERT INTO guardian_catalog_sessions(\
               session_id, run_id, boot_id, service_id, project_id, policy, kind, label,\
               created_seq, mru_seq, state, observed_ms, exit_status\
             ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind::<Text, _>(&record.session_id)
        .bind::<Text, _>(&record.run_id)
        .bind::<Text, _>(&record.boot_id)
        .bind::<Text, _>(&record.service_id)
        .bind::<Text, _>(&record.project_id)
        .bind::<Text, _>(&record.policy)
        .bind::<Text, _>(&record.kind)
        .bind::<Nullable<Text>, _>(record.label.as_deref())
        .bind::<BigInt, _>(created)
        .bind::<BigInt, _>(mru)
        .bind::<Text, _>(record.state.as_str())
        .bind::<BigInt, _>(record.observed_ms)
        .bind::<Nullable<Integer>, _>(record.exit_status)
        .execute(&mut self.connection)
        .map_err(unavailable)?;
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
            let mru: i64 = record
                .mru_seq
                .try_into()
                .map_err(|_| "Invalid guardian catalog sequence".to_owned())?;
            sql_query(
                "UPDATE guardian_catalog_sessions SET label=?, \
                 state=CASE WHEN state='ended' THEN state ELSE ? END, observed_ms=?, \
                 mru_seq=MAX(mru_seq, ?), \
                 exit_status=CASE WHEN state='ended' THEN exit_status ELSE ? END \
                 WHERE session_id=?",
            )
            .bind::<Nullable<Text>, _>(record.label.as_deref())
            .bind::<Text, _>(record.state.as_str())
            .bind::<BigInt, _>(record.observed_ms)
            .bind::<BigInt, _>(mru)
            .bind::<Nullable<Integer>, _>(record.exit_status)
            .bind::<Text, _>(&record.session_id)
            .execute(&mut self.connection)
            .map_err(unavailable)?;
            return Ok(());
        }
        self.insert(record)
    }

    pub fn update_state(
        &mut self,
        session_id: &str,
        state: State,
        observed_ms: i64,
        exit_status: Option<i32>,
    ) -> Result<usize, diesel::result::Error> {
        sql_query(
            "UPDATE guardian_catalog_sessions SET state=?, observed_ms=?, exit_status=? \
             WHERE session_id=?",
        )
        .bind::<Text, _>(state.as_str())
        .bind::<BigInt, _>(observed_ms)
        .bind::<Nullable<Integer>, _>(exit_status)
        .bind::<Text, _>(session_id)
        .execute(&mut self.connection)
    }

    /// Foreground acquisition/selection only; listing and observation never call this.
    pub fn touch_mru(
        &mut self,
        session_id: &str,
        mru_seq: u64,
    ) -> Result<usize, diesel::result::Error> {
        let mru: i64 = mru_seq.try_into().unwrap_or(i64::MAX);
        sql_query("UPDATE guardian_catalog_sessions SET mru_seq=? WHERE session_id=?")
            .bind::<BigInt, _>(mru)
            .bind::<Text, _>(session_id)
            .execute(&mut self.connection)
    }

    pub fn remove(&mut self, session_id: &str) -> Result<usize, diesel::result::Error> {
        sql_query("DELETE FROM guardian_catalog_sessions WHERE session_id=?")
            .bind::<Text, _>(session_id)
            .execute(&mut self.connection)
    }

    pub fn list(&mut self) -> Result<Vec<Record>, String> {
        let rows = sql_query(
            "SELECT session_id, run_id, boot_id, service_id, project_id, policy, kind, label,\
             created_seq, mru_seq, state, observed_ms, exit_status \
             FROM guardian_catalog_sessions ORDER BY mru_seq DESC, created_seq DESC",
        )
        .get_results::<StoredRow>(&mut self.connection)
        .map_err(unavailable)?;
        rows.into_iter()
            .map(|row| {
                Some(Record {
                    session_id: row.session_id,
                    run_id: row.run_id,
                    boot_id: row.boot_id,
                    service_id: row.service_id,
                    project_id: row.project_id,
                    policy: row.policy,
                    kind: row.kind,
                    label: row.label,
                    created_seq: row.created_seq.try_into().ok()?,
                    mru_seq: row.mru_seq.try_into().ok()?,
                    state: State::parse(&row.state)?,
                    observed_ms: row.observed_ms,
                    exit_status: row.exit_status,
                })
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
            project_id: "/project".into(),
            policy: "guarded".into(),
            kind: "shell".into(),
            label: Some("label".into()),
            created_seq: 1,
            mru_seq: 1,
            state,
            observed_ms: 1,
            exit_status: None,
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
        catalog.touch_mru("a", 7).unwrap();
        assert_eq!(catalog.list().unwrap()[0].mru_seq, 7);
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
            .update_state("live-0", State::Ended, 2, Some(0))
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
}
