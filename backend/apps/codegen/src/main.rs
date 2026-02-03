use anyhow::{Context, Result};
use crudder_lua::{RecipeLoader, RecipeRunner};
use std::path::{Path, PathBuf};

fn repo_root() -> Result<PathBuf> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in here.ancestors() {
        if dir.join("crudder").exists() {
            return Ok(dir.to_path_buf());
        }
    }
    anyhow::bail!("failed to locate repo root (expected crudder/ directory)")
}

fn write_files(root: &Path, files: Vec<crudder_lua::EmittedFile>) -> Result<()> {
    for file in files {
        let path = root.join(&file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create output directory {}", parent.display())
            })?;
        }
        std::fs::write(&path, file.content)
            .with_context(|| format!("failed to write {}", path.display()))?;
        println!("  wrote {}", file.path);
    }
    Ok(())
}

fn main() -> Result<()> {
    let root = repo_root()?;

    // Schema path can be overridden via env var or CLI arg
    let schema_name = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("CRUDDER_SCHEMA").ok())
        .unwrap_or_else(|| "switchboard.crudder".to_string());

    let schema_path = root.join("crudder").join(&schema_name);
    let schema_dir = schema_path
        .parent()
        .context("schema path should have a parent directory")?
        .to_path_buf();

    println!("Parsing schema: {}", schema_path.display());
    let source = std::fs::read_to_string(&schema_path)
        .with_context(|| format!("failed to read {}", schema_path.display()))?;

    let schema = crudder_parser::parse(&source).map_err(|e| {
        anyhow::anyhow!(
            "failed to parse {}: {:?}",
            schema_path.display(),
            e
        )
    })?;

    let loader = RecipeLoader::new().with_project_dir(schema_dir);

    // -------------------------------------------------------------------------
    // Backend recipe (Axum REST)
    // -------------------------------------------------------------------------
    println!("\nRunning backend recipe (axum-rest)...");
    {
        let recipe = loader
            .load("axum-rest")
            .map_err(|e| anyhow::anyhow!("failed to load recipe axum-rest: {}", e))?;

        let mut runner = RecipeRunner::new();

        // Switchboard-specific configuration
        runner.set_option("output_dir", "backend/crates/backend-api/src/generated/rest");
        runner.set_option("models_subdir", "models");
        runner.set_option("api_prefix", "/api/v1");
        runner.set_option("impl_module", "crate::rest::impls");
        runner.set_option("error_module", "crate::error");
        runner.set_option("state_type", "crate::state::GatewayState");
        runner.set_option("models_import", "crate::rest::models::*");
        runner.set_option("auth_type", "i64");
        runner.set_option("auth_extractor", "Extension");
        runner.set_option("auth_binding", "user_id");

        let files = runner
            .run(&schema, &recipe)
            .map_err(|e| anyhow::anyhow!("failed to run recipe axum-rest: {}", e))?;

        write_files(&root, files)?;
    }

    // -------------------------------------------------------------------------
    // Frontend recipe (TypeScript client)
    // -------------------------------------------------------------------------
    println!("\nRunning frontend recipe (typescript-client)...");
    {
        let recipe = loader
            .load("typescript-client")
            .map_err(|e| anyhow::anyhow!("failed to load recipe typescript-client: {}", e))?;

        let mut runner = RecipeRunner::new();

        // Switchboard-specific configuration
        runner.set_option("output_dir", "apps/web/src/generated");
        runner.set_option("api_prefix", "/api/v1");
        runner.set_option("auth_header", "Bearer ${token}");
        runner.set_option("content_type", "application/json");

        let files = runner
            .run(&schema, &recipe)
            .map_err(|e| anyhow::anyhow!("failed to run recipe typescript-client: {}", e))?;

        write_files(&root, files)?;
    }

    println!("\nCode generation complete!");
    Ok(())
}
