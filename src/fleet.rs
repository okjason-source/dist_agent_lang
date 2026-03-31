//! Fleet: named set of agents, optionally created from a mold (N instances).
//! Off-chain only. Storage: in-memory; when base path is provided, also persisted to `.dal/fleets.json`.
//! See COMPREHENSIVE_AGENT_AND_MOLD_PLANS.md §5.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Result of running a fleet (dispatching last_deployed_task to each member).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunReport {
    pub fleet_name: String,
    pub members_dispatched: u32,
    pub errors: Vec<String>,
}

/// A fleet is a named set of agent IDs. Optionally created from a mold (one mold + N instances).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fleet {
    pub name: String,
    /// If created from mold, the mold source (path or ipfs://cid).
    pub mold_source: Option<String>,
    /// Agent IDs in this fleet (order preserved).
    pub member_ids: Vec<String>,
    /// Last task deployed to this fleet (set by deploy). Runners/automation read this to know what to run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_deployed_task: Option<String>,
    /// When last_deployed_task was set (ISO-ish or opaque string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_deployed_at: Option<String>,
    /// Params used when creating from mold (and when scaling up). Reused by scale() for new members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_create_params: Option<HashMap<String, String>>,
}

static FLEETS: OnceLock<Mutex<HashMap<String, Fleet>>> = OnceLock::new();

fn fleets() -> std::sync::MutexGuard<'static, HashMap<String, Fleet>> {
    FLEETS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

fn path_for_fleets(base: &Path) -> PathBuf {
    base.join(".dal").join("fleets.json")
}

fn load_from_file(base: &Path) -> HashMap<String, Fleet> {
    let p = path_for_fleets(base);
    if let Ok(data) = fs::read_to_string(&p) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, Fleet>>(&data) {
            return map;
        }
    }
    HashMap::new()
}

fn save_to_file(base: &Path, map: &HashMap<String, Fleet>) -> Result<(), String> {
    let p = path_for_fleets(base);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create .dal: {}", e))?;
    }
    let data = serde_json::to_string_pretty(map).map_err(|e| format!("serialize fleets: {}", e))?;
    fs::write(&p, data).map_err(|e| format!("write {}: {}", p.display(), e))?;
    Ok(())
}

fn ensure_loaded(base: Option<&Path>) {
    if let Some(b) = base {
        let loaded = load_from_file(b);
        let mut f = fleets();
        *f = loaded;
    }
}

fn save_if_base(base: Option<&Path>) -> Result<(), String> {
    if let Some(b) = base {
        let f = fleets();
        save_to_file(b, &f)?;
    }
    Ok(())
}

/// Create an empty fleet with the given name. Fails if name already exists.
/// If base is Some, fleet data is persisted to base/.dal/fleets.json.
pub fn create(name: &str, base: Option<&Path>) -> Result<(), String> {
    ensure_loaded(base);
    let name = name.to_string();
    if name.is_empty() {
        return Err("fleet name cannot be empty".to_string());
    }
    let mut f = fleets();
    if f.contains_key(&name) {
        return Err(format!("fleet '{}' already exists", name));
    }
    f.insert(
        name.clone(),
        Fleet {
            name,
            mold_source: None,
            member_ids: Vec::new(),
            last_deployed_task: None,
            last_deployed_at: None,
            last_create_params: None,
        },
    );
    drop(f);
    save_if_base(base)
}

/// Create a fleet by spawning N agents from a mold. Same principal/mold/fallback rules as single create.
/// Each agent is named `<fleet_name>_0`, `<fleet_name>_1`, ...; params apply to each.
/// Fleet is persisted to base/.dal/fleets.json.
pub fn create_from_mold(
    name: &str,
    mold_source: &str,
    count: u32,
    base: &Path,
    params: Option<&HashMap<String, String>>,
) -> Result<(), String> {
    if count == 0 {
        return Err("fleet from mold requires count >= 1".to_string());
    }
    if count > 1000 {
        return Err("fleet from mold count capped at 1000".to_string());
    }
    ensure_loaded(Some(base));
    let name = name.to_string();
    let f = fleets();
    if f.contains_key(&name) {
        return Err(format!("fleet '{}' already exists", name));
    }
    drop(f);

    let mut member_ids = Vec::with_capacity(count as usize);
    for i in 0..count {
        let agent_name = format!("{}_{}", name, i);
        let ctx = crate::mold::create_from_mold_source(
            mold_source,
            base,
            Some(agent_name.as_str()),
            params,
        )
        .map_err(|e| format!("spawn agent {} from mold: {}", i, e))?;
        member_ids.push(ctx.agent_id);
    }

    let last_create_params =
        params.map(|p| p.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    let mut f = fleets();
    f.insert(
        name.clone(),
        Fleet {
            name,
            mold_source: Some(mold_source.to_string()),
            member_ids,
            last_deployed_task: None,
            last_deployed_at: None,
            last_create_params,
        },
    );
    drop(f);
    save_if_base(Some(base))
}

/// Add N members to an existing fleet from a mold. If the fleet is empty, sets mold_source.
/// Use this to populate an empty fleet or add more members (same as scale-up logic but explicit).
pub fn add_from_mold(
    name: &str,
    mold_source: &str,
    count: u32,
    base: &Path,
    params: Option<&HashMap<String, String>>,
) -> Result<(), String> {
    if count == 0 {
        return Err("add_from_mold requires count >= 1".to_string());
    }
    if count > 1000 {
        return Err("add_from_mold count capped at 1000".to_string());
    }
    ensure_loaded(Some(base));
    let f = fleets();
    let mut fleet = f
        .get(name)
        .cloned()
        .ok_or_else(|| format!("fleet '{}' not found", name))?;
    let start = fleet.member_ids.len();
    let last_create_params =
        params.map(|p| p.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    if fleet.mold_source.is_none() {
        fleet.mold_source = Some(mold_source.to_string());
    }
    if fleet.last_create_params.is_none() {
        fleet.last_create_params = last_create_params.clone();
    }
    drop(f);

    let mut member_ids = fleet.member_ids;
    for i in 0..count {
        let agent_name = format!("{}_{}", name, start + i as usize);
        let ctx = crate::mold::create_from_mold_source(
            mold_source,
            base,
            Some(agent_name.as_str()),
            params,
        )
        .map_err(|e| format!("add member {} from mold: {}", i, e))?;
        member_ids.push(ctx.agent_id);
    }

    let mut f = fleets();
    if let Some(fleet) = f.get_mut(name) {
        fleet.member_ids = member_ids;
        fleet
            .mold_source
            .get_or_insert_with(|| mold_source.to_string());
        if fleet.last_create_params.is_none() {
            fleet.last_create_params = last_create_params;
        }
    }
    drop(f);
    save_if_base(Some(base))
}

/// Add a single agent (by ID) to an existing fleet. Agent must already exist elsewhere; this only registers the ID.
pub fn add_member(name: &str, agent_id: &str, base: Option<&Path>) -> Result<(), String> {
    if agent_id.trim().is_empty() {
        return Err("agent_id cannot be empty".to_string());
    }
    ensure_loaded(base);
    let base_path = base.ok_or("add_member requires a base path for persistence")?;
    let mut f = fleets();
    let fleet = f
        .get_mut(name)
        .ok_or_else(|| format!("fleet '{}' not found", name))?;
    if fleet.member_ids.contains(&agent_id.to_string()) {
        drop(f);
        return Ok(());
    }
    fleet.member_ids.push(agent_id.to_string());
    drop(f);
    save_if_base(Some(base_path))
}

/// Record a deployment (task) for a fleet. Updates last_deployed_task and last_deployed_at; persists if base is set.
/// Runners/automation can read the fleet file to see what to execute for each fleet.
pub fn deploy(name: &str, task: &str, base: Option<&Path>) -> Result<(), String> {
    if task.trim().is_empty() {
        return Err("deploy task cannot be empty".to_string());
    }
    ensure_loaded(base);
    let base_path = base.ok_or("deploy requires a base path for persistence")?;
    let mut f = fleets();
    let mut fleet = f
        .get(name)
        .cloned()
        .ok_or_else(|| format!("fleet '{}' not found", name))?;
    fleet.last_deployed_task = Some(task.trim().to_string());
    fleet.last_deployed_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string()),
    );
    f.insert(name.to_string(), fleet);
    drop(f);
    save_if_base(Some(base_path))
}

/// List fleet names. If base is Some, loads from base/.dal/fleets.json first.
pub fn list(base: Option<&Path>) -> Vec<String> {
    ensure_loaded(base);
    let f = fleets();
    let mut names: Vec<String> = f.keys().cloned().collect();
    names.sort();
    names
}

/// Get fleet by name, if it exists. If base is Some, loads from file first.
pub fn show(name: &str, base: Option<&Path>) -> Option<Fleet> {
    ensure_loaded(base);
    let f = fleets();
    f.get(name).cloned()
}

/// Remove a fleet (agents remain in runtime; only the grouping is removed). Returns true if removed.
/// If base is Some, persists after remove.
pub fn delete(name: &str, base: Option<&Path>) -> Result<bool, String> {
    ensure_loaded(base);
    let mut f = fleets();
    let removed = f.remove(name).is_some();
    drop(f);
    if removed {
        save_if_base(base)?;
    }
    Ok(removed)
}

/// Resize fleet to N members. If base is Some, persists after change.
/// - Scale down (N < current): truncates member list to N (agent contexts are not removed).
/// - Scale up (N > current): only allowed when fleet has mold_source; spawns (N - current) new agents from the same mold and appends to members.
/// - Empty fleets (no mold_source) cannot be scaled up.
pub fn scale(name: &str, n: u32, base: Option<&Path>) -> Result<(), String> {
    if n > 1000 {
        return Err("fleet scale count capped at 1000".to_string());
    }
    ensure_loaded(base);
    let base_path = base.ok_or("scale requires a base path for persistence")?;

    let mut f = fleets();
    let fleet = f
        .get(name)
        .cloned()
        .ok_or_else(|| format!("fleet '{}' not found", name))?;
    let current = fleet.member_ids.len() as u32;

    if n == current {
        drop(f);
        return Ok(());
    }

    if n < current {
        let mut fleet = fleet;
        fleet.member_ids.truncate(n as usize);
        f.insert(name.to_string(), fleet);
        drop(f);
        return save_if_base(Some(base_path));
    }

    // n > current: scale up
    let mold_source = fleet.mold_source.as_deref().ok_or_else(|| {
        format!(
            "fleet '{}' has no mold; cannot scale up (create from mold first)",
            name
        )
    })?;

    let mut member_ids = fleet.member_ids;
    drop(f);

    let params_ref = fleet.last_create_params.as_ref();
    for i in current..n {
        let agent_name = format!("{}_{}", name, i);
        let ctx = crate::mold::create_from_mold_source(
            mold_source,
            base_path,
            Some(agent_name.as_str()),
            params_ref,
        )
        .map_err(|e| format!("spawn agent {} from mold: {}", i, e))?;
        member_ids.push(ctx.agent_id);
    }

    let mut f = fleets();
    if let Some(fleet) = f.get_mut(name) {
        fleet.member_ids = member_ids;
    }
    drop(f);
    save_if_base(Some(base_path))
}

/// Run fleets: for each fleet with last_deployed_task (and name matching filter if given), ensure it has members
/// (if empty and has mold_source, add one from mold), then dispatch last_deployed_task to each member via
/// agent::coordinate(agent_id, task, "task_distribution"). Returns one RunReport per fleet processed.
pub fn run(name_filter: Option<&str>, base: &Path) -> Result<Vec<RunReport>, String> {
    ensure_loaded(Some(base));
    let f = fleets();
    let names: Vec<String> = f
        .keys()
        .filter(|n| name_filter.map(|f| *n == f).unwrap_or(true))
        .cloned()
        .collect();
    drop(f);

    let mut reports = Vec::new();
    for name in names {
        let fleet = show(&name, Some(base)).ok_or_else(|| format!("fleet '{}' not found", name))?;
        let Some(ref task_desc) = fleet.last_deployed_task else {
            continue;
        };
        let mut member_ids = fleet.member_ids.clone();
        if member_ids.is_empty() {
            let mold = fleet.mold_source.as_deref().ok_or_else(|| {
                format!(
                    "fleet '{}' has no members and no mold_source; add members or deploy from mold first",
                    name
                )
            })?;
            add_from_mold(&name, mold, 1, base, fleet.last_create_params.as_ref())?;
            if let Some(updated) = show(&name, Some(base)) {
                member_ids = updated.member_ids;
            }
        }
        let mut errors = Vec::new();
        let mut dispatched = 0u32;
        for agent_id in &member_ids {
            let task_id = uuid::Uuid::new_v4().to_string();
            let task = match crate::stdlib::agent::create_agent_task(
                task_id.clone(),
                task_desc.clone(),
                "medium",
            ) {
                Some(t) => t,
                None => {
                    errors.push(format!(
                        "{}: create_agent_task failed (invalid priority)",
                        agent_id
                    ));
                    continue;
                }
            };
            match crate::stdlib::agent::coordinate(agent_id, task, "task_distribution") {
                Ok(true) => dispatched += 1,
                Ok(false) => errors.push(format!("{}: coordinate returned false", agent_id)),
                Err(e) => errors.push(format!("{}: {}", agent_id, e)),
            }
        }
        reports.push(RunReport {
            fleet_name: name,
            members_dispatched: dispatched,
            errors,
        });
    }
    Ok(reports)
}

/// Health summary for a fleet (member count, mold presence, last deployment).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FleetHealth {
    pub name: String,
    pub member_count: usize,
    pub has_mold: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deployed_task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deployed_at: Option<String>,
    pub status: String,
}

/// Report health for a fleet. Status is "ok" if fleet exists and has at least one member or a mold; "empty" if no members and no mold.
pub fn health(name: &str, base: Option<&Path>) -> Result<FleetHealth, String> {
    ensure_loaded(base);
    let f = fleets();
    let fleet = f
        .get(name)
        .cloned()
        .ok_or_else(|| format!("fleet '{}' not found", name))?;
    drop(f);
    let status = if !fleet.member_ids.is_empty() || fleet.mold_source.is_some() {
        "ok"
    } else {
        "empty"
    };
    Ok(FleetHealth {
        name: fleet.name,
        member_count: fleet.member_ids.len(),
        has_mold: fleet.mold_source.is_some(),
        last_deployed_task: fleet.last_deployed_task,
        last_deployed_at: fleet.last_deployed_at,
        status: status.to_string(),
    })
}

/// Export format for fleet(s) to infrastructure-as-code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    K8s,
    DockerCompose,
}

/// Export fleet(s) to YAML (k8s Job list or docker-compose). If name is None, exports all fleets.
pub fn export(
    name: Option<&str>,
    base: Option<&Path>,
    format: ExportFormat,
) -> Result<String, String> {
    ensure_loaded(base);
    let fleets_to_export: Vec<Fleet> = {
        let f = fleets();
        let v: Vec<Fleet> = match name {
            Some(n) => f.get(n).cloned().into_iter().collect(),
            None => f.values().cloned().collect(),
        };
        v
    };
    if fleets_to_export.is_empty() {
        return Err(name
            .map(|n| format!("fleet '{}' not found", n))
            .unwrap_or_else(|| "no fleets to export".to_string()));
    }
    let refs: Vec<&Fleet> = fleets_to_export.iter().collect();
    match format {
        ExportFormat::K8s => export_k8s(&refs),
        ExportFormat::DockerCompose => export_docker_compose(&refs),
    }
}

fn export_k8s(fleets: &[&Fleet]) -> Result<String, String> {
    let mut out = String::from("apiVersion: batch/v1\nkind: JobList\nitems:\n");
    for fleet in fleets {
        for (i, agent_id) in fleet.member_ids.iter().enumerate() {
            let job_name = format!("{}-{}", fleet.name, i)
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect::<String>();
            out.push_str(&format!(
                "- apiVersion: batch/v1\n  kind: Job\n  metadata:\n    name: {}\n  spec:\n    template:\n      spec:\n        containers:\n        - name: agent\n          image: dal-agent:latest\n          env:\n          - name: DAL_AGENT_ID\n            value: \"{}\"\n        restartPolicy: Never\n",
                job_name, agent_id
            ));
        }
    }
    Ok(out)
}

fn export_docker_compose(fleets: &[&Fleet]) -> Result<String, String> {
    let mut out = String::from("version: \"3\"\nservices:\n");
    for fleet in fleets {
        for (i, agent_id) in fleet.member_ids.iter().enumerate() {
            let svc = format!("{}-{}", fleet.name, i)
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect::<String>();
            out.push_str(&format!(
                "  {}:\n    image: dal-agent:latest\n    environment:\n      DAL_AGENT_ID: \"{}\"\n",
                svc, agent_id
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn create_and_list_empty_fleet() {
        let name = format!(
            "test_fleet_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        assert!(create(&name, None).is_ok());
        let names = list(None);
        assert!(names.contains(&name));
        assert!(show(&name, None).is_some());
        assert_eq!(show(&name, None).unwrap().member_ids.len(), 0);
        let _ = delete(&name, None);
    }

    #[test]
    #[serial_test::serial]
    fn create_duplicate_fails() {
        let name = format!(
            "dup_fleet_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        assert!(create(&name, None).is_ok());
        assert!(create(&name, None).is_err());
        let _ = delete(&name, None);
    }

    #[test]
    #[serial_test::serial]
    fn scale_requires_base() {
        let name = format!(
            "scale_no_base_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        assert!(create(&name, None).is_ok());
        assert!(scale(&name, 0, None).is_err());
        let _ = delete(&name, None);
    }

    // Mutation testing: catch count boundaries and delete return value (see docs/MUTATION_ANALYSIS.md).
    // Use a valid agent type (see mold / agent runtime); short names like `W` are rejected.
    const MINIMAL_MOLD: &str = r#"mold "m" "1.0"
agent
  type Worker
  role "r"
"#;

    #[test]
    #[serial_test::serial]
    fn create_from_mold_rejects_count_zero() {
        let base = tempfile::tempdir().unwrap();
        let err = create_from_mold("mf", MINIMAL_MOLD, 0, base.path(), None).unwrap_err();
        assert!(
            err.contains("count") && (err.contains("1") || err.contains(">= 1")),
            "expected count >= 1 error, got: {}",
            err
        );
    }

    #[test]
    #[serial_test::serial]
    fn create_from_mold_rejects_count_over_1000() {
        let base = tempfile::tempdir().unwrap();
        let err = create_from_mold("mf", MINIMAL_MOLD, 1001, base.path(), None).unwrap_err();
        assert!(
            err.contains("1000") || err.contains("capped"),
            "expected cap at 1000, got: {}",
            err
        );
    }

    #[test]
    #[serial_test::serial]
    fn add_from_mold_rejects_count_zero() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "add0_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = add_from_mold(&name, MINIMAL_MOLD, 0, base.path(), None).unwrap_err();
        assert!(
            err.contains("count") && (err.contains("1") || err.contains(">= 1")),
            "expected count >= 1 error, got: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn add_from_mold_rejects_count_over_1000() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "add1001_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = add_from_mold(&name, MINIMAL_MOLD, 1001, base.path(), None).unwrap_err();
        assert!(
            err.contains("1000") || err.contains("capped"),
            "expected cap at 1000, got: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn scale_rejects_count_over_1000() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "scale1001_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = scale(&name, 1001, Some(base.path())).unwrap_err();
        assert!(
            err.contains("1000") || err.contains("capped"),
            "expected scale cap at 1000, got: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn delete_returns_false_when_fleet_missing() {
        let base = tempfile::tempdir().unwrap();
        let ok = delete("nonexistent_fleet_xyz", Some(base.path())).unwrap();
        assert!(!ok, "delete nonexistent should return false");
    }

    #[test]
    #[serial_test::serial]
    fn delete_returns_true_when_fleet_removed() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "del_true_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let ok = delete(&name, Some(base.path())).unwrap();
        assert!(ok, "delete existing fleet should return true");
        assert!(show(&name, Some(base.path())).is_none());
    }

    /// Deploy + run dispatches `last_deployed_task` to each member via `agent::coordinate` (task_distribution).
    #[test]
    #[serial_test::serial]
    fn deploy_and_run_dispatches_to_all_members() {
        use std::fs;
        let base = tempfile::tempdir().unwrap();
        let mold_dal = r#"mold "fleet_run_test" "1.0"
agent
  type Worker
  role "run test worker"
"#;
        fs::write(base.path().join("frt.mold.dal"), mold_dal).unwrap();
        let name = format!(
            "fleet_run_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create_from_mold(&name, "frt", 2, base.path(), None).unwrap();
        deploy(&name, "integration task description", Some(base.path())).unwrap();
        let reports = run(Some(name.as_str()), base.path()).unwrap();
        assert_eq!(reports.len(), 1, "expected one RunReport: {:?}", reports);
        assert_eq!(reports[0].fleet_name, name);
        assert_eq!(
            reports[0].members_dispatched, 2,
            "coordinate should run for each member: {:?}",
            reports[0].errors
        );
        assert!(
            reports[0].errors.is_empty(),
            "unexpected errors: {:?}",
            reports[0].errors
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn create_rejects_empty_name() {
        let err = create("", None).unwrap_err();
        assert!(
            err.to_lowercase().contains("empty"),
            "expected empty name error, got: {}",
            err
        );
    }

    #[test]
    #[serial_test::serial]
    fn list_returns_sorted_names() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let b = format!("sort_b_{}", suffix);
        let a = format!("sort_a_{}", suffix);
        create(&b, None).unwrap();
        create(&a, None).unwrap();
        let names = list(None);
        let pos_a = names.iter().position(|n| n == &a).expect("a");
        let pos_b = names.iter().position(|n| n == &b).expect("b");
        assert!(
            pos_a < pos_b,
            "list should be sorted lexicographically: {:?}",
            names
        );
        let _ = delete(&a, None);
        let _ = delete(&b, None);
    }

    #[test]
    #[serial_test::serial]
    fn add_member_rejects_empty_agent_id() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "add_mem_empty_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = add_member(&name, "   ", Some(base.path())).unwrap_err();
        assert!(
            err.to_lowercase().contains("empty"),
            "expected empty agent_id: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn add_member_requires_base_path() {
        let name = format!(
            "add_base_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, None).unwrap();
        let err = add_member(&name, "agent-1", None).unwrap_err();
        assert!(
            err.contains("base path"),
            "expected base path error: {}",
            err
        );
        let _ = delete(&name, None);
    }

    #[test]
    #[serial_test::serial]
    fn add_member_idempotent_duplicate() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "add_dup_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        add_member(&name, "same-id", Some(base.path())).unwrap();
        add_member(&name, "same-id", Some(base.path())).unwrap();
        assert_eq!(show(&name, Some(base.path())).unwrap().member_ids, vec![
            "same-id".to_string()
        ]);
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn deploy_rejects_empty_task() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "dep_empty_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = deploy(&name, "   ", Some(base.path())).unwrap_err();
        assert!(
            err.to_lowercase().contains("empty"),
            "expected empty task: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn deploy_requires_base_path() {
        let name = format!(
            "dep_base_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, None).unwrap();
        let err = deploy(&name, "task", None).unwrap_err();
        assert!(
            err.contains("base path"),
            "expected base path error: {}",
            err
        );
        let _ = delete(&name, None);
    }

    #[test]
    #[serial_test::serial]
    fn health_not_found() {
        let err = health("no_such_fleet_ever", None).unwrap_err();
        assert!(err.contains("not found"), "{}", err);
    }

    #[test]
    #[serial_test::serial]
    fn health_empty_when_no_members_and_no_mold() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "health_empty_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let h = health(&name, Some(base.path())).unwrap();
        assert_eq!(h.status, "empty");
        assert_eq!(h.member_count, 0);
        assert!(!h.has_mold);
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn scale_noop_when_n_equals_current() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("noop.mold.dal"), MINIMAL_MOLD).unwrap();
        let name = format!(
            "scale_noop_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create_from_mold(&name, "noop", 2, base.path(), None).unwrap();
        let before = show(&name, Some(base.path())).unwrap().member_ids.len();
        scale(&name, before as u32, Some(base.path())).unwrap();
        assert_eq!(
            show(&name, Some(base.path())).unwrap().member_ids.len(),
            before
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn scale_down_truncates_members() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("trunc.mold.dal"), MINIMAL_MOLD).unwrap();
        let name = format!(
            "scale_trunc_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create_from_mold(&name, "trunc", 4, base.path(), None).unwrap();
        scale(&name, 1, Some(base.path())).unwrap();
        assert_eq!(show(&name, Some(base.path())).unwrap().member_ids.len(), 1);
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn scale_up_fails_when_fleet_has_no_mold() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "scale_nomold_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let err = scale(&name, 1, Some(base.path())).unwrap_err();
        assert!(
            err.contains("no mold") || err.contains("mold"),
            "expected no-mold scale-up error: {}",
            err
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn export_errors_when_fleet_missing() {
        let err = export(
            Some("missing_fleet_xyz"),
            None,
            ExportFormat::K8s,
        )
        .unwrap_err();
        assert!(err.contains("not found"), "{}", err);
    }

    #[test]
    #[serial_test::serial]
    fn export_k8s_and_compose_contain_agent_env() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("ex.mold.dal"), MINIMAL_MOLD).unwrap();
        let name = format!(
            "ex_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create_from_mold(&name, "ex", 1, base.path(), None).unwrap();
        let k8s = export(Some(name.as_str()), Some(base.path()), ExportFormat::K8s).unwrap();
        assert!(k8s.contains("DAL_AGENT_ID") && k8s.contains("kind: Job"));
        let dc = export(
            Some(name.as_str()),
            Some(base.path()),
            ExportFormat::DockerCompose,
        )
        .unwrap();
        assert!(dc.contains("DAL_AGENT_ID") && dc.contains("services:"));
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn run_skips_fleets_without_deploy_task() {
        let base = tempfile::tempdir().unwrap();
        let name = format!(
            "run_skip_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        create(&name, Some(base.path())).unwrap();
        let reports = run(None, base.path()).unwrap();
        assert!(
            reports.is_empty(),
            "no last_deployed_task => no reports: {:?}",
            reports
        );
        let _ = delete(&name, Some(base.path()));
    }

    #[test]
    #[serial_test::serial]
    fn add_from_mold_fails_when_fleet_missing() {
        let base = tempfile::tempdir().unwrap();
        let err = add_from_mold("ghost_fleet", MINIMAL_MOLD, 1, base.path(), None).unwrap_err();
        assert!(err.contains("not found"), "{}", err);
    }
}
