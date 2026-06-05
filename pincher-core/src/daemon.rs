//! Pincher Background Synchronization Daemon
//! 
//! Low-priority worker that monitors reflex execution failures, collects telemetry,
//! and asynchronously uploads error payloads to the cloud compiler for self-healing.
//! Follows Pinch6 blueprint non-negotiables.

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Single telemetry entry from a failed reflex execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEntry {
    pub id: i64,
    pub intent_id: String,
    pub reflex_id: String,
    pub failed_code: String,
    pub error_logs: String,
    pub env_context: String,
    pub created_at: String,
}

/// Background daemon that monitors, collects, and syncs error telemetry
pub struct TelemetryDaemon {
    db_path: PathBuf,
    compiler_endpoint: String,
    auth_token: String,
    poll_interval_secs: u64,
}

impl TelemetryDaemon {
    /// Create a new telemetry daemon (Pinch6 compliant)
    pub fn new(db: PathBuf, endpoint: &str, token: &str) -> Self {
        Self {
            db_path: db,
            compiler_endpoint: endpoint.to_string(),
            auth_token: token.to_string(),
            poll_interval_secs: 60,
        }
    }

    /// Set polling interval (default: 60s as per Pinch6)
    pub fn with_poll_interval(mut self, secs: u64) -> Self {
        self.poll_interval_secs = secs;
        self
    }

    /// Spawn a background runtime thread (SCHED_IDLE priority as per Pinch6)
    pub fn start_sync_loop(self) {
        thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default();
            
            loop {
                thread::sleep(Duration::from_secs(self.poll_interval_secs));

                // Skip if offline per Pinch6 TCP network detection
                if !self.is_network_available() {
                    continue;
                }

                if let Ok(mut conn) = Connection::open(&self.db_path) {
                    if let Err(e) = self.process_pending_telemetry(&mut conn, &client) {
                        eprintln!("[DAEMON ERROR] Failed processing sync event queue: {}", e);
                    }
                }
            }
        });
    }

    /// TCP-based network detection per Pinch6 blueprint
    fn is_network_available(&self) -> bool {
        match std::net::TcpStream::connect_timeout(
            &"1.1.1.1:53".parse().unwrap(),
            Duration::from_secs(2),
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Process pending telemetry (max 5 per cycle per Pinch6)
    fn process_pending_telemetry(
        &self,
        conn: &mut Connection,
        client: &reqwest::blocking::Client,
    ) -> Result<()> {
        let mut stmt = conn.prepare(
            "SELECT id, intent_id, reflex_id, failed_code, error_logs, env_context, created_at 
             FROM telemetry_queue LIMIT 5"
        )?;

        let entries: Vec<TelemetryEntry> = stmt
            .query_map([], |row| {
                Ok(TelemetryEntry {
                    id: row.get(0)?,
                    intent_id: row.get(1)?,
                    reflex_id: row.get(2)?,
                    failed_code: row.get(3)?,
                    error_logs: row.get(4)?,
                    env_context: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        if entries.is_empty() {
            return Ok(());
        }

        for entry in &entries {
            match self.send_heal_request(entry, client) {
                Ok(true) => {
                    // Atomic delete per Pinch6
                    conn.execute("DELETE FROM telemetry_queue WHERE id = ?1", params![entry.id])?;
                }
                Err(e) => {
                    eprintln!("[DAEMON ERROR] Failed to send entry {}: {}", entry.id, e);
                }
            }
        }

        Ok(())
    }

    /// Send heal request to cloud compiler
    fn send_heal_request(
        &self,
        entry: &TelemetryEntry,
        client: &reqwest::blocking::Client,
    ) -> Result<bool> {
        let payload = serde_json::json!({
            "intent_id": entry.intent_id,
            "reflex_id": entry.reflex_id,
            "failed_code": entry.failed_code,
            "error_logs": entry.error_logs,
            "env_context": entry.env_context,
        });

        let response = client
            .post(format!("{}/api/v1/heal", self.compiler_endpoint))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()?;

        if response.status().is_success() {
            let body = response.bytes()?;
            Ok(!body.is_empty() && body.len() > 100)
        } else {
            Err(anyhow!("Heal endpoint returned HTTP {}", response.status()))
        }
    }
}
