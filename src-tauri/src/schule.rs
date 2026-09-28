use crate::db::{now_ms, AppState};
use crate::error::{Error, Result};
use crate::models::AddSourceInput;
use crate::repo;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

const LEGACY_SUBJECT: &str = "Schule";
const WEEKDAYS: [&str; 5] = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag"];
const UNSORTED: &str = "Unsortiert";
const LOOSE_FILES_TOPIC: &str = "Dateien";
const SETTING_KEY: &str = "school_folder_path";

fn find_subject_by_name(conn: &Connection, name: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT id FROM subjects WHERE name=?1 AND archived=0",
            params![name],
            |r| r.get(0),
        )
        .optional()?)
}

fn find_topic_by_name(conn: &Connection, subject_id: &str, name: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT id FROM topics WHERE subject_id=?1 AND name=?2",
            params![subject_id, name],
            |r| r.get(0),
        )
        .optional()?)
}

struct Existing {
    id: String,
    subject_id: String,
    topic_id: Option<String>,
    kind: String,
    stored_path: Option<String>,
    created_at: i64,
}

fn find_source_by_origin(
    conn: &Connection,
    subject_ids: &[String],
    origin: &str,
) -> Result<Option<Existing>> {
    let mut stmt = conn.prepare(
        "SELECT id, subject_id, topic_id, kind, stored_path, created_at FROM sources WHERE origin=?1",
    )?;
    let rows = stmt.query_map(params![origin], |r| {
        Ok(Existing {
            id: r.get(0)?,
            subject_id: r.get(1)?,
            topic_id: r.get(2)?,
            kind: r.get(3)?,
            stored_path: r.get(4)?,
            created_at: r.get(5)?,
        })
    })?;
    for row in rows {
        let row = row?;
        if subject_ids.contains(&row.subject_id) {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

async fn refresh_source(app: &AppHandle, src: &Existing, path: &Path, mtime: i64) -> Result<()> {
    let state = app.state::<AppState>();
    {
        let c = state.db.lock().unwrap();
        match src.kind.as_str() {
            "pdf" | "image" | "audio" => {
                if let Some(p) = &src.stored_path {
                    std::fs::copy(path, p)?;
                }
            }
            "docx" | "pptx" | "xlsx" => {
                if let Some(p) = &src.stored_path {
                    let _ = std::fs::remove_file(p);
                }
                c.execute("UPDATE sources SET stored_path=NULL WHERE id=?1", params![src.id])?;
            }
            "txt" | "md" => {
                c.execute("UPDATE sources SET content=NULL WHERE id=?1", params![src.id])?;
            }
            _ => {}
        }
    }
    let res = crate::commands::reingest_source(app.clone(), src.id.clone()).await;
    let c = state.db.lock().unwrap();
    set_source_timestamp(&c, &src.id, mtime)?;
    res.map(|_| ())
}

fn set_source_timestamp(conn: &Connection, id: &str, ts_ms: i64) -> Result<()> {
    conn.execute(
        "UPDATE sources SET created_at=?2, updated_at=?2 WHERE id=?1",
        params![id, ts_ms],
    )?;
    Ok(())
}

fn file_mtime_ms(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or_else(now_ms)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("~$") || n.starts_with('.'))
        .unwrap_or(false)
}

fn list_entries(dir: &Path, want_dirs: bool) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|read| {
            read.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir() == want_dirs && !is_hidden(p))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SchoolSyncResult {
    pub added: i64,
    pub updated: i64,
    pub moved: i64,
    pub removed: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

static SYNC_RUNNING: AtomicBool = AtomicBool::new(false);

struct RunningGuard;

impl Drop for RunningGuard {
    fn drop(&mut self) {
        SYNC_RUNNING.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
pub async fn sync_school_folder(app: AppHandle) -> Result<SchoolSyncResult> {
    if SYNC_RUNNING.swap(true, Ordering::SeqCst) {
        return Err(Error::Other("school folder sync is already running".into()));
    }
    let _guard = RunningGuard;
    run_sync(&app).await
}

async fn run_sync(app: &AppHandle) -> Result<SchoolSyncResult> {
    let state = app.state::<AppState>();
    let mut result = SchoolSyncResult {
        added: 0,
        updated: 0,
        moved: 0,
        removed: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    let root = {
        let c = state.db.lock().unwrap();
        repo::get_setting(&c, SETTING_KEY)?
    };
    let Some(root) = root.filter(|s| !s.trim().is_empty()).map(PathBuf::from) else {
        return Ok(result);
    };
    if !root.is_dir() {
        return Err(Error::NotFound(format!(
            "school folder not found: {}",
            root.display()
        )));
    }

    let legacy_id = {
        let c = state.db.lock().unwrap();
        find_subject_by_name(&c, LEGACY_SUBJECT)?
    };
    let mut managed_ids: Vec<String> = legacy_id.iter().cloned().collect();

    let mut jobs: Vec<(PathBuf, String, String)> = Vec::new();
    let mut present_topics: Vec<(String, Vec<String>)> = Vec::new();
    for subject_name in WEEKDAYS.iter().copied().chain([UNSORTED]) {
        let subject_dir = root.join(subject_name);
        if !subject_dir.is_dir() {
            continue;
        }
        let mut topic_dirs: Vec<(String, PathBuf)> = list_entries(&subject_dir, true)
            .into_iter()
            .filter_map(|d| Some((d.file_name()?.to_str()?.to_string(), d)))
            .collect();
        topic_dirs.push((LOOSE_FILES_TOPIC.to_string(), subject_dir.clone()));
        let topics: Vec<(String, Vec<PathBuf>)> = topic_dirs
            .into_iter()
            .map(|(name, dir)| (name, list_entries(&dir, false)))
            .filter(|(_, files)| !files.is_empty())
            .collect();

        let c = state.db.lock().unwrap();
        let subject_id = match find_subject_by_name(&c, subject_name)? {
            Some(id) => id,
            None if topics.is_empty() => continue,
            None => {
                let glyph = if subject_name == UNSORTED { "📂" } else { "📅" };
                repo::insert_subject(&c, subject_name, None, Some(glyph), None)?
            }
        };
        managed_ids.push(subject_id.clone());
        present_topics.push((subject_id.clone(), topics.iter().map(|(n, _)| n.clone()).collect()));

        for (topic_name, files) in topics {
            let topic_id = match find_topic_by_name(&c, &subject_id, &topic_name)? {
                Some(id) => id,
                None => repo::insert_topic(&c, &subject_id, &topic_name, None, &[])?,
            };
            for f in files {
                jobs.push((f, subject_id.clone(), topic_id.clone()));
            }
        }
    }

    for (path, subject_id, topic_id) in jobs {
        let Some(origin) = path.to_str().map(str::to_string) else {
            result.errors.push(format!("non-UTF8 path: {}", path.display()));
            continue;
        };

        let existing = {
            let c = state.db.lock().unwrap();
            find_source_by_origin(&c, &managed_ids, &origin)?
        };
        if let Some(existing) = existing {
            let in_place = existing.subject_id == subject_id
                && existing.topic_id.as_deref() == Some(topic_id.as_str());
            if !in_place {
                let c = state.db.lock().unwrap();
                repo::move_source(&c, &existing.id, &subject_id, Some(&topic_id))?;
                result.moved += 1;
            }
            let mtime = file_mtime_ms(&path);
            if mtime > existing.created_at {
                match refresh_source(&app, &existing, &path, mtime).await {
                    Ok(()) => result.updated += 1,
                    Err(e) => result.errors.push(format!("{}: {e}", path.display())),
                }
            } else if in_place {
                result.skipped += 1;
            }
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Datei")
            .to_string();
        let input = AddSourceInput {
            subject_id,
            topic_id: Some(topic_id),
            name: Some(name),
            kind: None,
            text: None,
            path: Some(origin),
            url: None,
            tags: Vec::new(),
        };
        match crate::commands::add_source(app.clone(), input).await {
            Ok(added) => {
                let c = state.db.lock().unwrap();
                let _ = set_source_timestamp(&c, &added.source.id, file_mtime_ms(&path));
                result.added += 1;
            }
            Err(e) => result.errors.push(format!("{}: {e}", path.display())),
        }
    }

    {
        let c = state.db.lock().unwrap();
        for subject_id in &managed_ids {
            let sources: Vec<(String, Option<String>, Option<String>)> = c
                .prepare("SELECT id, origin, stored_path FROM sources WHERE subject_id=?1")?
                .query_map(params![subject_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for (id, origin, stored_path) in sources {
                let Some(origin) = origin.map(PathBuf::from) else { continue };
                if origin.starts_with(&root) && !origin.exists() {
                    repo::delete_source(&c, &id)?;
                    if let Some(p) = stored_path {
                        let _ = std::fs::remove_file(p);
                    }
                    result.removed += 1;
                }
            }
        }
        for (subject_id, names) in &present_topics {
            let empty: Vec<(String, String)> = c
                .prepare(
                    "SELECT t.id, t.name FROM topics t WHERE t.subject_id=?1 \
                     AND NOT EXISTS (SELECT 1 FROM sources s WHERE s.topic_id=t.id) \
                     AND NOT EXISTS (SELECT 1 FROM cheatsheets ch WHERE ch.topic_id=t.id)",
                )?
                .query_map(params![subject_id], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for (id, name) in empty {
                if !names.contains(&name) {
                    repo::delete_topic(&c, &id)?;
                    result.removed += 1;
                }
            }
        }
    }

    if let Some(legacy_id) = legacy_id {
        let c = state.db.lock().unwrap();
        let remaining: i64 = c.query_row(
            "SELECT COUNT(*) FROM sources WHERE subject_id=?1",
            params![legacy_id],
            |r| r.get(0),
        )?;
        if remaining == 0 {
            repo::set_subject_archived(&c, &legacy_id, true)?;
        }
    }

    Ok(result)
}
