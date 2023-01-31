use std::{collections::HashSet, fs, path};

use lazy_static::lazy_static;

fn include_pattern() -> &'static regex::Regex {
    lazy_static! {
        static ref INCLUDE: regex::Regex =
            regex::Regex::new(r#"(?m)^#include "(\S+)"(?-m)\n"#).unwrap();
    }
    &INCLUDE
}

fn safe_join(left: &path::Path, right: &path::Path) -> path::PathBuf {
    let mut path = path::PathBuf::from(left);
    for c in right.components().into_iter() {
        match c {
            path::Component::Normal(name) => {
                path.push(name);
            }
            path::Component::ParentDir => {
                path.pop();
            }
            _ => {}
        }
    }
    path
}

fn expand_inclusions(path: &path::Path, included_paths: &mut HashSet<path::PathBuf>) -> String {
    let source = fs::read_to_string(path).unwrap();
    include_pattern()
        .replace_all(&source, |caps: &regex::Captures| {
            let dir = path.parent().unwrap();
            let included_path = safe_join(dir, &path::PathBuf::from(&caps[1]))
                .canonicalize()
                .unwrap();

            if included_paths.contains(&included_path) {
                return "".to_string();
            }
            included_paths.insert(included_path.clone());

            let included_source = expand_inclusions(&included_path, included_paths);
            included_source
        })
        .to_string()
}

pub struct Source {
    pub code: String,
    pub included_paths: HashSet<path::PathBuf>,
}

pub fn load(path: &path::Path) -> Source {
    let mut included_paths = HashSet::new();
    let code = expand_inclusions(path, &mut included_paths);
    Source {
        code,
        included_paths,
    }
}

#[cfg(test)]
pub mod tests {
    use std::path;

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
                |caps: &regex::Captures| format!("{} code\n", &caps[1])
            )
        );
    }

    pub const EXPECTED_MAIN_CODE: &str = r#"inc8 code

inc6 code

inc5 code

inc4 code

inc3 code

inc1 code

inc2 code

inc7 code

main code
"#;

    #[test]
    fn load() {
        let super::Source { code, .. } = super::load(&path::Path::new("test/shaders/main.wgsl"));
        assert_eq!(code, EXPECTED_MAIN_CODE);
    }
}
