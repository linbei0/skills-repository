use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    domain::types::{SaveTemplateRequest, TemplateRecord},
    repositories::templates as templates_repository,
};

fn validate_template(request: &SaveTemplateRequest) -> Result<()> {
    if request.name.trim().is_empty() {
        return Err(anyhow!("template name is required"));
    }

    if request.scope != "system" && request.scope != "user" {
        return Err(anyhow!("unsupported template scope {}", request.scope));
    }

    for item in &request.items {
        if item.skill_ref.trim().is_empty() {
            return Err(anyhow!("template item skill_ref is required"));
        }
    }

    Ok(())
}

pub fn list_templates(path: &Path) -> Result<Vec<TemplateRecord>> {
    templates_repository::list_templates(path)
}

pub fn get_template(path: &Path, template_id: &str) -> Result<Option<TemplateRecord>> {
    templates_repository::get_template(path, template_id)
}

pub fn save_template(path: &Path, request: &SaveTemplateRequest) -> Result<TemplateRecord> {
    validate_template(request)?;
    templates_repository::save_template(path, request)
}

pub fn delete_template(path: &Path, template_id: &str) -> Result<()> {
    templates_repository::delete_template(path, template_id)
}
