use chrisomatic_core::{Counts, StepEffect, fully_exec_tree, plan};
use chrisomatic_spec::*;
use wasm_bindgen::prelude::*;

/// Validate `text` as a TOML-formatted _chrisomatic_ manifest, and return the
/// estimated number of "steps" it will take to execute [run_chrisomatic].
#[wasm_bindgen]
pub fn validate_manifest(
    text: &str,
    url: &str,
    username: &str,
    token: &str,
) -> Result<usize, String> {
    let manifest = canonicalize_manifest(text, url, username, token).map_err(|e| e.to_string())?;
    let tree = plan(manifest);
    Ok(tree.count())
}

/// Apply the changes specified by `text`, which is expected to be a
/// TOML-formatted _chrisomatic_ manifest (hint: validate it with
/// [validate_manifest]).
///
/// Returns list of error messages describing failed steps.
#[wasm_bindgen]
pub async fn run_chrisomatic(
    text: &str,
    url: &str,
    username: &str,
    token: &str,
    on_progress: &js_sys::Function,
) -> Result<Vec<String>, String> {
    let manifest = canonicalize_manifest(text, url, username, token).map_err(|e| e.to_string())?;
    let tree = plan(manifest);
    let client = reqwest::Client::new();
    let affected = fully_exec_tree(client, tree, |counts| {
        let this = JsValue::null();
        let _ = on_progress.call1(&this, &counts_to_object(counts));
    })
    .await;
    let error_messages = affected
        .into_iter()
        .filter_map(|(target, effect)| {
            if matches!(&effect, StepEffect::Unfulfilled(..) | StepEffect::Error(_)) {
                Some(format!("{target:?} not created because {effect:?}"))
            } else {
                None
            }
        })
        .collect();
    Ok(error_messages)
}

#[allow(unused_must_use)]
fn counts_to_object(counts: Counts) -> js_sys::Object {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"created".into(), &counts.created.into());
    js_sys::Reflect::set(&obj, &"modified".into(), &counts.modified.into());
    js_sys::Reflect::set(&obj, &"unmodified".into(), &counts.unmodified.into());
    js_sys::Reflect::set(&obj, &"unfulfilled".into(), &counts.unfulfilled.into());
    js_sys::Reflect::set(&obj, &"error".into(), &counts.error.into());
    obj
}

// #[wasm_bindgen]
// pub struct Counts {
//     pub unmodified: u32,
// }

// impl From<chrisomatic_core::Counts> for Counts {
//     fn from(value: chrisomatic_core::Counts) -> Self {
//         Self {
//             unmodified: value.unmodified,
//         }
//     }
// }

fn canonicalize_manifest(
    text: &str,
    url: &str,
    username: &str,
    token: &str,
) -> Result<Manifest, Box<dyn std::error::Error>> {
    let mut given: GivenManifest = toml::from_str(text)?;
    if given.global.cube.is_none() {
        given.global.cube = Some(CubeUrl::try_new(url)?);
    }
    if given.global.admin.is_none() {
        given.global.admin = Some(UserCredentials {
            username: Username::new(username.into()),
            secret: PasswordOrToken::Token(token.to_string()),
        })
    }
    Ok(given.try_into()?)
}
