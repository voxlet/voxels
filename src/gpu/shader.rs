pub mod load;
mod watch;

use std::{
    borrow,
    collections::{HashMap, HashSet},
    path,
};

struct ValidationError {}

fn validate(full_path: &path::Path) -> Result<(), ValidationError> {
    let load::Source { code, .. } = load::load(full_path);

    let mut validator = naga::valid::Validator::new(naga::valid::ValidationFlags::all());
    match naga::front::wgsl::parse_str(&code) {
        Err(e) => {
            e.emit_to_stderr();
            Err(ValidationError {})
        }
        Ok(module) => {
            if let Err(e) = validator.validate(&module) {
                eprintln!("{}", e);
                match e {
                    naga::valid::ValidationError::Function { error, .. } => {
                        eprintln!("{}", error);
                    }
                    _ => {}
                };
                return Err(ValidationError {});
            };
            Ok(())
        }
    }
}

pub struct Shaders {
    base_path: String,
    included_paths: HashMap<String, HashSet<path::PathBuf>>,
    watchers: HashMap<String, notify::RecommendedWatcher>,
}

impl Shaders {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: String::from(base_path),
            included_paths: HashMap::new(),
            watchers: HashMap::new(),
        }
    }

    fn full_path(&self, path: &str) -> path::PathBuf {
        path::Path::new(&self.base_path).join(path)
    }

    pub fn source(&mut self, path: &str) -> wgpu::ShaderSource {
        let full_path = self.full_path(path);
        let load::Source {
            code,
            mut included_paths,
        } = load::load(&full_path);
        included_paths.insert(full_path);
        self.included_paths.insert(path.to_string(), included_paths);
        wgpu::ShaderSource::Wgsl(borrow::Cow::Owned(code))
    }

    pub fn watch_source<F>(&mut self, path: &str, mut on_update: F)
    where
        F: 'static + FnMut() + Send,
    {
        tracing::info!(path, "watching");
        if let Some(included_paths) = self.included_paths.get(path) {
            let full_path = self.full_path(path);
            let watcher = watch::watch(&included_paths, move || {
                if let Ok(_) = validate(&full_path) {
                    on_update()
                }
            });
            self.watchers.insert(path.to_string(), watcher);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::load::tests::EXPECTED_MAIN_CODE;
    use std::borrow;

    #[test]
    fn source() {
        let mut shaders = super::Shaders::new("test/shaders");
        let source = shaders.source("main.wgsl");
        if let wgpu::ShaderSource::Wgsl(borrow::Cow::Owned(code)) = source {
            assert_eq!(code, EXPECTED_MAIN_CODE);
        } else {
            panic!("failed to load wgsl source")
        }
    }
}
