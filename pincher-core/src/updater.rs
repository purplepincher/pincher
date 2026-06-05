//! Pincher Client Update Engine
//! 
//! Atomic, secure client-side bundle updater with signature verification
//! Follows Pinch6 blueprint non-negotiables.

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Metadata about a package from the Pincher registry
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub latest_version: String,
    pub download_url: String,
    pub signature: String,
    pub checksum: String,
}

/// Pincher Client Updater implementation
pub struct PincherUpdater {
    registry_url: String,
    local_bundle_dir: PathBuf,
    cache_staging_dir: PathBuf,
    client: Client,
}

impl PincherUpdater {
    /// Create a new updater instance
    pub fn new(registry_url: &str, local_bundle_dir: PathBuf) -> Result<Self> {
        let cache_staging_dir = local_bundle_dir.join(".cache_staging");
        fs::create_dir_all(&cache_staging_dir)?;

        Ok(Self {
            registry_url: registry_url.to_string(),
            local_bundle_dir,
            cache_staging_dir,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
        })
    }

    /// Check for updates
    pub fn check_for_updates(&self, package_name: &str) -> Result<Option<PackageMetadata>> {
        let metadata_url = format!("{}/api/v1/packages/{}", self.registry_url, package_name);
        let response = self.client.get(&metadata_url).send()?;

        if !response.status().is_success() {
            if response.status() == 404 {
                return Ok(None);
            }
            return Err(anyhow!("Failed to fetch package metadata: {} (status: {})", 
                response.text()?, response.status()));
        }

        let value: Value = response.json()?;
        let metadata = PackageMetadata {
            name: package_name.to_string(),
            latest_version: value["latest_version"]
                .as_str()
                .ok_or_else(|| anyhow!("Malformed version in registry response"))?
                .to_string(),
            download_url: value["download_url"]
                .as_str()
                .ok_or_else(|| anyhow!("Malformed download URL in registry response"))?
                .to_string(),
            signature: value["signature"]
                .as_str()
                .ok_or_else(|| anyhow!("Malformed signature in registry response"))?
                .to_string(),
            checksum: value["checksum"]
                .as_str()
                .ok_or_else(|| anyhow!("Malformed checksum in registry response"))?
                .to_string(),
        };

        Ok(Some(metadata))
    }

    /// Full update workflow with atomic swap per Pinch6
    pub fn update_package(&self, package_name: &str) -> Result<()> {
        let metadata = match self.check_for_updates(package_name)? {
            Some(m) => m,
            None => return Ok(()),
        };

        let staged_nail = self.download_package(&metadata)?;
        let staged_sig = self.save_signature(&metadata)?;

        self.verify_package(&staged_nail, &staged_sig)?;
        self.atomic_swap(&staged_nail, &staged_sig, package_name)?;

        Ok(())
    }

    /// Download package to staging
    fn download_package(&self, metadata: &PackageMetadata) -> Result<PathBuf> {
        let staged_path = self.cache_staging_dir.join(format!("{}.nail", metadata.name));
        let mut response = self.client.get(&metadata.download_url).send()?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download package: {} (status: {})", 
                response.text()?, response.status()));
        }

        let mut file = File::create(&staged_path)?;
        response.copy_to(&mut file)?;

        Ok(staged_path)
    }

    /// Save signature to staging
    fn save_signature(&self, metadata: &PackageMetadata) -> Result<PathBuf> {
        let sig_path = self.cache_staging_dir.join(format!("{}.nail.sig", metadata.name));
        let mut file = File::create(&sig_path)?;
        writeln!(file, "{}", metadata.signature)?;
        Ok(sig_path)
    }

    /// Verify package integrity
    fn verify_package(&self, bundle_path: &Path, sig_path: &Path) -> Result<()> {
        use crate::security::BundleSecurityEngine;

        let security_key = b"SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY";
        let security_mgr = BundleSecurityEngine::new(security_key)?;

        let test_dir = self.cache_staging_dir.join("_verify_test");
        fs::create_dir_all(&test_dir)?;

        security_mgr.verify_and_unpack(bundle_path, &test_dir)?;
        fs::remove_dir_all(&test_dir)?;

        Ok(())
    }

    /// Atomic swap per Pinch6 blueprint: backup → remove old → rename new
    fn atomic_swap(&self, new_bundle: &Path, new_sig: &Path, package_name: &str) -> Result<()> {
        let final_bundle = self.local_bundle_dir.join(format!("{}.nail", package_name));
        let final_sig = self.local_bundle_dir.join(format!("{}.nail.sig", package_name));

        // Create backups
        if final_bundle.exists() {
            let backup_bundle = final_bundle.with_extension(".nail.bak");
            let backup_sig = final_sig.with_extension(".nail.sig.bak");
            fs::rename(&final_bundle, &backup_bundle)?;
            fs::rename(&final_sig, &backup_sig)?;
        }

        // Atomic rename
        fs::rename(new_bundle, &final_bundle)?;
        fs::rename(new_sig, &final_sig)?;

        Ok(())
    }
}
