mod editable;
mod locked;

pub use editable::EditableConfig;
pub use locked::{LockedConfig, LockedPackage};

use std::{collections::BTreeMap, fmt, fs, str};

use anyhow::{Context as _, Result};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};

use crate::{
    context::Context,
    util::{load_toml, not_found_err, remove_file_if_exists},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
    pub pkgs: BTreeMap<String, Package>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Package {
    #[serde(skip)]
    pub name:    String,
    pub version: Option<String>,
    #[serde(flatten)]
    pub source:  Source,
    pub desc:    Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(tag = "source")]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Github { repo: String },
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Github { repo } => {
                write!(f, "github.com:{}", repo)
            }
        }
    }
}

impl Source {
    fn is_default(&self) -> bool {
        matches!(self, Self::Github { .. })
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name, version, source, desc: _ } = self;
        write!(
            f,
            "{name}@{version} from {source}",
            version = version.as_deref().unwrap_or("latest"),
        )
    }
}

impl<'de> Deserialize<'de> for Source {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SourceVisitor)
    }
}

struct SourceVisitor;

impl<'de> Visitor<'de> for SourceVisitor {
    type Value = Source;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("enum Source")
    }
    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Source,
            Repo,
        }
        const FIELDS: &[&str] = &["github"];

        let (mut source, mut repo) = (None, None);
        while let Some(key) = map.next_key()? {
            match key {
                Field::Source => match source {
                    None => source = Some(map.next_value()?),
                    Some(_) => return Err(Error::duplicate_field("source")),
                },
                Field::Repo => match repo {
                    None => repo = Some(map.next_value()?),
                    Some(_) => return Err(Error::duplicate_field("repo")),
                },
            }
        }

        let source = match source.unwrap_or("github".to_owned()).as_str() {
            "github" => Source::Github { repo: repo.ok_or_else(|| Error::missing_field("repo"))? },
            s => return Err(Error::unknown_variant(s, FIELDS)),
        };

        Ok(source)
    }
}

impl Config {
    /// Load the configuration from the given path.
    pub fn load(ctx: &Context) -> Result<Self> {
        let mut cfg = match load_toml(&ctx.config_file) {
            Err(e) if not_found_err(e.root_cause()) => Config::init(ctx)?,
            cfg => cfg.with_context(|| format!("failed to load {}", ctx.config_file.display()))?,
        };

        // Set the package names for convenience.
        for (name, pkg) in cfg.pkgs.iter_mut() {
            pkg.name = name.clone();
        }

        Ok(cfg)
    }

    fn init(ctx: &Context) -> Result<Self> {
        remove_file_if_exists(&ctx.lock_file)
            .with_context(|| format!("failed to remove lock file {}", ctx.lock_file.display()))?;

        let default = include_str!("packages.toml");
        fs::write(&ctx.config_file, default)
            .with_context(|| format!("failed to init {}", ctx.config_file.display()))?;
        Ok(toml::from_str(default)?)
    }
}

impl From<LockedPackage> for Package {
    fn from(val: LockedPackage) -> Self {
        Package {
            name:    val.name,
            version: val.version.into(),
            source:  val.source,
            desc:    val.desc,
        }
    }
}
