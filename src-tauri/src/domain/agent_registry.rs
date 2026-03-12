use serde::Serialize;

#[derive(Debug, Clone)]
pub struct AgentPathClaim {
    pub agent_label: String,
    pub path: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapability {
    pub id: String,
    pub label: String,
    pub global_paths: Vec<String>,
    pub project_paths: Vec<String>,
    pub default_global_mode: String,
    pub default_project_mode: String,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Vec<AgentCapability>,
    global_path_claims: Vec<AgentPathClaim>,
    project_path_claims: Vec<AgentPathClaim>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let agents = vec![
            AgentCapability {
                id: "claude-code".into(),
                label: "Claude Code".into(),
                global_paths: vec![".claude/skills".into()],
                project_paths: vec![".claude/skills".into()],
                default_global_mode: "symlink".into(),
                default_project_mode: "copy".into(),
            },
            AgentCapability {
                id: "codex".into(),
                label: "OpenAI Codex".into(),
                global_paths: vec![".agents/skills".into(), ".codex/skills".into()],
                project_paths: vec![".agents/skills".into(), ".codex/skills".into()],
                default_global_mode: "symlink".into(),
                default_project_mode: "copy".into(),
            },
            AgentCapability {
                id: "cursor".into(),
                label: "Cursor".into(),
                global_paths: vec![".cursor/skills".into()],
                project_paths: vec![".cursor/skills".into(), ".agents/skills".into()],
                default_global_mode: "symlink".into(),
                default_project_mode: "copy".into(),
            },
            AgentCapability {
                id: "github-copilot".into(),
                label: "GitHub Copilot / VS Code".into(),
                global_paths: vec![".copilot/skills".into(), ".agents/skills".into()],
                project_paths: vec![".github/skills".into(), ".agents/skills".into()],
                default_global_mode: "symlink".into(),
                default_project_mode: "copy".into(),
            },
        ];

        Self {
            global_path_claims: vec![
                AgentPathClaim {
                    agent_label: "Claude Code".into(),
                    path: ".claude/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".codex/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".agents/skills".into(),
                    priority: 90,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".cursor/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".copilot/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".agents/skills".into(),
                    priority: 80,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".agents/skills".into(),
                    priority: 70,
                },
            ],
            project_path_claims: vec![
                AgentPathClaim {
                    agent_label: "Claude Code".into(),
                    path: ".claude/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".codex/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "OpenAI Codex".into(),
                    path: ".agents/skills".into(),
                    priority: 90,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".cursor/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "Cursor".into(),
                    path: ".agents/skills".into(),
                    priority: 80,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".github/skills".into(),
                    priority: 100,
                },
                AgentPathClaim {
                    agent_label: "GitHub Copilot / VS Code".into(),
                    path: ".agents/skills".into(),
                    priority: 70,
                },
            ],
            agents,
        }
    }

    pub fn agents(&self) -> &[AgentCapability] {
        &self.agents
    }

    pub fn preferred_global_path_for(&self, label: &str) -> Option<&str> {
        self.global_path_claims
            .iter()
            .filter(|claim| claim.agent_label == label)
            .max_by_key(|claim| claim.priority)
            .map(|claim| claim.path.as_str())
    }

    pub fn preferred_project_path_for(&self, label: &str) -> Option<&str> {
        self.project_path_claims
            .iter()
            .filter(|claim| claim.agent_label == label)
            .max_by_key(|claim| claim.priority)
            .map(|claim| claim.path.as_str())
    }
}
