use js_sys::{Uint8Array, JSON};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::{wasm_bindgen, JsCast, JsValue};

use anyhow::{bail, Result};
use wasm_bindgen_futures::JsFuture;

use super::AssetReader;
use web_sys::Response;

/// Represents the global object in the JavaScript context
#[wasm_bindgen]
extern "C" {
    /// The [Global](https://developer.mozilla.org/en-US/docs/Glossary/Global_object) object.
    type Global;

    /// The [window](https://developer.mozilla.org/en-US/docs/Web/API/Window) global object.
    #[wasm_bindgen(method, getter, js_name = Window)]
    fn window(this: &Global) -> JsValue;

    /// The [WorkerGlobalScope](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope) global object.
    #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
    fn worker(this: &Global) -> JsValue;
}

fn js_value_to_err(context: &str) -> impl FnOnce(JsValue) -> std::io::Error + '_ {
    move |value| {
        let message = match JSON::stringify(&value) {
            Ok(js_str) => format!("Failed to {context}: {js_str}"),
            Err(_) => {
                format!("Failed to {context} and also failed to stringify the JSValue of the error")
            }
        };

        std::io::Error::new(std::io::ErrorKind::Other, message)
    }
}

/// Simplified web asset reader <https://github.com/bevyengine/bevy/blob/main/crates/bevy_asset/src/io/wasm.rs#L27>
pub(crate) struct WebAssetReader {
    root_path: PathBuf,
}

impl AssetReader for WebAssetReader {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            root_path: path.as_ref().to_owned(),
        }
    }

    fn get_full_path(&self, path: &str) -> PathBuf {
        self.root_path.join(path)
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<Vec<u8>> {
        let fullpath = self.get_full_path(path);
        let path = fullpath.to_str().unwrap();
        // The JS global scope includes a self-reference via a specialising name, which can be used to determine the type of global context available.
        let global: Global = js_sys::global().unchecked_into();
        let promise = if !global.window().is_undefined() {
            let window: web_sys::Window = global.unchecked_into();
            window.fetch_with_str(path)
        } else if !global.worker().is_undefined() {
            let worker: web_sys::WorkerGlobalScope = global.unchecked_into();
            worker.fetch_with_str(path)
        } else {
            let error = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported JavaScript global context",
            );
            bail!(error);
        };
        let resp_value = JsFuture::from(promise)
            .await
            .map_err(js_value_to_err("fetch path"))?;
        let resp = resp_value
            .dyn_into::<Response>()
            .map_err(js_value_to_err("convert fetch to Response"))?;
        match resp.status() {
            200 => {
                let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
                let bytes = Uint8Array::new(&data).to_vec();
                Ok(bytes)
            }
            404 => bail!("Not Found {path}"),
            status => bail!("HTTP Error {status}"),
        }
    }
}
