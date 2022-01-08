use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToUpperCamelCase,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Case {
    #[serde(alias = "UpperCase")]
    #[serde(alias = "UPPER")]
    #[serde(alias = "UPPERCASE")]
    #[serde(rename(serialize = "UPPERCASE"))]
    Upper,

    #[serde(alias = "LowerCase")]
    #[serde(alias = "lower")]
    #[serde(alias = "lowercase")]
    #[serde(rename(serialize = "lowercase"))]
    Lower,

    #[serde(alias = "LowerSnakeCase")]
    #[serde(alias = "lower_snake")]
    #[serde(alias = "lower_snake_case")]
    #[serde(rename(serialize = "lower_snake_case"))]
    LowerSnake,

    #[serde(alias = "ScreamingSnake")]
    #[serde(alias = "ScreamingSnakeCase")]
    #[serde(alias = "SCREAMING_SNAKE")]
    #[serde(alias = "SCREAMING_SNAKE_CASE")]
    #[serde(alias = "UpperSnakeCase")]
    #[serde(alias = "UPPER_SNAKE")]
    #[serde(alias = "UPPER_SNAKE_CASE")]
    #[serde(rename(serialize = "UPPER_SNAKE_CASE"))]
    UpperSnake,

    #[serde(alias = "LowerKebabCase")]
    #[serde(alias = "lower-kebab")]
    #[serde(alias = "lower-kebab-case")]
    #[serde(rename(serialize = "lower-kebab-case"))]
    LowerKebab,

    #[serde(alias = "ScreamingKebab")]
    #[serde(alias = "ScreamingKebabCase")]
    #[serde(alias = "SCREAMING_KEBAB")]
    #[serde(alias = "SCREAMING_KEBAB_CASE")]
    #[serde(alias = "UpperKebabCase")]
    #[serde(alias = "UPPER-KEBAB")]
    #[serde(alias = "UPPER-KEBAB-CASE")]
    #[serde(rename(serialize = "UPPER-KEBAB-CASE"))]
    UpperKebab,

    #[serde(alias = "LowerCamelCase")]
    #[serde(alias = "lowerCamel")]
    #[serde(alias = "lowerCamelCase")]
    #[serde(rename(serialize = "lowerCamelCase"))]
    LowerCamel,

    #[serde(alias = "Pascal")]
    #[serde(alias = "PascalCase")]
    #[serde(alias = "UpperCamelCase")]
    #[serde(rename(serialize = "UpperCamelCase"))]
    UpperCamel,
}

impl Case {
    pub fn rename(&self, str: &str) -> String {
        match *self {
            Case::Upper => str.to_ascii_uppercase(),
            Case::Lower => str.to_ascii_lowercase(),
            Case::LowerSnake => str.to_snake_case(),
            Case::UpperSnake => str.to_shouty_snake_case(),
            Case::LowerKebab => str.to_kebab_case(),
            Case::UpperKebab => str.to_shouty_kebab_case(),
            Case::LowerCamel => str.to_lower_camel_case(),
            Case::UpperCamel => str.to_upper_camel_case(),
        }
    }

    pub fn rename_file_name(&self, path: &Path) -> PathBuf {
        let file_name = match path.file_stem() {
            None => return path.to_path_buf(),
            Some(file_name) => match file_name.to_str() {
                None => return path.to_path_buf(),
                Some(file_name) => file_name,
            },
        };
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or(PathBuf::new())
            .join(self.rename(file_name))
            .with_extension(path.extension().unwrap_or("".as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::case::Case;

    #[test]
    fn upper() {
        run_test(
            Case::Upper,
            &[
                "UPPERCASE",
                "LOWERCASE",
                "LOWER_SNAKE_CASE",
                "UPPER_SNAKE_CASE",
                "LOWER-KEBAB-CASE",
                "UPPER-KEBAB-CASE",
                "LOWERCAMELCASE",
                "UPPERCAMELCASE",
            ],
        );
    }

    #[test]
    fn lower() {
        run_test(
            Case::Lower,
            &[
                "uppercase",
                "lowercase",
                "lower_snake_case",
                "upper_snake_case",
                "lower-kebab-case",
                "upper-kebab-case",
                "lowercamelcase",
                "uppercamelcase",
            ],
        );
    }

    #[test]
    fn lower_snake() {
        run_test(
            Case::LowerSnake,
            &[
                "uppercase",
                "lowercase",
                "lower_snake_case",
                "upper_snake_case",
                "lower_kebab_case",
                "upper_kebab_case",
                "lower_camel_case",
                "upper_camel_case",
            ],
        );
    }

    #[test]
    fn upper_snake() {
        run_test(
            Case::UpperSnake,
            &[
                "UPPERCASE",
                "LOWERCASE",
                "LOWER_SNAKE_CASE",
                "UPPER_SNAKE_CASE",
                "LOWER_KEBAB_CASE",
                "UPPER_KEBAB_CASE",
                "LOWER_CAMEL_CASE",
                "UPPER_CAMEL_CASE",
            ],
        );
    }

    #[test]
    fn lower_kebab() {
        run_test(
            Case::LowerKebab,
            &[
                "uppercase",
                "lowercase",
                "lower-snake-case",
                "upper-snake-case",
                "lower-kebab-case",
                "upper-kebab-case",
                "lower-camel-case",
                "upper-camel-case",
            ],
        );
    }

    #[test]
    fn upper_kebab() {
        run_test(
            Case::UpperKebab,
            &[
                "UPPERCASE",
                "LOWERCASE",
                "LOWER-SNAKE-CASE",
                "UPPER-SNAKE-CASE",
                "LOWER-KEBAB-CASE",
                "UPPER-KEBAB-CASE",
                "LOWER-CAMEL-CASE",
                "UPPER-CAMEL-CASE",
            ],
        );
    }

    #[test]
    fn lower_camel() {
        run_test(
            Case::LowerCamel,
            &[
                "uppercase",
                "lowercase",
                "lowerSnakeCase",
                "upperSnakeCase",
                "lowerKebabCase",
                "upperKebabCase",
                "lowerCamelCase",
                "upperCamelCase",
            ],
        );
    }

    #[test]
    fn upper_camel() {
        run_test(
            Case::UpperCamel,
            &[
                "Uppercase",
                "Lowercase",
                "LowerSnakeCase",
                "UpperSnakeCase",
                "LowerKebabCase",
                "UpperKebabCase",
                "LowerCamelCase",
                "UpperCamelCase",
            ],
        );
    }

    fn run_test(case: Case, expected: &[&str]) {
        let test_strings = test_strings();
        assert_eq!(
            expected.len(),
            test_strings.len(),
            "Must have exactly as many expected results as test cases."
        );
        for i in 0..test_strings.len() {
            let test_str = test_strings.get(i).unwrap();
            let expected = expected.get(i).unwrap();
            assert_eq!(&case.rename(test_str), expected);
        }
    }

    fn test_strings() -> Vec<String> {
        vec![
            "UPPERCASE",
            "lowercase",
            "lower_snake_case",
            "UPPER_SNAKE_CASE",
            "lower-kebab-case",
            "UPPER-KEBAB-CASE",
            "lowerCamelCase",
            "UpperCamelCase",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    mod rename_file_name {
        use crate::template_renderer::case::Case;
        use std::path::PathBuf;

        #[test]
        fn renames_only_file_name() {
            let path = PathBuf::from("testPath/testName.ext");
            let result = Case::UpperSnake.rename_file_name(&path);
            assert_eq!(result, PathBuf::from("testPath/TEST_NAME.ext"));
        }
    }
}
