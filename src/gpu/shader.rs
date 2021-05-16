use std::{
    borrow::Cow,
    collections::HashSet,
    fs::read_to_string,
    path::{Component, Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::{Captures, Regex};

pub struct Shaders {
    base_path: String,
}

fn include_pattern() -> &'static Regex {
    lazy_static! {
        static ref INCLUDE: Regex = Regex::new(r#"(?m)^#include "(\S+)"(?-m)\n"#).unwrap();
    }
    &INCLUDE
}

fn safe_join(left: &Path, right: &Path) -> PathBuf {
    let mut path = PathBuf::from(left);
    for c in right.components().into_iter() {
        match c {
            Component::Normal(name) => {
                path.push(name);
            }
            Component::ParentDir => {
                path.pop();
            }
            _ => {}
        }
    }
    path
}

fn load(path: &Path, included_paths: &mut HashSet<PathBuf>) -> String {
    let source = read_to_string(path).unwrap();
    include_pattern()
        .replace_all(&source, |caps: &Captures| {
            let dir = path.parent().unwrap();
            let included_path = safe_join(dir, &PathBuf::from(&caps[1]))
                .canonicalize()
                .unwrap();

            if included_paths.contains(&included_path) {
                return "".to_string();
            }
            included_paths.insert(included_path.clone());

            let included_source = load(&included_path, included_paths);
            included_source
        })
        .to_string()
}

impl Shaders {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: String::from(base_path),
        }
    }

    pub fn source(&self, path: &str) -> wgpu::ShaderSource {
        let full_path = Path::new(&self.base_path).join(path);
        wgpu::ShaderSource::Wgsl(Cow::Owned(load(&full_path, &mut HashSet::new())))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use regex::Captures;

    #[test]
    fn include_pattern() {
        assert_eq!(
            r#"included-1 code
included-2 code
included-3 code
some code"#,
            super::include_pattern().replace_all(
                r#"#include "included-1"
#include "included-2"
#include "included-3"
some code"#,
                |caps: &Captures| format!("{} code\n", &caps[1])
            )
        );
    }

    #[test]
    fn load() {
        let shaders = super::Shaders::new("test/shaders");
        let source = shaders.source("main.wgsl");
        if let wgpu::ShaderSource::Wgsl(Cow::Owned(code)) = source {
            assert_eq!(
                code,
                r#"inc8 code

inc6 code

inc5 code

inc4 code

inc3 code

inc1 code

inc2 code

inc7 code

main code
"#
            );
        } else {
            panic!("failed to load wgsl source")
        }
    }
}
