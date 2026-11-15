use crate::chat::{LocalClock, local_today};
use crate::config::now;
use crate::credentials::KeyStatus;
use crate::workspace::{FILES, create_if_absent};
use crate::{AppError, Core, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const SOUL_BUDGET_CHARS: u32 = 8_000;
pub const DEFAULT_NAME: &str = "Pec";
pub const NAME_MAX_CHARS: u32 = 40;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SoulTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub text: &'static str,
}

const TEMPLATES: [SoulTemplate; 5] = [
    SoulTemplate {
        id: "warm-friend",
        title: "Warm friend",
        description: "Kind, attentive and easy to talk to.",
        text: include_str!("../assets/souls/warm-friend.md"),
    },
    SoulTemplate {
        id: "curious-scholar",
        title: "Curious scholar",
        description: "Loves ideas, asks good questions, explains clearly.",
        text: include_str!("../assets/souls/curious-scholar.md"),
    },
    SoulTemplate {
        id: "cheerful-sidekick",
        title: "Cheerful sidekick",
        description: "Upbeat, playful and always up for an adventure.",
        text: include_str!("../assets/souls/cheerful-sidekick.md"),
    },
    SoulTemplate {
        id: "calm-mentor",
        title: "Calm mentor",
        description: "Steady, thoughtful and honest, even when it is hard.",
        text: include_str!("../assets/souls/calm-mentor.md"),
    },
    SoulTemplate {
        id: "blank",
        title: "Blank",
        description: "Start from an empty page.",
        text: include_str!("../assets/souls/blank.md"),
    },
];

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Character {
    pub id: String,
    pub name: String,
}

pub(crate) fn characters() -> Vec<Character> {
    serde_json::from_str(include_str!("../assets/characters.json"))
        .expect("bundled characters parse")
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupOptions {
    pub templates: Vec<SoulTemplate>,
    pub characters: Vec<Character>,
    pub soul_budget_chars: u32,
    pub default_name: &'static str,
    pub name_max_chars: u32,
}

pub fn setup_options() -> SetupOptions {
    SetupOptions {
        templates: TEMPLATES.to_vec(),
        characters: characters(),
        soul_budget_chars: SOUL_BUDGET_CHARS,
        default_name: DEFAULT_NAME,
        name_max_chars: NAME_MAX_CHARS,
    }
}

/// onboarding FR-004: the wizard's steps, in order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SetupStep {
    #[default]
    Key,
    Soul,
    Avatar,
    AboutYou,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupDraft {
    pub step: SetupStep,
    pub soul: SoulDraft,
    pub character_id: String,
    pub about: AboutDraft,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SoulDraft {
    pub template_id: String,
    pub text: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AboutDraft {
    pub call_me: Option<String>,
    pub language: Option<String>,
}

/// onboarding FR-005: every field but the key has a working default.
impl Default for SetupDraft {
    fn default() -> Self {
        let template = &TEMPLATES[0];
        Self {
            step: SetupStep::Key,
            soul: SoulDraft {
                template_id: template.id.into(),
                text: template.text.into(),
                name: DEFAULT_NAME.into(),
            },
            character_id: characters()[0].id.clone(),
            about: AboutDraft::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "state")]
#[ts(export)]
pub enum SetupState {
    InProgress { draft: SetupDraft },
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompanionSummary {
    pub name: String,
    pub character_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppStatus {
    pub setup: SetupState,
    pub key: KeyStatus,
    pub companion: Option<CompanionSummary>,
    pub existing_workspace: bool,
    pub recreated_files: Vec<String>,
    /// The user's "call me" name from `USER.md`, for the greeting (PER-4, PER-5).
    pub call_me: Option<String>,
}

/// How `complete_setup` finishes the wizard (onboarding FR-008).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, TS)]
#[ts(export)]
pub enum SetupCompletion {
    New,
    AdoptExisting,
    ReplaceExisting,
}

fn trimmed(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// onboarding FR-018, FR-020, FR-021: the draft rules. Unknown ids are an interface bug, not a user mistake.
fn normalize(mut draft: SetupDraft) -> Result<SetupDraft> {
    let name = draft.soul.name.trim();
    if name.chars().count() > NAME_MAX_CHARS as usize {
        return Err(AppError::invalid("soul.name"));
    }
    draft.soul.name = if name.is_empty() {
        DEFAULT_NAME.into()
    } else {
        name.into()
    };
    if !TEMPLATES.iter().any(|t| t.id == draft.soul.template_id) {
        return Err(AppError::invalid("soul.templateId"));
    }
    if !characters().iter().any(|c| c.id == draft.character_id) {
        return Err(AppError::invalid("characterId"));
    }
    draft.about.call_me = trimmed(draft.about.call_me);
    draft.about.language = trimmed(draft.about.language);
    Ok(draft)
}

/// The workspace files setup writes, in the order of `FILES` (onboarding data 1).
fn render(draft: &SetupDraft) -> [String; 5] {
    let about = |v: &Option<String>| v.clone().unwrap_or_default();
    [
        draft.soul.text.clone(),
        format!(
            "# Identity\n\n- Name: {}\n- Avatar: {}\n",
            draft.soul.name, draft.character_id
        ),
        format!(
            "# About the user\n\n- Call them: {}\n- Preferred language: {}\n",
            about(&draft.about.call_me),
            about(&draft.about.language)
        ),
        "# Memory\n".into(),
        serde_json::to_string_pretty(
            &serde_json::json!({ "type": "static", "character": draft.character_id }),
        )
        .expect("manifest serializes")
            + "\n",
    ]
}

/// The value of a `- Key: value` line, if present and not empty (onboarding R9).
pub(crate) fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("- ")?
                .strip_prefix(key)?
                .strip_prefix(':')
        })
        .map(str::trim)
        .find(|v| !v.is_empty())
}

impl Core {
    /// Reads the name and avatar from `IDENTITY.md` on every call, so a hand edit is honored (onboarding FR-035).
    fn companion(&self) -> CompanionSummary {
        let text =
            std::fs::read_to_string(self.workspace_dir().join("IDENTITY.md")).unwrap_or_default();
        let defaults = SetupDraft::default();
        CompanionSummary {
            name: field(&text, "Name").unwrap_or(DEFAULT_NAME).into(),
            character_id: field(&text, "Avatar").map_or(defaults.character_id, Into::into),
        }
    }

    fn write_workspace(&self, draft: &SetupDraft) -> Result<Vec<String>> {
        let mut written = Vec::new();
        for (name, content) in FILES.iter().zip(render(draft)) {
            if create_if_absent(&self.workspace_dir().join(name), content.as_bytes())? {
                written.push((*name).to_owned());
            }
        }
        Ok(written)
    }

    /// Makes no network request (onboarding FR-002, FR-007, FR-032).
    pub fn app_status(&self) -> Result<AppStatus> {
        let state = self.state()?;
        let key = self.key_status();
        if state.setup.complete {
            // A file deleted after setup comes back from its default; the others are left alone.
            let companion = self.companion();
            let mut defaults = SetupDraft::default();
            defaults.soul.name.clone_from(&companion.name);
            defaults.character_id.clone_from(&companion.character_id);
            let recreated_files = self.write_workspace(&defaults)?;
            let profile =
                std::fs::read_to_string(self.workspace_dir().join("USER.md")).unwrap_or_default();
            return Ok(AppStatus {
                setup: SetupState::Complete,
                key,
                companion: Some(companion),
                existing_workspace: false,
                recreated_files,
                call_me: field(&profile, "Call them").map(Into::into),
            });
        }
        let mut draft = state.setup.draft.unwrap_or_default();
        if key == KeyStatus::Absent {
            draft.step = SetupStep::Key;
        }
        let existing_workspace = self.workspace_exists();
        Ok(AppStatus {
            setup: SetupState::InProgress { draft },
            key,
            companion: existing_workspace.then(|| self.companion()),
            existing_workspace,
            recreated_files: Vec::new(),
            call_me: None,
        })
    }

    /// onboarding FR-006: called whenever an entry or the step changes.
    pub fn save_setup_draft(&self, draft: SetupDraft) -> Result<()> {
        let draft = normalize(draft)?;
        if draft.step != SetupStep::Key && self.key_status() == KeyStatus::Absent {
            return Err(AppError::invalid("step"));
        }
        self.update_state(|s| s.setup.draft = Some(draft))
    }

    /// onboarding FR-022, FR-023, FR-008.
    pub fn complete_setup(
        &self,
        mode: SetupCompletion,
        clock: &LocalClock,
    ) -> Result<CompanionSummary> {
        let today = local_today(clock)?;
        let state = self.state()?;
        if state.setup.complete {
            return Err(AppError::invalid("setup"));
        }
        if self.key_status() == KeyStatus::Absent {
            return Err(AppError::invalid("step"));
        }
        let draft = normalize(state.setup.draft.unwrap_or_default())?;
        match (mode, self.workspace_exists()) {
            (SetupCompletion::New, false) => {
                self.write_workspace(&draft)?;
                self.apply_avatar(&draft.character_id, false)?;
            }
            (SetupCompletion::AdoptExisting, true) => {}
            (SetupCompletion::ReplaceExisting, true) => {
                self.backup_workspace(now())?;
                self.write_workspace(&draft)?;
                self.apply_avatar(&draft.character_id, false)?;
            }
            _ => return Err(AppError::invalid("mode")),
        }
        self.update_state(|s| {
            s.setup.complete = true;
            s.setup.draft = None;
            s.first_hello_pending = mode != SetupCompletion::AdoptExisting;
        })?;
        // A new companion is born today, with its world (world FR-001, FR-018). An adopted one keeps its
        // files untouched; the main view's `prepare_world` gives it an id later.
        if mode != SetupCompletion::AdoptExisting {
            self.ensure_world(&today)?;
        }
        Ok(self.companion())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> SetupDraft {
        SetupDraft::default()
    }

    #[test]
    fn options_offer_four_templates_a_blank_and_a_character() {
        let options = setup_options();
        assert!(options.templates.iter().filter(|t| t.id != "blank").count() >= 4);
        assert!(
            options
                .templates
                .iter()
                .any(|t| t.id == "blank" && t.text.is_empty())
        );
        assert!(!options.characters.is_empty());
    }

    #[test]
    fn the_name_is_trimmed_bounded_and_defaulted() {
        let long = "x".repeat(41);
        let cases = [
            ("  Ada  ", Some("Ada")),
            ("", Some(DEFAULT_NAME)),
            ("   ", Some(DEFAULT_NAME)),
            (&long[..40], Some(&long[..40])),
            (&long[..], None),
        ];
        for (input, expected) in cases {
            let mut d = draft();
            d.soul.name = input.into();
            let got = normalize(d).map(|d| d.soul.name).ok();
            assert_eq!(got.as_deref(), expected, "{input:?}");
        }
    }

    #[test]
    fn unknown_ids_are_invalid_input() {
        let mut d = draft();
        d.soul.template_id = "nope".into();
        assert_eq!(normalize(d), Err(AppError::invalid("soul.templateId")));
        let mut d = draft();
        d.soul.template_id = "blank".into();
        assert!(normalize(d).is_ok());
        let mut d = draft();
        d.character_id = "nope".into();
        assert_eq!(normalize(d), Err(AppError::invalid("characterId")));
    }

    #[test]
    fn about_values_are_trimmed_and_empty_becomes_none() {
        let cases = [
            (Some(" Sam "), Some("Sam")),
            (Some("   "), None),
            (None, None),
        ];
        for (input, expected) in cases {
            let mut d = draft();
            d.about.call_me = input.map(Into::into);
            d.about.language = input.map(Into::into);
            let d = normalize(d).unwrap();
            assert_eq!(
                (d.about.call_me.as_deref(), d.about.language.as_deref()),
                (expected, expected)
            );
        }
    }

    #[test]
    fn identity_fields_are_read_from_list_lines() {
        let text = "# Identity\n\nA note.\n- Name:   Ada Lovelace \n- Avatar:\n";
        assert_eq!(field(text, "Name"), Some("Ada Lovelace"));
        assert_eq!(field(text, "Avatar"), None);
    }
}
