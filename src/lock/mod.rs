pub mod installer;

use anyhow::{Context as _, Error, Result};
use futures::{stream, StreamExt, TryStreamExt};
use installer::install_package;
use serde::{Deserialize, Serialize};
use tokio::fs;
use url::Url;

use crate::{
    config::{Config, Package, Source},
    context::Context,
    provider::{github::Github, Provider},
    util::fs_ext::load_toml,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct LockedConfig {
    #[serde(flatten)]
    ctx: Context,

    #[serde(default)]
    pub pkgs: Vec<LockedPackage>,

    /// Any errors that occurred while generating this `LockedConfig`.
    #[serde(skip)]
    pub errors: Vec<Error>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "lowercase")]
pub struct LockedPackage {
    pub name:         String,
    pub version:      String,
    #[serde(flatten)]
    pub source:       Source,
    pub desc:         Option<String>,
    pub filename:     String,
    pub download_url: Option<Url>,
}

// Install a package.
pub async fn sync_package(ctx: &Context, pkg: &Package) -> Result<LockedPackage> {
    let provider = Github::new()?;
    let lpkg = provider.download(ctx, pkg).await?;

    install_package(ctx, &lpkg).await?;
    ctx.log_status("Checked", &format!("{}@{}", pkg.name, lpkg.version));

    Ok(lpkg)
}

pub async fn restore_package(ctx: &Context, lpkg: LockedPackage) -> Result<()> {
    let provider = Github::new()?;

    provider.download_locked(ctx, &lpkg).await?;

    install_package(ctx, &lpkg).await?;
    ctx.log_status("Checked", &format!("{}@{}", lpkg.name, lpkg.version));

    Ok(())
}

/// Install all necessary packages, and returns a [`LockedConfig`].
pub async fn sync_packages(ctx: &Context, config: Config) -> Result<LockedConfig> {
    let mut locked = Vec::new();
    for pkg in config.pkgs {
        let lpkg = sync_package(ctx, &pkg).await?;
        locked.push(lpkg);
    }

    Ok(LockedConfig { ctx: ctx.clone(), pkgs: locked, errors: Vec::new() })
}

/// Restore packages according to the given [`LockedConfig`].
pub async fn restore_packages(lcfg: LockedConfig) -> Result<()> {
    let _: Vec<()> = stream::iter(lcfg.pkgs.into_iter())
        .then(|pkg| restore_package(&lcfg.ctx, pkg))
        .try_collect()
        .await?;

    Ok(())
}

impl LockedConfig {
    pub async fn load(ctx: &Context) -> Result<Self> {
        let mut lcfg: LockedConfig = load_toml(&ctx.lock_file)
            .await
            .with_context(|| format!("failed to load {}", ctx.lock_file.display()))?;
        lcfg.ctx = ctx.clone();
        Ok(lcfg)
    }

    /// Write this `LockedConfig` to the given path.
    pub async fn save(&self) -> Result<()> {
        let buf = toml::to_string_pretty(self).context("failed to serialize `LockedConfig`")?;
        fs::write(&self.ctx.lock_file, buf)
            .await
            .with_context(|| format!("failed to save {}", self.ctx.lock_file.display()))
    }

    /// Add a package to the configuration.
    pub fn add_pkg(&mut self, lpkg: LockedPackage) {
        self.pkgs.push(lpkg);
    }
}
