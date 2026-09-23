//! Validated design directions from the chat template picker.
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum DesignChoice {
    Template { template_id: String },
    Custom {},
}

impl DesignChoice {
    pub(crate) fn append_to(self, text: &mut String) -> Result<()> {
        anyhow::ensure!(!text.trim().is_empty(), "Describe what you want to create");
        anyhow::ensure!(text.len() <= 32_000, "Description is too long");
        match self {
            Self::Template { template_id } => {
                let template = fenix_agent::designs::template(&template_id)?;
                let label = format!(
                    "{} / {}",
                    template["title_es"].as_str().unwrap_or_default(),
                    template["title"].as_str().unwrap_or_default()
                ).replace(['[', ']', '\n', '\r'], " ");
                text.push_str(&format!(
                    "\n\nPlantilla elegida / Selected template: [{label}](/design?template={}).\n\
                     Adapta el contenido a mi solicitud conservando esta composición. / \
                     Adapt the content to my request while preserving this composition.\n",
                    template["id"].as_str().unwrap_or_default(),
                ));
            }
            Self::Custom {} => text.push_str(
                "\n\nDiseño propio / Custom design: crea directamente a partir de mi descripción, \
                 sin plantilla. / Create directly from my description, without a catalog template.\n",
            ),
        }
        anyhow::ensure!(
            text.len() <= 32_000,
            "Shorten the description to leave room for the design direction"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_preserves_the_brief_and_resolves_only_catalog_templates() {
        let mut text = "Create five slides about renewable energy for Project Atlas.".to_owned();
        DesignChoice::Template {
            template_id: "research-deck".into(),
        }
        .append_to(&mut text)
        .unwrap();
        assert!(text.starts_with("Create five slides about renewable energy for Project Atlas."));
        assert!(text.contains("/design?template=research-deck"));
        assert!(text.contains("preserving this composition"));
        assert!(!text.contains("design_templates"));
        let mut original = "Build an interface".to_owned();
        assert!(
            DesignChoice::Template {
                template_id: "missing\ninjected".into()
            }
            .append_to(&mut original)
            .is_err()
        );
        assert_eq!(original, "Build an interface");
        DesignChoice::Custom {}.append_to(&mut original).unwrap();
        assert!(original.contains("without a catalog template"));
        assert!(!original.contains("template_id:"));
        assert!(
            DesignChoice::Custom {}
                .append_to(&mut String::new())
                .is_err()
        );
        assert!(
            serde_json::from_str::<DesignChoice>(
                r#"{"mode":"custom","template_id":"research-deck"}"#
            )
            .is_err()
        );
    }
}
