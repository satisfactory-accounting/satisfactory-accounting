use thiserror::Error;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::StorageManager;

/// Errors from checking the state of the storage manager.
#[derive(Debug, Error)]
pub enum StorageManagerError {
    /// An underlying error from JS.
    #[error("Error from JS: {0:?}")]
    JsError(JsValue),
    /// The window was missing.
    #[error("Window was None")]
    NoWindow,
    /// The navigator was missing.
    #[error("window.navigator was null or undefined")]
    NavigatorMissing,
    /// The storage manager was missing.
    #[error("window.navigator.storage was null or undefined")]
    StorageManagerMissing,
}

impl From<JsValue> for StorageManagerError {
    fn from(value: JsValue) -> Self {
        StorageManagerError::JsError(value)
    }
}

/// Gets the StorageManager.
fn storage_manager() -> Result<StorageManager, StorageManagerError> {
    let navigator = web_sys::window()
        .ok_or(StorageManagerError::NoWindow)?
        .navigator();
    if navigator.is_null() || navigator.is_undefined() {
        return Err(StorageManagerError::NavigatorMissing);
    }
    let storage_manager = navigator.storage();
    if storage_manager.is_null() || storage_manager.is_undefined() {
        return Err(StorageManagerError::StorageManagerMissing);
    }
    Ok(storage_manager)
}

/// Requests to persist local storage.
pub async fn persist_local_storage() -> Result<(), StorageManagerError> {
    let future: JsFuture = storage_manager()?.persist()?.into();
    let _ = future.await?;
    Ok(())
}
