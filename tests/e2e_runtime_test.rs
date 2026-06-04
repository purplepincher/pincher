//! End-to-End Integration Test Harness for Pincher Runtime
//! 
//! Validates the full reflex execution pipeline from text intent → sandboxed execution

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Import core Pincher runtime components
    use pincher_core::{
        ReflexEngine,
        Embedder,
        BundleSecurityEngine,
        migration::{fingerprint, pack_nail, unpack_nail},
    };

    #[test]
    fn test_end_to_end_reflex_execution_pipeline() -> Result<()> {
        // Setup isolated temporary test environment
        let tmp_dir = TempDir::new()?;
        let workspace_path = tmp_dir.path();
        let test_notes_dir = workspace_path.join("Documents/Notes");
        fs::create_dir_all(&test_notes_dir)?;

        // Create starting test file
        let mut sample_note = File::create(test_notes_dir.join("daily_reflection.md"))?;
        writeln!(sample_note, "# Core Concept\nDeveloping an AI engine paradigm shift.")?;

        // Initialize SQLite reflex database
        let db_path = workspace_path.join("reflexes.db");
        let conn = pincher_core::db::schema::init_db(&db_path)?;

        // Initialize embedder with fallback hash model
        let embedder = pincher_core::embed::Embedder::new(None)?;

        // Create test reflex: "create git repo for notes directory"
        let intent = "create git repo for my notes";
        let action_template = "cd {{dir}} && git init && git commit -m \"Initial commit\"";
        let params = vec![("dir".to_string(), test_notes_dir.to_string_lossy().to_string())];

        // Teach the reflex to the engine
        let mut engine = ReflexEngine::new(conn, embedder);
        let reflex = engine.teach(intent, action_template)?;
        println!("✅ Taught reflex: {}", reflex.intent);

        // Simulate user intent
        let execution_result = engine.do_command("initialize a git repository for my notes")?;
        println!("📋 Execution result: {} (latency: {}ms)", execution_result.output, execution_result.latency_ms);

        // Verify post conditions: git repo was created
        let git_dir = test_notes_dir.join(".git");
        assert!(git_dir.exists(), ".git directory was not created in sandboxed execution");
        assert!(git_dir.join("HEAD").exists(), "Git HEAD file missing");
        assert!(git_dir.join("config").exists(), "Git config file missing");

        println!("✅ E2E pipeline validation successful!");
        Ok(())
    }

    #[test]
    fn test_nail_migration_portability() -> Result<()> {
        // Test full agent rig portability between shells
        let tmp_source = TempDir::new()?;
        let tmp_target = TempDir::new()?;

        // Create source agent rig
        let source_db = tmp_source.path().join("reflexes.db");
        let conn = pincher_core::db::schema::init_db(&source_db)?;
        let embedder = pincher_core::embed::Embedder::new(None)?;
        let mut engine = ReflexEngine::new(conn, embedder);

        // Teach test reflexes
        engine.teach("list files", "ls -la")?;
        engine.teach("show disk usage", "df -h")?;

        // Pack into nail bundle
        let nail_path = tmp_source.path().join("agent.nail");
        pack_nail(tmp_source.path(), &nail_path)?;
        println!("✅ Packed agent nail bundle: {}", nail_path.display());

        // Unpack to target shell
        unpack_nail(&nail_path, tmp_target.path())?;
        let target_db = tmp_target.path().join("reflexes.db");
        assert!(target_db.exists(), "Reflex DB was not unpacked correctly");

        // Verify reflexes exist on target
        let target_conn = rusqlite::Connection::open(&target_db)?;
        let count: i64 = target_conn.query_row("SELECT COUNT(*) FROM reflexes", [], |row| row.get(0))?;
        assert_eq!(count, 2, "Expected 2 reflexes after migration");

        println!("✅ Nail migration portability validation successful!");
        Ok(())
    }

    #[test]
    fn test_sandbox_separation() -> Result<()> {
        // Verify sandbox prevents access to host filesystem
        let tmp_dir = TempDir::new()?;
        let forbidden_path = "/etc/passwd";

        let db_path = tmp_dir.path().join("reflexes.db");
        let conn = pincher_core::db::schema::init_db(&db_path)?;
        let embedder = pincher_core::embed::Embedder::new(None)?;
        let mut engine = ReflexEngine::new(conn, embedder);

        // Teach a dangerous reflex that tries to access host files
        let dangerous_intent = "read the system password file";
        let dangerous_action = "cat /etc/passwd";
        let reflex = engine.teach(dangerous_intent, dangerous_action)?;

        // Execute should fail due to sandbox veto
        let execution_result = engine.execute(&reflex, "");
        assert!(execution_result.is_err(), "Dangerous action should have been vetoed");
        assert!(execution_result.unwrap_err().to_string().contains("denied"), "Error should indicate vetoed action");

        println!("✅ Sandbox security validation successful!");
        Ok(())
    }
}