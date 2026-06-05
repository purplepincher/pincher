//! Pincher Central Community Registry Client
//! 
//! Stateless client for publishing, querying, and fetching .nail bundles
//! from the Pincher package registry. Follows Pinch6 blueprint non-negotiables.

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

/// Package metadata from the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub package_id: String,
    pub name: String,
    pub latest_version: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub download_url: String,
    pub signature: String,
    pub checksum: String,
}

/// Release entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEntry {
    pub version_semver: String,
    pub compiled_at: String,
    pub checksum: String,
}

/// Pincher Registry client for publishing and fetching packages
pub struct PincherRegistryClient {
    registry_url: String,
    auth_token: String,
    client: Client,
}

impl PincherRegistryClient {
    /// Create a new registry client
    pub fn new(url: &str, token: &str) -> Self {
        Self {
            registry_url: url.to_string(),
            auth_token: token.to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Register a new developer account with the registry (Pinch6 compliant)
    pub fn register_developer(&self, username: &str, public_key: &str) -> Result<String> {
        let payload = serde_json::json!({
            "username": username,
            "public_key": public_key,
        });

        let response = self.client
            .post(format!("{}/api/v1/developers/register", self.registry_url))
            .json(&payload)
            .send()
            .context("Failed to send developer registration request")?;

        if !response.status().is_success() {
            return Err(anyhow!("Registration failed: HTTP {}", response.status()));
        }

        let result: serde_json::Value = response.json()?;
        let token = result["api_token"]
            .as_str()
            .ok_or_else(|| anyhow!("No API token in registration response"))?
            .to_string();

        Ok(token)
    }

    /// Register a new package in the registry (Pinch6 compliant)
    pub fn register_package(
        &self,
        name: &str,
        description: &str,
        repository_url: &str,
    ) -> Result<String> {
        let payload = serde_json::json!({
            "name": name,
            "description": description,
            "repository_url": repository_url,
        });

        let response = self.client
            .post(format!("{}/api/v1/packages", self.registry_url))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .context("Failed to register package")?;

        if !response.status().is_success() {
            return Err(anyhow!("Package registration failed: HTTP {} - {}", 
                response.status(), response.text()?));
        }

        let result: serde_json::Value = response.json()?;
        let package_id = result["package_id"]
            .as_str()
            .ok_or_else(|| anyhow!("No package_id in response"))?
            .to_string();

        Ok(package_id)
    }

    /// Publish a .nail bundle to the registry (Pinch6 compliant: multipart + signature)
    pub fn publish_package(&self, nail_path: &Path) -> Result<()> {
        let sig_path = nail_path.with_extension("nail.sig");
        if !sig_path.exists() {
            return Err(anyhow!(
                "Cannot publish unsigned packages. Run 'pincher pack' first to generate {}.nail.sig",
                nail_path.file_name().unwrap().to_string_lossy()
            ));
        }

        // Read the .nail binary
        let mut nail_file = File::open(nail_path)
            .context("Failed to open .nail bundle")?;
        let mut package_bytes = Vec::new();
        nail_file.read_to_end(&mut package_bytes)?;

        // Read the signature
        let mut sig_file = File::open(&sig_path)
            .context("Failed to open .nail.sig signature file")?;
        let mut sig_string = String::new();
        sig_file.read_to_string(&mut sig_string)?;

        let package_name = nail_path.file_name().unwrap().to_string_lossy();

        // Build multipart form as required by Pinch6
        let form = reqwest::blocking::multipart::Form::new()
            .text("signature", sig_string.trim().to_string())
            .part(
                "bundle",
                reqwest::blocking::multipart::Part::bytes(package_bytes)
                    .file_name(package_name.to_string()),
            );

        let response = self.client
            .post(format!("{}/api/v1/publish", self.registry_url))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .multipart(form)
            .send()
            .context("Failed to send publish request")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let body = response.text()?;
            Err(anyhow!("Registry rejected publication: HTTP {} - {}", 
                response.status(), body))
        }
    }

    /// Query package metadata from the registry
    pub fn query_package(&self, package_name: &str) -> Result<PackageMetadata> {
        let response = self.client
            .get(format!("{}/api/v1/packages/{}", self.registry_url, package_name))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .context("Failed to fetch package metadata")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch package '{}': HTTP {}", 
                package_name, response.status()));
        }

        let metadata: PackageMetadata = response.json()?;
        Ok(metadata)
    }

    /// List all releases of a package
    pub fn list_releases(&self, package_name: &str) -> Result<Vec<ReleaseEntry>> {
        let response = self.client
            .get(format!("{}/api/v1/packages/{}/releases", self.registry_url, package_name))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .context("Failed to list releases")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list releases for '{}': HTTP {}", 
                package_name, response.status()));
        }

        let releases: Vec<ReleaseEntry> = response.json()?;
        Ok(releases)
    }

    /// Download a specific version of a package
    pub fn download_package(&self, package_name: &str, version: &str, output_path: &Path) -> Result<PathBuf> {
        let response = self.client
            .get(format!("{}/api/v1/packages/{}/versions/{}", 
                self.registry_url, package_name, version))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .context("Failed to download package")?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to download package: HTTP {}", response.status()));
        }

        let bytes = response.bytes()?;
        let output_file = output_path.join(format!("{}.nail", package_name));
        std::fs::write(&output_file, &bytes)
            .context("Failed to write downloaded package")?;

        Ok(output_file)
    }
}
