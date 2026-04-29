//! Agent skills: extensible skill registry, .skill.dal loader, and prompt builder.
//! See docs/SKILLS_DESIGN_PLAN.md.
//!
//! Skills are named bundles (description, tools) that define what an agent can do.
//! Built-in skills: development, creative, office, home, project_init.
//! User-defined skills are loaded from `.skill.dal` files at a configurable path.
//! Persistence is the default for the registry: loaded once at first use.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

// ── Skill definition ────────────────────────────────────────────────

/// A skill definition: name, optional category, description, and granted tools.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillDefinition {
    pub name: String,
    pub category: Option<SkillCategory>,
    pub description: String,
    /// Tool IDs or tags this skill grants (e.g. "read_file", "write_file").
    /// Empty means "description-only" — no additional tools beyond base.
    pub tools: Vec<String>,
    /// Whether this is a built-in skill (cannot be overridden by user definitions).
    pub builtin: bool,
}

/// Skill categories — the four pillars of the default learning path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SkillCategory {
    Development,
    Creative,
    Office,
    Home,
}

impl SkillCategory {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Some(Self::Development),
            "creative" => Some(Self::Creative),
            "office" => Some(Self::Office),
            "home" => Some(Self::Home),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Creative => "creative",
            Self::Office => "office",
            Self::Home => "home",
        }
    }
}

impl std::str::FromStr for SkillCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SkillCategory::from_str(s).ok_or(())
    }
}

// ── Skill registry ──────────────────────────────────────────────────

/// In-memory registry of all known skills (built-in + user-defined).
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillDefinition>,
}

/// Default skill set for the DAL COO app (executive sub-agent / smart assist).
/// When an agent has no explicit skills, use these.
pub const DEFAULT_LEARNING_PATH_SKILLS: &[&str] = &["development", "creative", "office", "home"];

/// Hard skill: always available to every agent.
pub const PROJECT_INIT_SKILL: &str = "project_init";

/// Base tools (all agents): reply, run, search, fetch_url, ask_user, dal_init.
const BASE_TOOLS: &str = "reply, run (shell), search, fetch_url, ask_user, dal_init";

/// Development skill tools.
const DEVELOPMENT_TOOL_IDS: &[&str] = &[
    "read_file",
    "write_file",
    "list_dir",
    "dal_check",
    "dal_run",
];

impl SkillRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with built-in skills.
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register(SkillDefinition {
            name: "project_init".to_string(),
            category: Some(SkillCategory::Development),
            description: "Initialize and set up a DAL project (dal.toml + entry file); run dal init or create layout.".to_string(),
            tools: vec!["dal_init".to_string()],
            builtin: true,
        });
        reg.register(SkillDefinition {
            name: "development".to_string(),
            category: Some(SkillCategory::Development),
            description: "Coding, scripts, DAL projects: read/edit files, dal check/run, debug, explain code.".to_string(),
            tools: DEVELOPMENT_TOOL_IDS.iter().map(|s| s.to_string()).collect(),
            builtin: true,
        });
        reg.register(SkillDefinition {
            name: "creative".to_string(),
            category: Some(SkillCategory::Creative),
            description:
                "Writing, design, ideation: drafts, outlines, copy, storytelling, tone, brainstorm."
                    .to_string(),
            tools: vec![],
            builtin: true,
        });
        reg.register(SkillDefinition {
            name: "office".to_string(),
            category: Some(SkillCategory::Office),
            description: "Tasks, scheduling, docs, meetings: summarize, action lists, email/note drafts, coordination.".to_string(),
            tools: vec![],
            builtin: true,
        });
        reg.register(SkillDefinition {
            name: "home".to_string(),
            category: Some(SkillCategory::Home),
            description: "Personal tasks, routines, health, family, hobbies: reminders, recipes, how-tos, lists.".to_string(),
            tools: vec![],
            builtin: true,
        });
        reg
    }

    /// Register a skill definition. User-defined skills cannot override built-ins.
    pub fn register(&mut self, skill: SkillDefinition) {
        if let Some(existing) = self.skills.get(&skill.name) {
            if existing.builtin && !skill.builtin {
                log::warn!(
                    "Ignoring user-defined skill '{}': cannot override built-in.",
                    skill.name
                );
                return;
            }
        }
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Register multiple skill definitions (e.g. from a loaded file).
    pub fn register_all(&mut self, skills: Vec<SkillDefinition>) {
        for skill in skills {
            self.register(skill);
        }
    }

    /// Look up a skill by name.
    pub fn get(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.get(name)
    }

    /// All registered skill names.
    pub fn names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Resolve a list of skill names against the registry.
    /// Returns the definitions found; unknown names are logged and skipped.
    pub fn resolve(&self, names: &[String]) -> Vec<&SkillDefinition> {
        names
            .iter()
            .filter_map(|name| {
                let def = self.skills.get(name.as_str());
                if def.is_none() {
                    log::debug!("Skill '{}' not found in registry; skipping.", name);
                }
                def
            })
            .collect()
    }

    /// Collect the union of tool IDs granted by the given skill names.
    pub fn tools_for_skills(&self, names: &[String]) -> Vec<String> {
        let mut tools = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for def in self.resolve(names) {
            for tool in &def.tools {
                if seen.insert(tool.clone()) {
                    tools.push(tool.clone());
                }
            }
        }
        tools
    }

    /// Build skill descriptions block for the given skill names.
    pub fn descriptions_for_skills(&self, names: &[String]) -> Vec<String> {
        self.resolve(names)
            .into_iter()
            .filter(|d| !d.description.is_empty())
            .map(|d| format!("- {}: {}", d.name, d.description))
            .collect()
    }

    /// List all registered skills as (name, description). For DAL skills::list().
    pub fn list_names_and_descriptions(&self) -> Vec<(String, String)> {
        self.skills
            .iter()
            .map(|(name, def)| (name.clone(), def.description.clone()))
            .collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── .skill.dal loader ───────────────────────────────────────────────

/// Parse skill definitions from a `.skill.dal` file or string.
///
/// Format (DAL-native declarative):
/// ```text
/// skill "ms_office" {
///   category "office"
///   description "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
///   tools "run" "search"
/// }
/// ```
///
/// Multiple skill blocks per file are supported.
pub fn parse_skill_dal(source: &str) -> Result<Vec<SkillDefinition>, String> {
    let mut skills = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some(&(i, _)) = chars.peek() {
        skip_whitespace_and_comments(&mut chars);
        if chars.peek().is_none() {
            break;
        }

        let rest = &source[chars.peek().map(|&(i, _)| i).unwrap_or(source.len())..];
        if rest.starts_with("skill")
            && rest[5..].starts_with(|c: char| c.is_whitespace() || c == '"')
        {
            for _ in 0..5 {
                chars.next();
            }
            skip_whitespace_and_comments(&mut chars);

            let name = parse_quoted_string(&mut chars, source)
                .ok_or_else(|| format!("Expected quoted skill name near byte {}", i))?;

            skip_whitespace_and_comments(&mut chars);

            expect_char(&mut chars, '{').map_err(|_| {
                format!("Expected '{{' after skill name '{}' near byte {}", name, i)
            })?;

            let mut category: Option<SkillCategory> = None;
            let mut description = String::new();
            let mut tools: Vec<String> = Vec::new();

            loop {
                skip_whitespace_and_comments(&mut chars);
                if chars.peek().map(|&(_, c)| c) == Some('}') {
                    chars.next();
                    break;
                }
                if chars.peek().is_none() {
                    return Err(format!("Unterminated skill block '{}'", name));
                }

                let keyword = parse_identifier(&mut chars, source);
                skip_whitespace_and_comments(&mut chars);

                match keyword.as_str() {
                    "category" => {
                        let val = parse_quoted_string(&mut chars, source).ok_or_else(|| {
                            format!("Expected quoted category in skill '{}'", name)
                        })?;
                        category = SkillCategory::from_str(&val);
                    }
                    "description" => {
                        description = parse_quoted_string(&mut chars, source).ok_or_else(|| {
                            format!("Expected quoted description in skill '{}'", name)
                        })?;
                    }
                    "tools" => {
                        while let Some(tool) = try_parse_quoted_string(&mut chars, source) {
                            tools.push(tool);
                            skip_whitespace_and_comments(&mut chars);
                        }
                    }
                    "" => {
                        return Err(format!("Unexpected character in skill '{}' block", name));
                    }
                    other => {
                        log::warn!("Unknown field '{}' in skill '{}'; skipping.", other, name);
                        skip_to_newline(&mut chars);
                    }
                }
            }

            skills.push(SkillDefinition {
                name,
                category,
                description,
                tools,
                builtin: false,
            });
        } else {
            skip_to_newline(&mut chars);
        }
    }

    Ok(skills)
}

/// Load skill definitions from a single `.skill.dal` file.
pub fn load_skill_file(path: &Path) -> Result<Vec<SkillDefinition>, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    parse_skill_dal(&source)
}

/// Load skill definitions from a path (file or directory of `.skill.dal` files).
pub fn load_skills_from_path(path: &Path) -> Result<Vec<SkillDefinition>, String> {
    if path.is_file() {
        return load_skill_file(path);
    }
    if path.is_dir() {
        let mut all = Vec::new();
        let entries =
            std::fs::read_dir(path).map_err(|e| format!("Failed to read dir {:?}: {}", path, e))?;
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "dal")
                    .unwrap_or(false)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.contains(".skill."))
                        .unwrap_or(false)
            })
            .collect();
        paths.sort();
        for p in paths {
            match load_skill_file(&p) {
                Ok(defs) => all.extend(defs),
                Err(e) => log::warn!("Skipping {:?}: {}", p, e),
            }
        }
        return Ok(all);
    }
    Ok(Vec::new())
}

// ── Parser helpers ──────────────────────────────────────────────────

type CharIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

fn skip_whitespace_and_comments(chars: &mut CharIter) {
    loop {
        match chars.peek() {
            Some(&(_, c)) if c.is_whitespace() => {
                chars.next();
            }
            Some(&(_, '#')) => {
                skip_to_newline(chars);
            }
            Some(&(_, '/')) => {
                let mut clone = chars.clone();
                clone.next();
                if clone.peek().map(|&(_, c)| c) == Some('/') {
                    skip_to_newline(chars);
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
}

fn skip_to_newline(chars: &mut CharIter) {
    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        if c == '\n' {
            break;
        }
    }
}

fn parse_quoted_string(chars: &mut CharIter, _source: &str) -> Option<String> {
    if chars.peek().map(|&(_, c)| c) != Some('"') {
        return None;
    }
    chars.next(); // consume opening "
    let mut result = String::new();
    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        if c == '"' {
            return Some(result);
        }
        if c == '\\' {
            if let Some(&(_, escaped)) = chars.peek() {
                chars.next();
                match escaped {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
                continue;
            }
        }
        result.push(c);
    }
    None // unterminated string
}

fn try_parse_quoted_string(chars: &mut CharIter, source: &str) -> Option<String> {
    if chars.peek().map(|&(_, c)| c) == Some('"') {
        parse_quoted_string(chars, source)
    } else {
        None
    }
}

fn parse_identifier(chars: &mut CharIter, _source: &str) -> String {
    let mut result = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
            chars.next();
        } else {
            break;
        }
    }
    result
}

fn expect_char(chars: &mut CharIter, expected: char) -> Result<(), ()> {
    if chars.peek().map(|&(_, c)| c) == Some(expected) {
        chars.next();
        Ok(())
    } else {
        Err(())
    }
}

// ── Skills config ───────────────────────────────────────────────────

/// Resolve the skills path from env and agent.toml/dal.toml.
/// Returns None if no skills path is configured and no default directory exists.
pub fn resolve_skills_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("DAL_SKILLS_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
        log::warn!("DAL_SKILLS_PATH={:?} does not exist.", path);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    for name in &["agent.toml", "dal.toml"] {
        let toml_path = cwd.join(name);
        if let Ok(content) = std::fs::read_to_string(&toml_path) {
            if let Ok(table) = content.parse::<toml::Table>() {
                if let Some(sp) = table
                    .get("agent")
                    .and_then(|v| v.as_table())
                    .and_then(|t| t.get("skills_path"))
                    .and_then(|v| v.as_str())
                {
                    let mut p = PathBuf::from(sp);
                    if !p.is_absolute() {
                        p = cwd.join(p);
                    }
                    if p.exists() {
                        return Some(p);
                    }
                    log::warn!("skills_path={:?} from {} does not exist.", p, name);
                }
            }
        }
    }

    // Default: .dal/ directory if it contains any .skill.dal files
    let dal_dir = cwd.join(".dal");
    if dal_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&dal_dir) {
            let has_skill_files = entries.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(".skill.") && n.ends_with(".dal"))
                    .unwrap_or(false)
            });
            if has_skill_files {
                return Some(dal_dir);
            }
        }
    }

    None
}

// ── Global registry (loaded once) ───────────────────────────────────

fn get_global_registry() -> &'static Mutex<SkillRegistry> {
    static REGISTRY: OnceLock<Mutex<SkillRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut reg = SkillRegistry::with_builtins();
        if let Some(path) = resolve_skills_path() {
            match load_skills_from_path(&path) {
                Ok(defs) => {
                    let count = defs.len();
                    reg.register_all(defs);
                    if count > 0 {
                        log::info!("Loaded {} user-defined skill(s) from {:?}", count, path);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to load user skills from {:?}: {}", path, e);
                }
            }
        }
        Mutex::new(reg)
    })
}

/// Get a clone of the global skill registry.
pub fn global_registry() -> SkillRegistry {
    get_global_registry().lock().unwrap().clone()
}

/// List skill names and descriptions. If path is None, uses the global registry (built-ins + resolved skills_path). If path is Some(p), loads from that directory only.
pub fn list_skills(path: Option<&Path>) -> Result<Vec<(String, String)>, String> {
    let list = if let Some(p) = path {
        let defs = load_skills_from_path(p)?;
        defs.into_iter().map(|d| (d.name, d.description)).collect()
    } else {
        global_registry().list_names_and_descriptions()
    };
    Ok(list)
}

/// Register additional skills into the global registry (e.g. from runtime or tests).
pub fn register_global_skills(skills: Vec<SkillDefinition>) {
    let mut reg = get_global_registry().lock().unwrap();
    reg.register_all(skills);
}

// ── Programmatic encouragement ──────────────────────────────────────

/// Fixed prompt block encouraging the agent to use search and past experience.
/// Always included in every agent prompt.
pub const ENCOURAGEMENT_BLOCK: &str = "\
When you don't have a specific tool for a task, use **search** or **run** to find documentation, \
APIs, or commands you need. Use your past executions in context as examples of how you've solved \
similar problems. If you've successfully discovered information through search or run before, \
apply the same approach to new challenges.";

/// Build an optional meta-from-memory sentence when the agent has relevant history.
/// Inspects the provided context (e.g. evolve content or memory summary) for evidence
/// of past tool use (search, run) and returns a reinforcing sentence if found.
pub fn meta_from_memory(context_text: &str) -> Option<String> {
    if context_text.is_empty() {
        return None;
    }
    let lower = context_text.to_lowercase();
    let has_search = lower.contains("search") || lower.contains("searched");
    let has_run = lower.contains("\"action\":\"run\"") || lower.contains("used run");

    if has_search && has_run {
        Some("In past executions you have used search and run when you didn't have a dedicated tool. This approach has worked well — apply it again when needed.".to_string())
    } else if has_search {
        Some("In past executions you have used search to find information when you didn't have a dedicated tool. Continue this approach when needed.".to_string())
    } else if has_run {
        Some("In past executions you have used run to execute commands when you didn't have a dedicated tool. Continue this approach when needed.".to_string())
    } else {
        None
    }
}

/// Build a reinforcement note for memory when the agent successfully discovers something via search or run.
pub fn reinforcement_note(action: &str, discovery: &str) -> String {
    format!(
        "Used {} to discover: {}. Apply this pattern in future when no direct tool is available.",
        action, discovery
    )
}

// ── tools_description_for_skills (backward-compatible API) ──────────

/// Build the tools_description string for the agent prompt from the given skill names.
/// Uses the global registry. Always includes project_init (hard skill) and encouragement.
/// If skills is empty, uses DEFAULT_LEARNING_PATH_SKILLS.
/// This is the main entry point used by agent_serve.rs.
pub fn tools_description_for_skills(skills: &[String]) -> String {
    let registry = global_registry();
    tools_description_for_skills_with_registry(skills, &registry, None)
}

/// Build the tools_description string using an explicit registry and optional memory context.
/// Richer variant for callers that have access to memory/evolve content.
pub fn tools_description_for_skills_with_registry(
    skills: &[String],
    registry: &SkillRegistry,
    memory_context: Option<&str>,
) -> String {
    let mut effective: Vec<String> = skills.to_vec();
    if effective.is_empty() {
        effective = DEFAULT_LEARNING_PATH_SKILLS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
    }
    if !effective.iter().any(|s| s == PROJECT_INIT_SKILL) {
        effective.insert(0, PROJECT_INIT_SKILL.to_string());
    }

    let skill_descriptions = registry.descriptions_for_skills(&effective);
    let resolved_tools = registry.tools_for_skills(&effective);

    let skills_block = if skill_descriptions.is_empty() {
        "You are a helpful assistant.".to_string()
    } else {
        format!(
            "You are operating as the DAL COO surface, serving your principal. Skills:\n{}",
            skill_descriptions.join("\n")
        )
    };

    let has_development = effective.iter().any(|s| s == "development");
    let dev_tools_str = DEVELOPMENT_TOOL_IDS.join(", ");
    let tools_list = if has_development {
        format!("{}; {}", BASE_TOOLS, dev_tools_str)
    } else {
        let mut extra: Vec<&str> = Vec::new();
        for tool in &resolved_tools {
            let t = tool.as_str();
            if !BASE_TOOLS.contains(t) && !DEVELOPMENT_TOOL_IDS.contains(&t) {
                extra.push(t);
            }
        }
        if extra.is_empty() {
            BASE_TOOLS.to_string()
        } else {
            format!("{}; {}", BASE_TOOLS, extra.join(", "))
        }
    };

    let json_tools = if has_development {
        r#"Reply with JSON: {"action":"reply","text":"..."} or {"action":"run","cmd":"..."} or {"action":"search","query":"..."} or {"action":"fetch_url","url":"https://..."} or {"action":"ask_user","message":"..."} or {"action":"dal_init","template":"general"|"chain"|"iot"|"agent"} or {"action":"read_file","path":"..."} or {"action":"write_file","path":"...","contents":"..."} or {"action":"list_dir","path":"."} or {"action":"dal_check","path":"file.dal"} or {"action":"dal_run","path":"file.dal"}. Paths are relative to working directory."#
    } else {
        r#"Reply with JSON: {"action":"reply","text":"..."} or {"action":"run","cmd":"..."} or {"action":"search","query":"..."} or {"action":"fetch_url","url":"https://..."} or {"action":"ask_user","message":"..."} or {"action":"dal_init","template":"general"|"chain"|"iot"|"agent"} to initialize a DAL project (omit template for general)."#
    };

    let meta_sentence = memory_context
        .and_then(|ctx| meta_from_memory(ctx))
        .unwrap_or_default();

    let encouragement = if meta_sentence.is_empty() {
        ENCOURAGEMENT_BLOCK.to_string()
    } else {
        format!("{} {}", ENCOURAGEMENT_BLOCK, meta_sentence)
    };

    format!(
        "{} Available tools: {}. {} {} When the objective is satisfied, use reply with your final answer. Respect the constraints below.",
        skills_block, tools_list, json_tools, encouragement
    )
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_skills_produce_non_empty_description() {
        let desc = tools_description_for_skills(&[]);
        assert!(desc.contains("DAL COO"));
        assert!(desc.contains("development"));
        assert!(desc.contains("creative"));
        assert!(desc.contains("office"));
        assert!(desc.contains("home"));
        assert!(desc.contains("project_init"));
        assert!(desc.contains("reply"));
        assert!(desc.contains("ask_user"));
    }

    #[test]
    fn explicit_skills_used() {
        let desc = tools_description_for_skills(&["development".to_string()]);
        assert!(desc.contains("development"));
        assert!(desc.contains("project_init"));
    }

    #[test]
    fn encouragement_always_present() {
        let desc = tools_description_for_skills(&[]);
        assert!(desc.contains("When you don't have a specific tool"));
        assert!(desc.contains("search"));
    }

    #[test]
    fn registry_builtins() {
        let reg = SkillRegistry::with_builtins();
        assert!(reg.get("development").is_some());
        assert!(reg.get("creative").is_some());
        assert!(reg.get("office").is_some());
        assert!(reg.get("home").is_some());
        assert!(reg.get("project_init").is_some());
        assert_eq!(reg.len(), 5);
    }

    #[test]
    fn registry_user_skill_additive() {
        let mut reg = SkillRegistry::with_builtins();
        reg.register(SkillDefinition {
            name: "ms_office".to_string(),
            category: Some(SkillCategory::Office),
            description: "Use MS Office tools via run or scripts.".to_string(),
            tools: vec!["run".to_string()],
            builtin: false,
        });
        assert_eq!(reg.len(), 6);
        assert!(reg.get("ms_office").is_some());
        assert_eq!(
            reg.get("ms_office").unwrap().description,
            "Use MS Office tools via run or scripts."
        );
    }

    #[test]
    fn registry_cannot_override_builtin() {
        let mut reg = SkillRegistry::with_builtins();
        reg.register(SkillDefinition {
            name: "development".to_string(),
            category: Some(SkillCategory::Development),
            description: "Overridden!".to_string(),
            tools: vec![],
            builtin: false,
        });
        assert!(reg.get("development").unwrap().builtin);
        assert_ne!(reg.get("development").unwrap().description, "Overridden!");
    }

    #[test]
    fn registry_resolve_skills() {
        let mut reg = SkillRegistry::with_builtins();
        reg.register(SkillDefinition {
            name: "samsung_smart_home".to_string(),
            category: Some(SkillCategory::Home),
            description: "Control Samsung SmartThings devices.".to_string(),
            tools: vec!["run".to_string(), "search".to_string()],
            builtin: false,
        });
        let resolved = reg.resolve(&[
            "development".to_string(),
            "samsung_smart_home".to_string(),
            "nonexistent".to_string(),
        ]);
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn registry_tools_union() {
        let mut reg = SkillRegistry::with_builtins();
        reg.register(SkillDefinition {
            name: "social_media".to_string(),
            category: Some(SkillCategory::Office),
            description: "Social media management.".to_string(),
            tools: vec![
                "run".to_string(),
                "search".to_string(),
                "custom_api".to_string(),
            ],
            builtin: false,
        });
        let tools = reg.tools_for_skills(&["development".to_string(), "social_media".to_string()]);
        assert!(tools.contains(&"read_file".to_string()));
        assert!(tools.contains(&"custom_api".to_string()));
        assert!(tools.contains(&"run".to_string()));
    }

    #[test]
    fn user_skill_in_prompt() {
        let mut reg = SkillRegistry::with_builtins();
        reg.register(SkillDefinition {
            name: "ms_office".to_string(),
            category: Some(SkillCategory::Office),
            description: "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
                .to_string(),
            tools: vec!["run".to_string()],
            builtin: false,
        });
        let desc = tools_description_for_skills_with_registry(
            &["office".to_string(), "ms_office".to_string()],
            &reg,
            None,
        );
        assert!(desc.contains("ms_office"));
        assert!(desc.contains("MS Office"));
    }

    #[test]
    fn parse_skill_dal_basic() {
        let source = r#"
skill "ms_office" {
  category "office"
  description "Use MS Office tools (Word, Excel, Outlook) via run or scripts."
  tools "run" "search"
}

skill "samsung_smart_home" {
  category "home"
  description "Control and query Samsung SmartThings devices."
  tools "run" "search"
}
"#;
        let skills = parse_skill_dal(source).unwrap();
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "ms_office");
        assert_eq!(skills[0].category, Some(SkillCategory::Office));
        assert!(skills[0].description.contains("MS Office"));
        assert_eq!(
            skills[0].tools,
            vec!["run".to_string(), "search".to_string()]
        );
        assert!(!skills[0].builtin);
        assert_eq!(skills[1].name, "samsung_smart_home");
        assert_eq!(skills[1].category, Some(SkillCategory::Home));
    }

    #[test]
    fn parse_skill_dal_with_comments() {
        let source = r#"
# My custom skills
skill "social_media_manager" {
  category "office"
  description "Manage social media accounts, schedule posts, analyze engagement."
  tools "run" "search"
}
// Another comment style
"#;
        let skills = parse_skill_dal(source).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "social_media_manager");
    }

    #[test]
    fn parse_skill_dal_no_category() {
        let source = r#"
skill "custom_tool" {
  description "A custom tool with no category."
  tools "run"
}
"#;
        let skills = parse_skill_dal(source).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].category, None);
    }

    #[test]
    fn parse_skill_dal_description_only() {
        let source = r#"
skill "advisor" {
  category "creative"
  description "Provide strategic advice and brainstorming."
}
"#;
        let skills = parse_skill_dal(source).unwrap();
        assert_eq!(skills.len(), 1);
        assert!(skills[0].tools.is_empty());
    }

    #[test]
    fn parse_skill_dal_empty() {
        let skills = parse_skill_dal("").unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn parse_skill_dal_unterminated_error() {
        let source = r#"skill "broken" {"#;
        assert!(parse_skill_dal(source).is_err());
    }

    #[test]
    fn meta_from_memory_both() {
        let ctx =
            r#"I searched for the API documentation. Then I used run to install the package."#;
        let meta = meta_from_memory(ctx);
        assert!(meta.is_some());
        assert!(meta.unwrap().contains("search and run"));
    }

    #[test]
    fn meta_from_memory_search_only() {
        let meta = meta_from_memory("I searched for the docs.");
        assert!(meta.is_some());
        assert!(meta.unwrap().contains("search"));
    }

    #[test]
    fn meta_from_memory_empty() {
        assert!(meta_from_memory("").is_none());
    }

    #[test]
    fn meta_from_memory_no_match() {
        assert!(meta_from_memory("The agent replied with the answer.").is_none());
    }

    #[test]
    fn reinforcement_note_format() {
        let note = reinforcement_note("search", "MS Office API endpoint");
        assert!(note.contains("search"));
        assert!(note.contains("MS Office API endpoint"));
    }

    #[test]
    fn load_skills_from_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("office.skill.dal");
        std::fs::write(
            &file_path,
            r#"
skill "ms_office" {
  category "office"
  description "Use MS Office tools."
  tools "run"
}
"#,
        )
        .unwrap();

        let skills = load_skills_from_path(dir.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "ms_office");
    }

    #[test]
    fn load_skills_ignores_non_skill_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("agent.dal"), "// not a skill file").unwrap();
        std::fs::write(
            dir.path().join("my.skill.dal"),
            r#"skill "test" { description "test" }"#,
        )
        .unwrap();

        let skills = load_skills_from_path(dir.path()).unwrap();
        assert_eq!(skills.len(), 1);
    }

    #[test]
    fn skill_category_roundtrip() {
        for name in &["development", "dev", "creative", "office", "home"] {
            assert!(
                SkillCategory::from_str(name).is_some(),
                "Failed for {}",
                name
            );
        }
        assert!(SkillCategory::from_str("nonexistent").is_none());
        assert_eq!(SkillCategory::Development.as_str(), "development");
    }

    #[test]
    fn full_integration_user_skill_in_registry_and_prompt() {
        let mut reg = SkillRegistry::with_builtins();
        let source = r#"
skill "calendar_scheduler" {
  category "office"
  description "Manage a custom calendar platform for scheduling meetings and events."
  tools "run" "search"
}
"#;
        let defs = parse_skill_dal(source).unwrap();
        reg.register_all(defs);

        let desc = tools_description_for_skills_with_registry(
            &["office".to_string(), "calendar_scheduler".to_string()],
            &reg,
            Some("Previously I searched for the calendar API and used run to create an event."),
        );
        assert!(desc.contains("calendar_scheduler"));
        assert!(desc.contains("custom calendar platform"));
        assert!(desc.contains("When you don't have a specific tool"));
        assert!(desc.contains("past executions you have used search and run"));
    }
}
