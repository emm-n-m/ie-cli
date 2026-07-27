use ie_formats::{AreaScalarPatch, CreatureScalarPatch};
use std::fs;
use std::path::Path;

pub(crate) fn collect_are_patches(
    sets: &[String],
    patch_json: Option<&Path>,
) -> Result<Vec<AreaScalarPatch>, Box<dyn std::error::Error>> {
    let mut patches = sets
        .iter()
        .map(|set| {
            let (field, value) = set
                .split_once('=')
                .ok_or_else(|| format!("invalid --set value '{set}', expected field=value"))?;
            Ok(AreaScalarPatch {
                field: field.to_string(),
                value: value.to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    if let Some(path) = patch_json {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        match &value {
            serde_json::Value::Object(fields) => {
                patches.extend(fields.iter().map(|(field, value)| AreaScalarPatch {
                    field: field.clone(),
                    value: scalar_json_value_to_string(value),
                }));
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    let field = row
                        .get("field")
                        .and_then(serde_json::Value::as_str)
                        .ok_or("ARE patch array entries must include string field")?;
                    let value = row
                        .get("value")
                        .ok_or("ARE patch array entries must include value")?;
                    patches.push(AreaScalarPatch {
                        field: field.to_string(),
                        value: scalar_json_value_to_string(value),
                    });
                }
            }
            _ => return Err("ARE patch JSON must be an object or array".into()),
        }
    }

    Ok(patches)
}

pub(crate) fn collect_cre_patches(
    sets: &[String],
    patch_json: Option<&Path>,
) -> Result<Vec<CreatureScalarPatch>, Box<dyn std::error::Error>> {
    let mut patches = sets
        .iter()
        .map(|set| {
            let (field, value) = set
                .split_once('=')
                .ok_or_else(|| format!("invalid --set value '{set}', expected field=value"))?;
            Ok(CreatureScalarPatch {
                field: field.to_string(),
                value: value.to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    if let Some(path) = patch_json {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        patches.extend(parse_cre_patch_json(&value)?);
    }

    Ok(patches)
}

fn parse_cre_patch_json(
    value: &serde_json::Value,
) -> Result<Vec<CreatureScalarPatch>, Box<dyn std::error::Error>> {
    match value {
        serde_json::Value::Object(fields) => Ok(fields
            .iter()
            .map(|(field, value)| CreatureScalarPatch {
                field: field.clone(),
                value: scalar_json_value_to_string(value),
            })
            .collect()),
        serde_json::Value::Array(rows) => rows
            .iter()
            .map(|row| {
                let field = row
                    .get("field")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("patch array entries must include string field")?;
                let value = row
                    .get("value")
                    .ok_or("patch array entries must include value")?;
                Ok(CreatureScalarPatch {
                    field: field.to_string(),
                    value: scalar_json_value_to_string(value),
                })
            })
            .collect(),
        _ => Err("patch JSON must be an object or array".into()),
    }
}

fn scalar_json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
