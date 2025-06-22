use deno_core::{
    error::ModuleLoaderError, resolve_import, resolve_path, ModuleLoadResponse, ModuleLoader,
    ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType, RequestedModuleType,
    ResolutionKind,
};
use deno_error::JsErrorBox;
use futures::FutureExt;
use std::path::PathBuf;

pub struct AppModuleLoader {
    pub module_root: PathBuf,
}

impl AppModuleLoader {
    async fn load_file_module(
        module_specifier: ModuleSpecifier,
        requested_module_type: RequestedModuleType,
        path: PathBuf,
    ) -> Result<ModuleSource, ModuleLoaderError> {
        let module_type = if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            // We only return JSON modules if extension was actually `.json`.
            // In other cases we defer to actual requested module type, so runtime
            // can decide what to do with it.
            if ext == "json" {
                ModuleType::Json
            } else if ext == "wasm" {
                ModuleType::Wasm
            } else {
                match &requested_module_type {
                    RequestedModuleType::Other(ty) => ModuleType::Other(ty.clone()),
                    _ => ModuleType::JavaScript,
                }
            }
        } else {
            ModuleType::JavaScript
        };

        // If we loaded a JSON file, but the "requested_module_type" (that is computed from
        // import attributes) is not JSON we need to fail.
        if module_type == ModuleType::Json && requested_module_type != RequestedModuleType::Json {
            return Err(ModuleLoaderError::JsonMissingAttribute);
        }

        let code = tokio::fs::read(path).await?;
        let module = ModuleSource::new(
            module_type,
            ModuleSourceCode::Bytes(code.into_boxed_slice().into()),
            &module_specifier,
            None,
        );
        Ok(module)
    }
}

impl ModuleLoader for AppModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        if specifier.starts_with("../") || specifier.starts_with("./") {
            return resolve_path(specifier, &self.module_root)
                .map_err(|_| ModuleLoaderError::NotFound);
        }

        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dynamic: bool,
        requested_module_type: RequestedModuleType,
    ) -> ModuleLoadResponse {
        let module_specifier = module_specifier.clone();

        match module_specifier.to_file_path() {
            // File import
            Ok(path) => ModuleLoadResponse::Async(
                Self::load_file_module(module_specifier, requested_module_type, path).boxed_local(),
            ),

            // Non file imports currently unsupported
            Err(_) => ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
                "Provided module specifier \"{module_specifier}\" is not a file URL."
            ))
            .into())),
        }
    }
}
