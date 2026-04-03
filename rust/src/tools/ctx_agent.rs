use crate::core::agents::{AgentRegistry, AgentStatus};

#[allow(clippy::too_many_arguments)]
pub fn handle(
    action: &str,
    agent_type: Option<&str>,
    role: Option<&str>,
    project_root: &str,
    current_agent_id: Option<&str>,
    message: Option<&str>,
    category: Option<&str>,
    to_agent: Option<&str>,
    status: Option<&str>,
) -> String {
    match action {
        "register" => {
            let atype = agent_type.unwrap_or("unknown");
            let mut registry = AgentRegistry::load_or_create();
            registry.cleanup_stale(24);
            let agent_id = registry.register(atype, role, project_root);
            match registry.save() {
                Ok(()) => format!(
                    "Agent registered: {agent_id} (type: {atype}, role: {})",
                    role.unwrap_or("none")
                ),
                Err(e) => format!("Registered as {agent_id} but save failed: {e}"),
            }
        }

        "list" => {
            let mut registry = AgentRegistry::load_or_create();
            registry.cleanup_stale(24);
            let _ = registry.save();

            let agents = registry.list_active(Some(project_root));
            if agents.is_empty() {
                return "No active agents for this project.".to_string();
            }

            let mut out = format!("Active agents ({}):\n", agents.len());
            for a in agents {
                let role_str = a.role.as_deref().unwrap_or("-");
                let status_msg = a
                    .status_message
                    .as_deref()
                    .map(|m| format!(" — {m}"))
                    .unwrap_or_default();
                let age = (chrono::Utc::now() - a.last_active).num_minutes();
                out.push_str(&format!(
                    "  {} [{}] role={} status={}{} (last active: {}m ago, pid: {})\n",
                    a.agent_id, a.agent_type, role_str, a.status, status_msg, age, a.pid
                ));
            }
            out
        }

        "post" => {
            let msg = match message {
                Some(m) => m,
                None => return "Error: message is required for post".to_string(),
            };
            let cat = category.unwrap_or("status");
            let from = current_agent_id.unwrap_or("anonymous");
            let mut registry = AgentRegistry::load_or_create();
            let msg_id = registry.post_message(from, to_agent, cat, msg);
            match registry.save() {
                Ok(()) => {
                    let target = to_agent.unwrap_or("all agents (broadcast)");
                    format!("Posted [{cat}] to {target}: {msg} (id: {msg_id})")
                }
                Err(e) => format!("Posted but save failed: {e}"),
            }
        }

        "read" => {
            let agent_id = match current_agent_id {
                Some(id) => id,
                None => {
                    return "Error: agent must be registered first (use action=register)"
                        .to_string()
                }
            };
            let mut registry = AgentRegistry::load_or_create();
            let messages = registry.read_unread(agent_id);

            if messages.is_empty() {
                let _ = registry.save();
                return "No new messages.".to_string();
            }

            let mut out = format!("New messages ({}):\n", messages.len());
            for m in &messages {
                let age = (chrono::Utc::now() - m.timestamp).num_minutes();
                out.push_str(&format!(
                    "  [{}] from {} ({}m ago): {}\n",
                    m.category, m.from_agent, age, m.message
                ));
            }
            let _ = registry.save();
            out
        }

        "status" => {
            let agent_id = match current_agent_id {
                Some(id) => id,
                None => return "Error: agent must be registered first".to_string(),
            };
            let new_status = match status {
                Some("active") => AgentStatus::Active,
                Some("idle") => AgentStatus::Idle,
                Some("finished") => AgentStatus::Finished,
                Some(other) => {
                    return format!("Unknown status: {other}. Use: active, idle, finished")
                }
                None => return "Error: status value is required".to_string(),
            };
            let status_msg = message;

            let mut registry = AgentRegistry::load_or_create();
            registry.set_status(agent_id, new_status.clone(), status_msg);
            match registry.save() {
                Ok(()) => format!(
                    "Status updated: {} → {}{}",
                    agent_id,
                    new_status,
                    status_msg.map(|m| format!(" ({m})")).unwrap_or_default()
                ),
                Err(e) => format!("Status set but save failed: {e}"),
            }
        }

        "info" => {
            let registry = AgentRegistry::load_or_create();
            let total = registry.agents.len();
            let active = registry
                .agents
                .iter()
                .filter(|a| a.status == AgentStatus::Active)
                .count();
            let messages = registry.scratchpad.len();
            format!(
                "Agent Registry: {total} total, {active} active, {messages} scratchpad entries\nLast updated: {}",
                registry.updated_at.format("%Y-%m-%d %H:%M UTC")
            )
        }

        _ => format!("Unknown action: {action}. Use: register, list, post, read, status, info"),
    }
}
