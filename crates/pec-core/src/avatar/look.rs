//! `workspace/avatar/`: which look the companion uses, its files, and the bundled characters they are copied from
//! (avatar R5 to R7).

use super::{Activity, Mood};
use crate::onboarding::{characters, field};
use crate::workspace::write_atomic;
use crate::{AppError, Core, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Expression {
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Motion {
    pub motion_group: String,
}

/// What `workspace/avatar/manifest.json` says to draw (avatar data 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
#[ts(export)]
pub enum AvatarLook {
    Live2d {
        character: String,
        model: String,
        #[serde(default)]
        moods: BTreeMap<Mood, Expression>,
        #[serde(default)]
        activities: BTreeMap<Activity, Motion>,
    },
    Static {
        character: String,
    },
    Picture {
        character: String,
        image: String,
    },
}

impl AvatarLook {
    fn character(&self) -> &str {
        match self {
            Self::Live2d { character, .. }
            | Self::Static { character }
            | Self::Picture { character, .. } => character,
        }
    }

    fn valid(&self) -> bool {
        match self {
            Self::Live2d { model, .. } => safe(model) && model.ends_with(".model3.json"),
            Self::Static { .. } => true,
            Self::Picture { image, .. } => {
                safe(image) && !image.contains('/') && extension(image, &PICTURES)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct AvatarManifest {
    pub look: AvatarLook,
    /// Why the manifest could not be used as written, when it could not (avatar FR-017, FR-031, FR-032).
    pub problem: Option<AvatarProblem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub enum AvatarProblem {
    Unreadable,
    MissingModel,
    CharacterGone,
}

#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum AvatarSource {
    Workspace,
    /// A bundled character's own files, for the wizard before any workspace exists.
    Bundled {
        character: String,
    },
}

#[derive(Serialize)]
struct ManifestFile<'a> {
    version: u32,
    #[serde(flatten)]
    look: &'a AvatarLook,
}

const MANIFEST: &str = "manifest.json";
const MODEL_FILES: [&str; 5] = ["json", "moc3", "png", "jpg", "webp"];
const PICTURES: [&str; 3] = ["png", "jpg", "webp"];
/// avatar FR-030.
const PICTURE_MAX_BYTES: usize = 5 * 1024 * 1024;

/// A relative path that stays inside its folder.
fn safe(path: &str) -> bool {
    let path = Path::new(path);
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn extension(path: &str, allowed: &[&str]) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| allowed.contains(&e.to_ascii_lowercase().as_str()))
}

impl Core {
    fn avatar_dir(&self) -> PathBuf {
        self.workspace_dir().join("avatar")
    }

    /// A bundled character's folder, if the app bundles one by that id.
    fn bundled(&self, character: &str) -> Option<PathBuf> {
        let dir = self.characters.as_ref()?.join(character);
        (safe(character) && characters().iter().any(|c| c.id == character)).then_some(dir)
    }

    /// The character's default look: its Live2D model when the app has one, else its still portrait.
    fn default_look(&self, character: &str) -> AvatarLook {
        let template = self.bundled(character).and_then(|dir| {
            let text = fs::read_to_string(dir.join("avatar.json")).ok()?;
            let mut value: Value = serde_json::from_str(&text).ok()?;
            value["character"] = character.into();
            let look: AvatarLook = serde_json::from_value(value).ok()?;
            let AvatarLook::Live2d { model, .. } = &look else {
                return None;
            };
            let file = dir
                .join("live2d")
                .join(model.strip_prefix(&format!("{character}/"))?);
            (look.valid() && file.is_file()).then_some(look)
        });
        template.unwrap_or_else(|| AvatarLook::Static {
            character: character.into(),
        })
    }

    fn identity_character(&self) -> String {
        let text = fs::read_to_string(self.workspace_dir().join("IDENTITY.md")).unwrap_or_default();
        field(&text, "Avatar").map_or_else(|| characters()[0].id.clone(), Into::into)
    }

    fn write_manifest(&self, look: &AvatarLook) -> Result<()> {
        let file = ManifestFile { version: 2, look };
        let text = serde_json::to_string_pretty(&file).expect("manifest serializes") + "\n";
        write_atomic(&self.avatar_dir().join(MANIFEST), text.as_bytes())?;
        self.avatar.update(|i| i.look = i.look.wrapping_add(1));
        Ok(())
    }

    /// Copies a character's model into the workspace as a whole, then switches to it (avatar data 1).
    fn copy_model(&self, character: &str) -> Result<()> {
        let from = self.bundled(character).map(|d| d.join("live2d"));
        let (Some(from), dir) = (from, self.avatar_dir()) else {
            return Err(AppError::invalid("character"));
        };
        let (tmp, to) = (dir.join(format!("{character}.tmp")), dir.join(character));
        let _ = fs::remove_dir_all(&tmp);
        copy_dir(&from, &tmp)
            .and_then(|()| match fs::remove_dir_all(&to) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => fs::rename(&tmp, &to),
            })
            .map_err(|_| AppError::not_writable(&to))
    }

    /// Writes a character's default look, or its still portrait, copying its model when it has one.
    pub(crate) fn apply_avatar(&self, character: &str, still: bool) -> Result<()> {
        let look = match self.default_look(character) {
            AvatarLook::Live2d { .. } if still => AvatarLook::Static {
                character: character.into(),
            },
            look => look,
        };
        if matches!(look, AvatarLook::Live2d { .. }) {
            self.copy_model(character)?;
        }
        self.write_manifest(&look)
    }

    /// avatar FR-031, FR-032: the manifest as written, after the one-time upgrade of onboarding's and a repair of
    /// missing model files. Before setup, the default character's look.
    pub fn avatar_manifest(&self) -> Result<AvatarManifest> {
        let ok = |look| {
            let gone = *crate::chat::lock(&self.avatar.gone);
            Ok(AvatarManifest {
                look,
                problem: gone.then_some(AvatarProblem::CharacterGone),
            })
        };
        if !self.state()?.setup.complete {
            return ok(self.default_look(&characters()[0].id));
        }
        let unusable = |problem| {
            Ok(AvatarManifest {
                look: AvatarLook::Static {
                    character: self.identity_character(),
                },
                problem: Some(problem),
            })
        };
        let value = match fs::read(self.avatar_dir().join(MANIFEST)) {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(value) => value,
                Err(_) => return unusable(AvatarProblem::Unreadable),
            },
            Err(_) => Value::Null,
        };
        // Onboarding wrote exactly this; anything else unversioned is a person's edit (avatar R6).
        let written_by_setup = value.as_object().is_some_and(|o| {
            o.len() == 2 && o.get("type") == Some(&"static".into()) && o["character"].is_string()
        });
        if value.is_null() || written_by_setup {
            let character = value["character"]
                .as_str()
                .map_or_else(|| self.identity_character(), Into::into);
            self.apply_avatar(&character, false)?;
            return self.avatar_manifest();
        }
        let look = match serde_json::from_value::<AvatarLook>(value) {
            Ok(look) if look.valid() => look,
            _ => return unusable(AvatarProblem::Unreadable),
        };
        let missing = match &look {
            AvatarLook::Live2d { model, .. } => !self.avatar_dir().join(model).is_file(),
            AvatarLook::Picture { image, .. } => !self.avatar_dir().join(image).is_file(),
            AvatarLook::Static { .. } => false,
        };
        if !missing {
            return ok(look);
        }
        let character = look.character().to_owned();
        match (&look, self.default_look(&character)) {
            (AvatarLook::Live2d { .. }, AvatarLook::Live2d { .. }) => {
                self.copy_model(&character)?;
                self.avatar.update(|i| i.look = i.look.wrapping_add(1));
                ok(look)
            }
            _ if self.bundled(&character).is_none() => {
                let default = characters()[0].id.clone();
                self.apply_avatar(&default, false)?;
                *crate::chat::lock(&self.avatar.gone) = true;
                self.avatar_manifest()
            }
            _ => unusable(AvatarProblem::MissingModel),
        }
    }

    /// avatar FR-029, FR-030: a bundled character, or its still portrait, recorded in the manifest and `IDENTITY.md`.
    pub fn choose_avatar(&self, character: &str, still: bool) -> Result<AvatarManifest> {
        if !characters().iter().any(|c| c.id == character) {
            return Err(AppError::invalid("character"));
        }
        self.apply_avatar(character, still)?;
        *crate::chat::lock(&self.avatar.gone) = false;
        let written = self.edit_workspace("IDENTITY.md", |current| {
            let mut lines: Vec<String> = current.lines().map(Into::into).collect();
            let line = format!("- Avatar: {character}");
            match lines
                .iter()
                .position(|l| l.trim_start().starts_with("- Avatar:"))
            {
                Some(at) if lines[at] == line => return Ok((current.to_owned(), None)),
                Some(at) => lines[at] = line,
                None => lines.push(line),
            }
            let next = lines.join("\n") + "\n";
            Ok((next.clone(), Some(next)))
        })?;
        if let Some(content) = written {
            self.record_written("IDENTITY.md", content)?;
        }
        self.avatar_manifest()
    }

    /// avatar FR-030: the user's own picture, checked by its first bytes, whatever its name says (avatar R7).
    pub fn set_avatar_picture(&self, bytes: &[u8]) -> Result<AvatarManifest> {
        if bytes.len() > PICTURE_MAX_BYTES {
            return Err(AppError::invalid("pictureSize"));
        }
        let ext = match bytes {
            [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => "png",
            [0xff, 0xd8, 0xff, ..] => "jpg",
            [
                b'R',
                b'I',
                b'F',
                b'F',
                _,
                _,
                _,
                _,
                b'W',
                b'E',
                b'B',
                b'P',
                ..,
            ] => "webp",
            _ => return Err(AppError::invalid("pictureType")),
        };
        let character = self.avatar_manifest()?.look.character().to_owned();
        let image = format!("picture.{ext}");
        write_atomic(&self.avatar_dir().join(&image), bytes)?;
        for other in PICTURES.iter().filter(|e| **e != ext) {
            let _ = fs::remove_file(self.avatar_dir().join(format!("picture.{other}")));
        }
        self.write_manifest(&AvatarLook::Picture { character, image })?;
        self.avatar_manifest()
    }

    /// avatar FR-033: a file of the look, read only inside `workspace/avatar/` or a bundled character's `live2d/`.
    pub fn avatar_file(&self, source: &AvatarSource, path: &str) -> Result<Vec<u8>> {
        let invalid = || AppError::invalid("path");
        if !safe(path) || !extension(path, &MODEL_FILES) {
            return Err(invalid());
        }
        let (base, rel) = match source {
            AvatarSource::Workspace => (self.avatar_dir(), path),
            AvatarSource::Bundled { character } => {
                let dir = self.bundled(character).ok_or_else(invalid)?;
                let rel = path
                    .strip_prefix(&format!("{character}/"))
                    .ok_or_else(invalid)?;
                (dir.join("live2d"), rel)
            }
        };
        let file = base.join(rel);
        if fs::symlink_metadata(&file).is_err() {
            return Err(AppError::invalid("missing"));
        }
        let inside = fs::canonicalize(&file)
            .ok()
            .zip(fs::canonicalize(&base).ok())
            .is_some_and(|(f, b)| f.starts_with(b));
        if !inside || fs::symlink_metadata(&file).is_ok_and(|m| m.is_symlink()) {
            return Err(invalid());
        }
        fs::read(&file).map_err(|_| AppError::invalid("missing"))
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifests_parse_and_unsafe_paths_are_refused() {
        let parse = |text: &str| {
            serde_json::from_str::<AvatarLook>(text)
                .ok()
                .filter(AvatarLook::valid)
        };
        let ok = [
            r#"{"version":2,"type":"live2d","character":"pip","model":"pip/pip.model3.json","moods":{"happy":{"expression":"smile"}},"activities":{"idle":{"motionGroup":"Idle"}}}"#,
            r#"{"version":2,"type":"static","character":"pip"}"#,
            r#"{"type":"picture","character":"pip","image":"picture.webp"}"#,
        ];
        for text in ok {
            assert!(parse(text).is_some(), "{text}");
        }
        let refused = [
            r#"{"type":"live2d","character":"pip","model":"../pip.model3.json"}"#,
            r#"{"type":"live2d","character":"pip","model":"pip/pip.json"}"#,
            r#"{"type":"live2d","character":"pip","model":"C:/pip.model3.json"}"#,
            r#"{"type":"picture","character":"pip","image":"picture.gif"}"#,
            r#"{"type":"picture","character":"pip","image":"pip/picture.png"}"#,
            r#"{"type":"cartoon","character":"pip"}"#,
        ];
        for text in refused {
            assert!(parse(text).is_none(), "{text}");
        }
    }
}
