use std::{
    collections::VecDeque,
    env,
    ffi::OsStr,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Body,
    extract::{Query, State, WebSocketUpgrade, ws::Message},
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use constant_time_eq::constant_time_eq;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{error, info};
use walkdir::WalkDir;

const VALIDATOR_VERSION: &str = "flac-reference-v1|ffmpeg-errors-v1";
const COOKIE_NAME: &str = "audio_integrity_session";

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
    library_root: PathBuf,
    import_roots: Vec<PathBuf>,
    admin_user: Arc<str>,
    admin_password: Arc<str>,
    session_token: Arc<str>,
    lidarr_token: Arc<str>,
    api_token: Option<Arc<str>>,
    scan: Arc<Mutex<ScanStatus>>,
    cancel: Arc<AtomicBool>,
    events: broadcast::Sender<RealtimeEvent>,
    scan_workers: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanStatus {
    run_id: Option<i64>,
    phase: String,
    mode: Option<String>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    total_files: u64,
    processed_files: u64,
    verified_files: u64,
    skipped_files: u64,
    corrupt_files: u64,
    suspect_files: u64,
    error_files: u64,
    total_bytes: u64,
    processed_bytes: u64,
    read_bytes: u64,
    current_path: Option<String>,
    current_validator: Option<String>,
    message: Option<String>,
    log: VecDeque<LogLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogLine {
    at: DateTime<Utc>,
    level: String,
    text: String,
}

#[derive(Clone, Serialize)]
struct RealtimeEvent {
    kind: &'static str,
    status: ScanStatus,
}

impl Default for ScanStatus {
    fn default() -> Self {
        Self {
            run_id: None,
            phase: "idle".into(),
            mode: None,
            started_at: None,
            finished_at: None,
            total_files: 0,
            processed_files: 0,
            verified_files: 0,
            skipped_files: 0,
            corrupt_files: 0,
            suspect_files: 0,
            error_files: 0,
            total_bytes: 0,
            processed_bytes: 0,
            read_bytes: 0,
            current_path: None,
            current_validator: None,
            message: None,
            log: VecDeque::new(),
        }
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ScanRequest {
    mode: String,
}

#[derive(Deserialize)]
struct ResultQuery {
    verdict: Option<String>,
    query: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Deserialize)]
struct VerifyRequest {
    path: String,
}

#[derive(Serialize)]
struct SessionResponse {
    authenticated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyResponse {
    accepted: bool,
    verdict: String,
    cached: bool,
    validator: String,
    authenticity: String,
    authenticity_message: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    known_files: u64,
    healthy_files: u64,
    corrupt_files: u64,
    suspect_files: u64,
    error_files: u64,
    checked_bytes: u64,
    last_completed_at: Option<String>,
    validator_version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileResult {
    path: String,
    size: u64,
    format: String,
    verdict: String,
    authenticity: String,
    checked_at: String,
    duration_ms: u64,
    message: String,
    source: String,
    mtime_ns: String,
    validator_version: String,
    authenticity_message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultsPage {
    items: Vec<FileResult>,
    total: u64,
    limit: u32,
    offset: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanRun {
    id: i64,
    mode: String,
    status: String,
    started_at: String,
    finished_at: Option<String>,
    total_files: u64,
    verified_files: u64,
    skipped_files: u64,
    corrupt_files: u64,
}

#[derive(Debug)]
struct Validation {
    verdict: String,
    validator: String,
    authenticity: String,
    authenticity_message: String,
    message: String,
    duration_ms: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let data_dir = PathBuf::from(env_var("DATA_DIR", "/data"));
    fs::create_dir_all(&data_dir).expect("create data directory");
    let db_path = data_dir.join("audio-integrity.sqlite3");
    init_db(&db_path).expect("initialize database");

    let (events, _) = broadcast::channel(256);
    let state = AppState {
        db_path,
        library_root: PathBuf::from(env_var("LIBRARY_ROOT", "/music")),
        import_roots: env_var("IMPORT_ROOTS", "/downloads,/manual-import")
            .split(',')
            .map(PathBuf::from)
            .collect(),
        admin_user: env_var("ADMIN_USER", "admin").into(),
        admin_password: required_env("ADMIN_PASSWORD").into(),
        session_token: required_env("SESSION_TOKEN").into(),
        lidarr_token: required_env("LIDARR_TOKEN").into(),
        api_token: env::var("API_TOKEN")
            .ok()
            .filter(|v| !v.is_empty())
            .map(Into::into),
        scan: Arc::new(Mutex::new(ScanStatus::default())),
        cancel: Arc::new(AtomicBool::new(false)),
        events,
        scan_workers: env_var("SCAN_WORKERS", "3")
            .parse::<usize>()
            .unwrap_or(3)
            .clamp(1, 4),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/metrics.js", get(metrics_script))
        .route("/app.js", get(script))
        .route("/favicon.ico", get(favicon))
        .route("/healthz", get(health))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/session", get(session))
        .route("/api/summary", get(summary))
        .route("/api/status", get(status))
        .route("/api/realtime", get(realtime))
        .route("/api/results", get(results))
        .route("/api/history", get(history))
        .route("/api/scans", post(start_scan))
        .route("/api/scans/cancel", post(cancel_scan))
        .route("/internal/lidarr/verify", post(verify_import))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address: SocketAddr = env_var("LISTEN_ADDR", "0.0.0.0:8080")
        .parse()
        .expect("valid LISTEN_ADDR");
    let listener = tokio::net::TcpListener::bind(address).await.expect("bind");
    info!(%address, "audio integrity listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

fn env_var(name: &str, fallback: &str) -> String {
    env::var(name).unwrap_or_else(|_| fallback.into())
}

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Response<Body> {
    asset(
        include_str!("../static/index.html"),
        "text/html; charset=utf-8",
    )
}

async fn styles() -> Response<Body> {
    asset(
        include_str!("../static/styles.css"),
        "text/css; charset=utf-8",
    )
}

async fn script() -> Response<Body> {
    asset(
        include_str!("../static/app.js"),
        "text/javascript; charset=utf-8",
    )
}

async fn metrics_script() -> Response<Body> {
    asset(
        include_str!("../static/metrics.js"),
        "text/javascript; charset=utf-8",
    )
}

async fn favicon() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn asset(content: &'static str, content_type: &'static str) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(content))
        .unwrap()
}

async fn health() -> &'static str {
    "ok"
}

async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    if secure_eq(&body.username, &state.admin_user)
        && secure_eq(&body.password, &state.admin_password)
    {
        let cookie = format!(
            "{COOKIE_NAME}={}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=86400",
            state.session_token
        );
        return (StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response();
    }
    (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
}

async fn logout() -> impl IntoResponse {
    (
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            format!("{COOKIE_NAME}=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0"),
        )],
    )
}

async fn session(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    Json(SessionResponse {
        authenticated: authorized(&headers, &state),
    })
}

async fn summary(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match load_summary(&state.db_path) {
        Ok(value) => Json(value).into_response(),
        Err(err) => internal_error(err),
    }
}

async fn status(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(state.scan.lock().unwrap().clone()).into_response()
}

async fn realtime(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    ws.on_upgrade(move |mut socket| async move {
        let mut receiver = state.events.subscribe();
        let initial = RealtimeEvent {
            kind: "status",
            status: state.scan.lock().unwrap().clone(),
        };
        if let Ok(payload) = serde_json::to_string(&initial)
            && socket.send(Message::Text(payload.into())).await.is_err()
        {
            return;
        }
        loop {
            tokio::select! {
                event = receiver.recv() => match event {
                    Ok(event) => {
                        if let Ok(payload) = serde_json::to_string(&event)
                            && socket.send(Message::Text(payload.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let snapshot = RealtimeEvent {
                            kind: "status",
                            status: state.scan.lock().unwrap().clone(),
                        };
                        if let Ok(payload) = serde_json::to_string(&snapshot)
                            && socket.send(Message::Text(payload.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                },
                incoming = socket.recv() => match incoming {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    })
    .into_response()
}

async fn results(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ResultQuery>,
) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match load_results(&state.db_path, &query) {
        Ok(value) => Json(value).into_response(),
        Err(err) => internal_error(err),
    }
}

async fn history(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match load_history(&state.db_path) {
        Ok(value) => Json(value).into_response(),
        Err(err) => internal_error(err),
    }
}

async fn start_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ScanRequest>,
) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if body.mode != "incremental" && body.mode != "full" {
        return (StatusCode::BAD_REQUEST, "mode must be incremental or full").into_response();
    }
    {
        let mut scan = state.scan.lock().unwrap();
        if scan.phase == "discovering" || scan.phase == "scanning" || scan.phase == "cancelling" {
            return (StatusCode::CONFLICT, "a scan is already running").into_response();
        }
        *scan = ScanStatus {
            phase: "discovering".into(),
            mode: Some(body.mode.clone()),
            started_at: Some(Utc::now()),
            ..Default::default()
        };
        push_log(&mut scan, "info", format!("{} scan started", body.mode));
    }
    publish_status(&state);
    state.cancel.store(false, Ordering::Relaxed);
    let worker_state = state.clone();
    let mode = body.mode;
    std::thread::Builder::new()
        .name("library-scan".into())
        .spawn(move || run_scan(worker_state, mode))
        .expect("spawn scan worker");
    StatusCode::ACCEPTED.into_response()
}

async fn cancel_scan(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&headers, &state) {
        return StatusCode::UNAUTHORIZED;
    }
    let accepted = {
        let mut scan = state.scan.lock().unwrap();
        if scan.phase == "discovering" || scan.phase == "scanning" {
            state.cancel.store(true, Ordering::Relaxed);
            scan.phase = "cancelling".into();
            true
        } else {
            false
        }
    };
    if accepted {
        publish_status(&state);
        StatusCode::ACCEPTED
    } else {
        StatusCode::CONFLICT
    }
}

async fn verify_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VerifyRequest>,
) -> impl IntoResponse {
    let supplied = headers
        .get("x-integrity-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !secure_eq(supplied, &state.lidarr_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let path = match allowed_import_path(&body.path, &state.import_roots) {
        Ok(path) => path,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(VerifyResponse {
                    accepted: false,
                    verdict: "rejected".into(),
                    cached: false,
                    validator: "path-policy".into(),
                    authenticity: "unknown".into(),
                    authenticity_message: "File was not analyzed".into(),
                    message,
                }),
            )
                .into_response();
        }
    };
    let db_path = state.db_path.clone();
    match tokio::task::spawn_blocking(move || verify_one(&db_path, &path, "lidarr", false)).await {
        Ok(Ok((result, cached))) => {
            let accepted = result.verdict == "healthy";
            let status = if accepted {
                StatusCode::OK
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (
                status,
                Json(VerifyResponse {
                    accepted,
                    verdict: result.verdict,
                    cached,
                    validator: result.validator,
                    authenticity: result.authenticity,
                    authenticity_message: result.authenticity_message,
                    message: result.message,
                }),
            )
                .into_response()
        }
        Ok(Err(err)) => internal_error(err),
        Err(err) => internal_error(err),
    }
}

fn run_scan(state: AppState, mode: String) {
    let started_at = Utc::now();
    let connection = match open_db(&state.db_path) {
        Ok(connection) => connection,
        Err(err) => return fail_scan(&state, format!("database: {err}")),
    };
    let run_id = match connection.execute(
        "INSERT INTO scan_runs (mode, status, started_at) VALUES (?1, 'running', ?2)",
        params![mode, started_at.to_rfc3339()],
    ) {
        Ok(_) => connection.last_insert_rowid(),
        Err(err) => return fail_scan(&state, format!("database: {err}")),
    };
    state.scan.lock().unwrap().run_id = Some(run_id);
    publish_status(&state);

    let files = discover_files(&state.library_root);
    let (files, total_bytes) = match files {
        Ok(files) => {
            let total_bytes = files.iter().map(|(_, size)| size).sum();
            (files, total_bytes)
        }
        Err(err) => return fail_scan(&state, err),
    };
    {
        let mut scan = state.scan.lock().unwrap();
        scan.phase = "scanning".into();
        scan.total_files = files.len() as u64;
        scan.total_bytes = total_bytes;
        push_log(
            &mut scan,
            "info",
            format!("Discovered {} audio files", files.len()),
        );
    }
    publish_status(&state);

    let worker_count = state.scan_workers.min(files.len().max(1));
    {
        let mut scan = state.scan.lock().unwrap();
        push_log(
            &mut scan,
            "info",
            format!("Checking with {worker_count} parallel workers"),
        );
    }
    publish_status(&state);

    let queue = Arc::new(Mutex::new(VecDeque::from(files)));
    let disk_reader = Arc::new(Mutex::new(()));
    let worker_error = Arc::new(Mutex::new(None));
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let state = state.clone();
            let queue = Arc::clone(&queue);
            let disk_reader = Arc::clone(&disk_reader);
            let worker_error = Arc::clone(&worker_error);
            let mode = mode.clone();
            scope.spawn(move || {
                let worker_connection = match open_db(&state.db_path) {
                    Ok(connection) => connection,
                    Err(err) => {
                        *worker_error.lock().unwrap() = Some(format!("database: {err}"));
                        return;
                    }
                };
                loop {
                    if state.cancel.load(Ordering::Relaxed) {
                        return;
                    }
                    let Some((path, size)) = queue.lock().unwrap().pop_front() else {
                        return;
                    };
                    process_scan_file(
                        &state,
                        &worker_connection,
                        &path,
                        size,
                        mode == "full",
                        &disk_reader,
                    );
                }
            });
        }
    });

    if let Some(err) = worker_error.lock().unwrap().take() {
        fail_scan(&state, err);
    } else if state.cancel.load(Ordering::Relaxed) {
        finish_scan(&state, &connection, "cancelled");
    } else {
        finish_scan(&state, &connection, "completed");
    }
}

fn process_scan_file(
    state: &AppState,
    connection: &Connection,
    path: &Path,
    size: u64,
    force: bool,
    disk_reader: &Mutex<()>,
) {
    let relative = path
        .strip_prefix(&state.library_root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    {
        let mut scan = state.scan.lock().unwrap();
        scan.current_path = Some(relative.clone());
        scan.current_validator = Some(validator_for(path).into());
    }
    publish_status(state);
    match verify_one_with_connection(connection, path, "library", force, Some(disk_reader)) {
        Ok((result, cached)) => {
            let mut scan = state.scan.lock().unwrap();
            scan.processed_files += 1;
            scan.processed_bytes += size;
            if cached {
                scan.skipped_files += 1;
            } else {
                scan.read_bytes += size;
                if result.authenticity == "likely_lossy" {
                    scan.suspect_files += 1;
                    push_log(
                        &mut scan,
                        "warning",
                        format!(
                            "Likely lossy source: {relative} — {}",
                            result.authenticity_message
                        ),
                    );
                }
                match result.verdict.as_str() {
                    "healthy" => scan.verified_files += 1,
                    "corrupt" => {
                        scan.corrupt_files += 1;
                        push_log(
                            &mut scan,
                            "error",
                            format!("Corrupt: {relative} — {}", result.message),
                        );
                    }
                    _ => {
                        scan.error_files += 1;
                        push_log(
                            &mut scan,
                            "warning",
                            format!("Could not verify: {relative} — {}", result.message),
                        );
                    }
                }
            }
        }
        Err(err) => {
            let mut scan = state.scan.lock().unwrap();
            scan.processed_files += 1;
            scan.processed_bytes += size;
            scan.read_bytes += size;
            scan.error_files += 1;
            push_log(&mut scan, "warning", format!("{relative}: {err}"));
        }
    }
    publish_status(state);
}

fn discover_files(root: &Path) -> Result<Vec<(PathBuf, u64)>, String> {
    if !root.is_dir() {
        return Err(format!("library root is not readable: {}", root.display()));
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|err| err.to_string())?;
        if entry.file_type().is_file() && supported(entry.path()) {
            let size = entry.metadata().map_err(|err| err.to_string())?.len();
            files.push((entry.into_path(), size));
        }
    }
    files.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn verify_one(
    db_path: &Path,
    path: &Path,
    source: &str,
    force: bool,
) -> Result<(Validation, bool), String> {
    let connection = open_db(db_path).map_err(|err| err.to_string())?;
    verify_one_with_connection(&connection, path, source, force, None)
}

fn verify_one_with_connection(
    connection: &Connection,
    path: &Path,
    source: &str,
    force: bool,
    disk_reader: Option<&Mutex<()>>,
) -> Result<(Validation, bool), String> {
    let metadata = path.metadata().map_err(|err| err.to_string())?;
    if !metadata.is_file() {
        return Err("path is not a regular file".into());
    }
    let size = metadata.len();
    let mtime_ns = metadata
        .modified()
        .map_err(|err| err.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| err.to_string())?
        .as_nanos()
        .min(i64::MAX as u128) as i64;
    let path_text = path.to_string_lossy();
    if !force {
        let cached = connection
            .query_row(
                "SELECT verdict, validator, message, duration_ms, authenticity, authenticity_message FROM file_results
                 WHERE path = ?1 AND size = ?2 AND mtime_ns = ?3 AND validator_version = ?4",
                params![path_text.as_ref(), size as i64, mtime_ns, VALIDATOR_VERSION],
                |row| {
                    Ok(Validation {
                        verdict: row.get(0)?,
                        validator: row.get(1)?,
                        message: row.get(2)?,
                        duration_ms: row.get::<_, i64>(3)? as u64,
                        authenticity: row.get(4)?,
                        authenticity_message: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|err| err.to_string())?;
        if let Some(cached) = cached {
            return Ok((cached, true));
        }
    }
    let format = extension(path).unwrap_or_else(|| "unknown".into());
    let result = validate(path, disk_reader);
    connection
        .execute(
            "INSERT INTO file_results
             (path, size, mtime_ns, validator_version, validator, format, verdict, authenticity,
              authenticity_message, checked_at, duration_ms, message, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(path) DO UPDATE SET
               size=excluded.size, mtime_ns=excluded.mtime_ns, validator_version=excluded.validator_version,
               validator=excluded.validator, format=excluded.format, verdict=excluded.verdict,
               authenticity=excluded.authenticity, authenticity_message=excluded.authenticity_message,
               checked_at=excluded.checked_at, duration_ms=excluded.duration_ms,
               message=excluded.message, source=excluded.source",
            params![
                path_text.as_ref(), size as i64, mtime_ns, VALIDATOR_VERSION, result.validator,
                format, result.verdict, result.authenticity, result.authenticity_message,
                Utc::now().to_rfc3339(), result.duration_ms as i64, result.message, source
            ],
        )
        .map_err(|err| err.to_string())?;
    Ok((result, false))
}

fn validate(path: &Path, disk_reader: Option<&Mutex<()>>) -> Validation {
    let disk_guard =
        disk_reader.map(|lock| lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner()));
    let start = Instant::now();
    let (validator, output) = if extension(path).as_deref() == Some("flac") {
        (
            "flac --test",
            Command::new("flac")
                .args(["--test", "--silent", "--"])
                .arg(path)
                .output(),
        )
    } else {
        (
            "ffmpeg decode",
            Command::new("ffmpeg")
                .args(["-hide_banner", "-nostdin", "-v", "error", "-i"])
                .arg(path)
                .args(["-map", "0:a:0", "-f", "null", "-"])
                .output(),
        )
    };
    drop(disk_guard);
    let duration_ms = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
    let mut result = match output {
        Err(err) => Validation {
            verdict: "error".into(),
            validator: validator.into(),
            authenticity: "unknown".into(),
            authenticity_message: "Integrity validation did not complete".into(),
            message: format!("validator unavailable: {err}"),
            duration_ms,
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let failed =
                !output.status.success() || (validator == "ffmpeg decode" && !stderr.is_empty());
            Validation {
                verdict: if failed {
                    "corrupt".into()
                } else {
                    "healthy".into()
                },
                validator: validator.into(),
                authenticity: if validator == "flac --test" {
                    "pending".into()
                } else {
                    "not_applicable".into()
                },
                authenticity_message: if validator == "flac --test" {
                    "Authenticity analysis has not run".into()
                } else {
                    "Authenticity analysis applies to lossless containers".into()
                },
                message: if failed {
                    shorten(
                        if stderr.is_empty() {
                            "decoder rejected file"
                        } else {
                            &stderr
                        },
                        4000,
                    )
                } else {
                    "Strict decode completed without errors".into()
                },
                duration_ms,
            }
        }
    };
    if result.verdict == "healthy" && extension(path).as_deref() == Some("flac") {
        match Command::new("isflac").arg(path).output() {
            Ok(output) if output.status.code() == Some(0) => {
                result.authenticity = "likely_genuine".into();
                result.authenticity_message =
                    shorten(String::from_utf8_lossy(&output.stdout).trim(), 1000);
            }
            Ok(output) if output.status.code() == Some(2) => {
                result.authenticity = "likely_lossy".into();
                result.authenticity_message =
                    shorten(String::from_utf8_lossy(&output.stdout).trim(), 1000);
            }
            Ok(output) => {
                result.authenticity = "unknown".into();
                result.authenticity_message =
                    shorten(String::from_utf8_lossy(&output.stderr).trim(), 1000);
            }
            Err(err) => {
                result.authenticity = "unknown".into();
                result.authenticity_message = format!("authenticity analyzer unavailable: {err}");
            }
        }
    }
    result
}

fn validator_for(path: &Path) -> &'static str {
    if extension(path).as_deref() == Some("flac") {
        "flac --test"
    } else {
        "ffmpeg decode"
    }
}

fn supported(path: &Path) -> bool {
    matches!(
        extension(path).as_deref(),
        Some("flac" | "mp3" | "wav" | "m4a" | "ape" | "opus")
    )
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|value| value.to_ascii_lowercase())
}

fn allowed_import_path(input: &str, roots: &[PathBuf]) -> Result<PathBuf, String> {
    let requested = Path::new(input);
    if !requested.is_absolute() {
        return Err("import path must be absolute".into());
    }
    let canonical = requested
        .canonicalize()
        .map_err(|err| format!("cannot resolve import path: {err}"))?;
    if !canonical.is_file() {
        return Err("import path is not a regular file".into());
    }
    for root in roots {
        if let Ok(root) = root.canonicalize()
            && canonical.starts_with(root)
        {
            return Ok(canonical);
        }
    }
    Err("import path is outside approved staging roots".into())
}

fn finish_scan(state: &AppState, connection: &Connection, status: &str) {
    let mut scan = state.scan.lock().unwrap();
    scan.phase = status.into();
    scan.finished_at = Some(Utc::now());
    scan.current_path = None;
    scan.current_validator = None;
    push_log(&mut scan, "info", format!("Scan {status}"));
    if let Some(run_id) = scan.run_id
        && let Err(err) = connection.execute(
            "UPDATE scan_runs SET status=?1, finished_at=?2, total_files=?3, verified_files=?4,
             skipped_files=?5, corrupt_files=?6, error_files=?7, total_bytes=?8 WHERE id=?9",
            params![
                status,
                Utc::now().to_rfc3339(),
                scan.total_files as i64,
                scan.verified_files as i64,
                scan.skipped_files as i64,
                scan.corrupt_files as i64,
                scan.error_files as i64,
                scan.total_bytes as i64,
                run_id
            ],
        )
    {
        error!(%err, "could not finalize scan run");
    }
    drop(scan);
    publish_status(state);
}

fn fail_scan(state: &AppState, message: String) {
    let mut scan = state.scan.lock().unwrap();
    scan.phase = "failed".into();
    scan.finished_at = Some(Utc::now());
    scan.message = Some(message.clone());
    push_log(&mut scan, "error", message);
    drop(scan);
    publish_status(state);
}

fn publish_status(state: &AppState) {
    let _ = state.events.send(RealtimeEvent {
        kind: "status",
        status: state.scan.lock().unwrap().clone(),
    });
}

fn push_log(scan: &mut ScanStatus, level: &str, text: String) {
    if scan.log.len() >= 80 {
        scan.log.pop_front();
    }
    scan.log.push_back(LogLine {
        at: Utc::now(),
        level: level.into(),
        text,
    });
}

fn init_db(path: &Path) -> rusqlite::Result<()> {
    let connection = open_db(path)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS file_results (
            path TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            mtime_ns INTEGER NOT NULL,
            validator_version TEXT NOT NULL,
            validator TEXT NOT NULL,
            format TEXT NOT NULL,
            verdict TEXT NOT NULL,
            authenticity TEXT NOT NULL DEFAULT 'unknown',
            authenticity_message TEXT NOT NULL DEFAULT '',
            checked_at TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            message TEXT NOT NULL,
            source TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS file_results_verdict_idx ON file_results(verdict, checked_at DESC);
         CREATE TABLE IF NOT EXISTS scan_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mode TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            total_files INTEGER NOT NULL DEFAULT 0,
            verified_files INTEGER NOT NULL DEFAULT 0,
            skipped_files INTEGER NOT NULL DEFAULT 0,
            corrupt_files INTEGER NOT NULL DEFAULT 0,
            error_files INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0
         );",
    )?;
    connection.execute(
        "UPDATE scan_runs SET status='cancelled', finished_at=?1 WHERE status='running'",
        [Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(30))?;
    Ok(connection)
}

fn load_summary(path: &Path) -> Result<Summary, String> {
    let connection = Connection::open(path).map_err(|err| err.to_string())?;
    let (known, healthy, corrupt, suspects, errors, bytes): (i64, i64, i64, i64, i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(verdict='healthy'),0), COALESCE(SUM(verdict='corrupt'),0),
                    COALESCE(SUM(authenticity='likely_lossy'),0), COALESCE(SUM(verdict='error'),0),
                    COALESCE(SUM(size),0) FROM file_results",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        )
        .map_err(|err| err.to_string())?;
    let last = connection
        .query_row(
            "SELECT finished_at FROM scan_runs WHERE status='completed' ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| err.to_string())?;
    Ok(Summary {
        known_files: known as u64,
        healthy_files: healthy as u64,
        corrupt_files: corrupt as u64,
        suspect_files: suspects as u64,
        error_files: errors as u64,
        checked_bytes: bytes as u64,
        last_completed_at: last,
        validator_version: VALIDATOR_VERSION,
    })
}

fn load_results(path: &Path, query: &ResultQuery) -> Result<ResultsPage, String> {
    let connection = Connection::open(path).map_err(|err| err.to_string())?;
    let verdict = query.verdict.as_deref().unwrap_or("all");
    let needle = format!("%{}%", query.query.as_deref().unwrap_or_default());
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0);
    let total = connection
        .query_row(
            "SELECT COUNT(*) FROM file_results
             WHERE (?1='all' OR verdict=?1 OR authenticity=?1) AND path LIKE ?2 ESCAPE '\\'",
            params![verdict, needle],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|err| err.to_string())? as u64;
    let mut statement = connection
        .prepare(
            "SELECT path, size, format, verdict, authenticity, checked_at, duration_ms, message, source, mtime_ns, validator_version, authenticity_message FROM file_results
             WHERE (?1='all' OR verdict=?1 OR authenticity=?1) AND path LIKE ?2 ESCAPE '\\'
             ORDER BY CASE verdict WHEN 'corrupt' THEN 0 WHEN 'error' THEN 1 ELSE 2 END, checked_at DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|err| err.to_string())?;
    let rows = statement
        .query_map(params![verdict, needle, limit, offset], |row| {
            Ok(FileResult {
                path: row.get(0)?,
                size: row.get::<_, i64>(1)? as u64,
                format: row.get(2)?,
                verdict: row.get(3)?,
                authenticity: row.get(4)?,
                checked_at: row.get(5)?,
                duration_ms: row.get::<_, i64>(6)? as u64,
                message: row.get(7)?,
                source: row.get(8)?,
                mtime_ns: row.get::<_, i64>(9)?.to_string(),
                validator_version: row.get(10)?,
                authenticity_message: row.get(11)?,
            })
        })
        .map_err(|err| err.to_string())?;
    let items = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())?;
    Ok(ResultsPage {
        items,
        total,
        limit,
        offset,
    })
}

fn load_history(path: &Path) -> Result<Vec<ScanRun>, String> {
    let connection = Connection::open(path).map_err(|err| err.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT id, mode, status, started_at, finished_at, total_files, verified_files, skipped_files, corrupt_files
             FROM scan_runs ORDER BY id DESC LIMIT 30",
        )
        .map_err(|err| err.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(ScanRun {
                id: row.get(0)?,
                mode: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                finished_at: row.get(4)?,
                total_files: row.get::<_, i64>(5)? as u64,
                verified_files: row.get::<_, i64>(6)? as u64,
                skipped_files: row.get::<_, i64>(7)? as u64,
                corrupt_files: row.get::<_, i64>(8)? as u64,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| err.to_string())
}

fn authorized(headers: &HeaderMap, state: &AppState) -> bool {
    if headers
        .get("x-integrity-token")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            state
                .api_token
                .as_ref()
                .is_some_and(|token| secure_eq(value, token))
        })
    {
        return true;
    }
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|item| {
                let (name, value) = item.trim().split_once('=')?;
                (name == COOKIE_NAME).then_some(value)
            })
        })
        .is_some_and(|value| secure_eq(value, &state.session_token))
}

fn secure_eq(left: &str, right: &str) -> bool {
    left.len() == right.len() && constant_time_eq(left.as_bytes(), right.as_bytes())
}

fn internal_error(error: impl std::fmt::Display) -> axum::response::Response {
    error!(%error, "request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

fn shorten(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.into()
    } else {
        format!("{}…", value.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn supported_extensions_are_case_insensitive() {
        assert!(supported(Path::new("track.FLAC")));
        assert!(supported(Path::new("track.m4a")));
        assert!(!supported(Path::new("cover.jpg")));
    }

    #[test]
    fn shortens_long_messages() {
        assert_eq!(shorten("abcdef", 4), "abcd…");
        assert_eq!(shorten("abc", 4), "abc");
    }

    #[test]
    fn results_include_filtered_total_and_requested_page() {
        let database = std::env::temp_dir().join(format!(
            "audio-integrity-pagination-{}-{}.db",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        init_db(&database).expect("database should initialize");
        let connection = open_db(&database).expect("database should open");
        for index in 0..3 {
            connection
                .execute(
                    "INSERT INTO file_results
                     (path, size, mtime_ns, validator_version, validator, format, verdict,
                      authenticity, checked_at, duration_ms, message, source)
                     VALUES (?1, 1, 1, 'test', 'flac', 'flac', 'corrupt', 'unknown',
                             '2026-01-01T00:00:00Z', 1, 'test', 'library')",
                    [format!("/music/{index}.flac")],
                )
                .expect("row should insert");
        }
        drop(connection);

        let page = load_results(
            &database,
            &ResultQuery {
                verdict: Some("corrupt".into()),
                query: None,
                limit: Some(1),
                offset: Some(1),
            },
        )
        .expect("page should load");

        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.limit, 1);
        assert_eq!(page.offset, 1);
        let serialized = serde_json::to_value(&page.items[0]).unwrap();
        assert_eq!(serialized["mtimeNs"], "1");
        assert_eq!(serialized["validatorVersion"], "test");
        assert!(serialized["authenticityMessage"].is_string());
        fs::remove_file(database).expect("database should be removable");
    }
}
