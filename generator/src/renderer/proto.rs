use crate::renderer::case::Case;

pub const PACKAGE_SEPARATOR: char = '.';
pub const PACKAGE_SEPARATOR_STR: &str = ".";

pub struct TypePath<'a> {
    components: Vec<String>,
    type_name: Option<String>,
    separator: Option<&'a str>,
    type_name_case: Option<Case>,
    package_case: Option<Case>,
}

impl<'a> TypePath<'a> {
    /// Assumes the incoming `package` is a package without any type name.
    pub fn from_package(package: &str) -> Self {
        Self {
            components: break_into_components(package),
            type_name: None,
            separator: None,
            type_name_case: None,
            package_case: None,
        }
    }

    /// Assumes the `type_name` is at least a type, and any preceding package separators specify
    /// the package.
    pub fn from_type(type_name: &str) -> Self {
        let (package, type_name) = extract_package_from_type(type_name);
        Self {
            components: package
                .map(|pkg| break_into_components(pkg))
                .unwrap_or_else(|| Vec::new()),
            type_name: type_name.map(str::to_string),
            separator: None,
            type_name_case: None,
            package_case: None,
        }
    }

    /// Components of the package, not including any type name.
    pub fn components(&self) -> &Vec<String> {
        &self.components
    }

    pub fn type_name_with_case(&self) -> Option<String> {
        self.type_name
            .as_ref()
            .map(|name| match self.type_name_case {
                None => name.to_string(),
                Some(case) => case.rename(&name),
            })
    }

    /// Aka number of components in our package.
    pub fn depth(&self) -> usize {
        self.components().len()
    }

    /// Set the separator to use when rendering the package into a string.
    pub fn set_separator(&mut self, sep: &'a str) {
        self.separator = Some(sep);
    }

    pub fn set_name_case(&mut self, case: Option<Case>) {
        self.type_name_case = case;
    }

    pub fn set_package_case(&mut self, case: Option<Case>) {
        self.package_case = case;
    }

    pub fn separator(&self) -> &str {
        self.separator.unwrap_or(PACKAGE_SEPARATOR_STR)
    }

    pub fn to_string(&self) -> String {
        let mut components = self
            .components
            .iter()
            .map(|s| match self.package_case {
                None => s.to_string(),
                Some(case) => case.rename(s),
            })
            .collect::<Vec<String>>();
        if let Some(type_name) = &self.type_name_with_case() {
            components.push(type_name.to_string());
        };
        components.join(self.separator())
    }

    pub fn relative_to<P: AsRef<str>, F: AsRef<str>>(
        &self,
        package: Option<P>,
        parent_prefix: Option<F>,
    ) -> String {
        let package = match package {
            None => return self.to_string(),
            Some(package) => TypePath::from_package(package.as_ref()),
        };
        let matching_depth = TypePath::matching_depth(&self, &package) as usize;
        let full_prefix = if self.components().len() > 0 {
            TypePath::create_relative_prefix(
                package.depth(),
                matching_depth,
                parent_prefix.as_ref(),
                self.separator(),
            )
        } else {
            // No package components means we're a top-level type.
            "".to_string()
        };
        if parent_prefix.is_none() && package.depth() > self.depth() {
            // When not using a parent prefix, if the package is deeper, it _has_ to specify us
            // fully qualified.
            // e.g.
            // self:    root.sub.TypeName
            // package: root.sub.inner.pkg
            // In this case, not using a prefix would conflict.
            return self.to_string();
        }
        self.to_relative_string(matching_depth, full_prefix)
    }

    /// Creates a path string that ignores the first `relative_depth` number of components.
    /// i.e. a path that is relative to that depth level.
    fn to_relative_string(&self, relative_depth: usize, prefix: String) -> String {
        let mut result = prefix;
        if !result.is_empty() {
            result.push_str(self.separator());
        }
        let mut depth = 0;
        for component in self.components() {
            if depth >= relative_depth {
                result.push_str(component);
                result.push_str(self.separator());
            }
            depth += 1;
        }
        if let Some(type_name) = &self.type_name_with_case() {
            result.push_str(type_name);
        }
        result
    }

    /// Walk up the tree, prepending a prefix for each step we take to get to the matching depth.
    /// ```txt
    ///             O
    ///             O <- matching depth = 2
    ///            / \
    ///           O   O
    ///          /     \
    /// from -> X       O
    ///                  \
    ///                   X <- to (self)
    ///
    /// Must traverse up the tree _2_ times (from.depth() - matching_depth) to reach the fork.
    /// Resulting prefix: super.super
    /// ```
    fn create_relative_prefix<S: AsRef<str>>(
        from_depth: usize,
        matching_depth: usize,
        parent_prefix: Option<S>,
        separator: &str,
    ) -> String {
        match parent_prefix {
            None => "".to_string(),
            Some(parent_prefix) => {
                let parent_count = from_depth - matching_depth;
                vec![parent_prefix.as_ref(); parent_count].join(separator)
            }
        }
    }

    /// Compares the packages of each type, returning the depth to which they match.
    /// e.g.
    /// ```txt
    /// lhs                 rhs                         depth
    /// root.sub.TypeName   root.sub.child.Other        2
    /// root.sub.TypeName   root.sub.Other              2
    /// root.sub.TypeName   root.Other                  1
    /// root.sub.TypeName   Other                       0
    /// root.sub.TypeName   alt.sub.Other               0
    /// ```
    /// depth: root.sub.TypeName vs root.TypeName =
    pub fn matching_depth(lhs: &TypePath, rhs: &TypePath) -> i32 {
        let mut matches = 0;
        for i in 0..lhs.components().len() {
            let lhs_component = lhs
                .components()
                .get(i)
                .unwrap_or_else(|| unreachable!("Bounds improperly specified"));
            match rhs.components().get(i) {
                Some(rhs_component) if rhs_component == lhs_component => matches += 1,
                _ => break,
            };
        }
        matches
    }
}

pub fn normalize_prefix(path: &str) -> &str {
    // Normalizes a path by removing the first separator.
    // e.g. ".root.sub.TypeName" to "root.sub.TypeName"
    if path.starts_with(PACKAGE_SEPARATOR) {
        &path[1..path.len()]
    } else {
        path
    }
}

fn extract_package_from_type(type_name: &str) -> (Option<&str>, Option<&str>) {
    match type_name.rsplit_once(PACKAGE_SEPARATOR) {
        None => (None, Some(type_name)),
        Some((package, name)) => (Some(package), Some(name)),
    }
}

fn break_into_components(package: &str) -> Vec<String> {
    normalize_prefix(package)
        .split(PACKAGE_SEPARATOR)
        .map(str::to_string)
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod tests {
    mod depth {
        use crate::renderer::proto::TypePath;

        #[test]
        fn depth_from_type() {
            assert_eq!(TypePath::from_type("TypeName").depth(), 0);
            assert_eq!(TypePath::from_type("root.TypeName").depth(), 1);
            assert_eq!(TypePath::from_type("root.sub.TypeName").depth(), 2);
            assert_eq!(TypePath::from_type("root.sub.inner.TypeName").depth(), 3);
        }

        #[test]
        fn depth_from_package() {
            assert_eq!(TypePath::from_package("TypeName").depth(), 1);
            assert_eq!(TypePath::from_package("root.TypeName").depth(), 2);
            assert_eq!(TypePath::from_package("root.sub.TypeName").depth(), 3);
            assert_eq!(TypePath::from_package("root.sub.inner.TypeName").depth(), 4);
        }
    }

    mod matching_depth {
        use crate::renderer::proto::TypePath;

        #[test]
        fn one_top_level() {
            let a = "root.sub.inner.TypeName";
            let b = "TypeName";
            assert_type_ab_ba(&a, &b, 0);
            assert_package_ab_ba(&a, &b, 0);
        }

        #[test]
        fn both_top_level() {
            let a = "TypeName";
            let b = "TypeName";
            assert_type_ab_ba(&a, &b, 0);
            assert_package_ab_ba(&a, &b, 1);
        }

        #[test]
        fn one_match() {
            let a = "root.sub.inner.TypeName";
            let b = "root.TypeName";
            assert_type_ab_ba(&a, &b, 1);
            assert_package_ab_ba(&a, &b, 1);
        }

        #[test]
        fn multiple_matches() {
            let a = "root.sub.inner.TypeName";
            let b = "root.sub.TypeName";
            assert_type_ab_ba(&a, &b, 2);
            assert_package_ab_ba(&a, &b, 2);
        }

        #[test]
        fn separate_tree() {
            let a = "root.sub.inner.TypeName";
            let b = "sub.TypeName";
            assert_type_ab_ba(&a, &b, 0);
            assert_package_ab_ba(&a, &b, 0);
        }

        #[test]
        fn equivalent() {
            let a = "root.sub.inner.TypeName";
            let b = "root.sub.inner.TypeName";
            assert_type_ab_ba(&a, &b, 3);
            assert_package_ab_ba(&a, &b, 4);
        }

        fn assert_type_ab_ba(a: &str, b: &str, value: i32) {
            let a = TypePath::from_type(a);
            let b = TypePath::from_type(b);
            assert_eq!(TypePath::matching_depth(&a, &b), value, "type: a vs b");
            assert_eq!(TypePath::matching_depth(&b, &a), value, "type: b vs a");
        }

        fn assert_package_ab_ba(a: &str, b: &str, value: i32) {
            let a = TypePath::from_package(a);
            let b = TypePath::from_package(b);
            assert_eq!(TypePath::matching_depth(&a, &b), value, "package: a vs b");
            assert_eq!(TypePath::matching_depth(&b, &a), value, "package: b vs a");
        }
    }

    mod to_string {
        use crate::renderer::case::Case;
        use crate::renderer::proto::TypePath;

        #[test]
        fn combines_package_and_type() {
            let path = TypePath::from_type("root.sub.TypeName");
            assert_eq!(path.to_string(), "root.sub.TypeName");
        }

        #[test]
        fn normalizes_incoming_package() {
            let path = TypePath::from_type(".root.sub.TypeName");
            assert_eq!(path.to_string(), "root.sub.TypeName");
        }

        #[test]
        fn uses_custom_separator() {
            let mut path = TypePath::from_type("root.sub.TypeName");
            path.set_separator("::");
            assert_eq!(path.to_string(), "root::sub::TypeName");
        }

        #[test]
        fn package_only() {
            let path = TypePath::from_package("root.sub");
            assert_eq!(path.to_string(), "root.sub");
        }

        #[test]
        fn type_only() {
            let path = TypePath::from_type("TypeName");
            assert_eq!(path.to_string(), "TypeName");
        }

        #[test]
        fn with_name_case() {
            let mut path = TypePath::from_type("root.sub.TypeName");
            path.set_name_case(Some(Case::UpperSnake));
            assert_eq!(path.to_string(), "root.sub.TYPE_NAME");
        }

        #[test]
        fn with_package_case() {
            let mut path = TypePath::from_type("rootPkg.subPkg.TypeName");
            path.set_package_case(Some(Case::UpperSnake));
            assert_eq!(path.to_string(), "ROOT_PKG.SUB_PKG.TypeName");
        }
    }

    mod relative_type {
        use crate::renderer::case::Case;
        use crate::renderer::proto::TypePath;

        #[test]
        fn no_package_uses_fully_qualified_type() {
            let qualified = TypePath::from_type("root.sub.TypeName");
            let result = qualified.relative_to::<&str, &str>(None, None);
            assert_eq!(result, qualified.to_string());
        }

        #[test]
        fn different_prefix_uses_fully_qualified_type() {
            let qualified = TypePath::from_type("root.sub.TypeName");
            let result = qualified.relative_to::<&str, &str>(Some("other.sub"), None);
            assert_eq!(result, qualified.to_string());
        }

        #[test]
        fn matching_longer_prefix_uses_fully_qualified_type() {
            let qualified = TypePath::from_type("root.sub.TypeName");
            let result = qualified.relative_to::<&str, &str>(Some("root.sub.sub2.sub3"), None);
            assert_eq!(result, qualified.to_string());
        }

        #[test]
        fn matching_shorter_prefix_uses_partially_qualified_type() {
            let qualified = TypePath::from_type("root.sub.TypeName");
            let result = qualified.relative_to::<&str, &str>(Some("root"), None);
            assert_eq!(result, "sub.TypeName");
        }

        #[test]
        fn matching_prefix_uses_non_qualified_type() {
            let qualified = TypePath::from_type("root.sub.TypeName");
            let result = qualified.relative_to::<&str, &str>(Some("root.sub"), None);
            assert_eq!(result, "TypeName");
        }

        #[test]
        fn fully_qualified_with_case() {
            let mut qualified = TypePath::from_type("root.sub.TypeName");
            qualified.set_name_case(Some(Case::UpperSnake));
            let result = qualified.relative_to::<&str, &str>(None, None);
            assert_eq!(result, "root.sub.TYPE_NAME");
        }

        #[test]
        fn relative_with_case() {
            let mut qualified = TypePath::from_type("root.sub.TypeName");
            qualified.set_name_case(Some(Case::UpperSnake));
            let result = qualified.relative_to::<&str, &str>(Some("root"), None);
            assert_eq!(result, "sub.TYPE_NAME");
        }

        mod with_parent_prefix {
            use crate::renderer::proto::TypePath;

            #[test]
            fn sibling() {
                run_test("grand.parent.me.Sibling", "grand.parent.me", "Sibling");
            }

            #[test]
            fn child() {
                run_test(
                    "grand.parent.me.child.Name",
                    "grand.parent.me",
                    "child.Name",
                );
            }

            #[test]
            fn parent() {
                run_test("grand.parent.Name", "grand.parent.me", "super.Name");
            }

            #[test]
            fn grandparent() {
                run_test("grand.Name", "grand.parent.me", "super.super.Name");
            }

            #[test]
            fn cousin() {
                run_test(
                    "grand.parent.cousin.Name",
                    "grand.parent.me",
                    "super.cousin.Name",
                );
            }

            #[test]
            fn separate_family() {
                run_test(
                    "other.sub.Name",
                    "grand.parent.me",
                    "super.super.super.other.sub.Name",
                );
            }

            #[test]
            fn top_level() {
                run_test("Name", "grand.parent.me", "Name");
            }

            fn run_test(qualified: &str, package: &str, expected: &str) {
                let qualified = TypePath::from_type(qualified);
                let relative_type = qualified.relative_to(Some(&package), Some(&"super"));
                assert_eq!(relative_type, expected);
            }
        }
    }

    mod extract_package_from_type {
        use crate::renderer::proto::extract_package_from_type;

        #[test]
        fn single_component_is_type_name() {
            let (package, type_name) = extract_package_from_type("TypeName");
            assert_eq!(package, None);
            assert_eq!(type_name, Some("TypeName"));
        }

        #[test]
        fn multiple_component_splits() {
            let (package, type_name) = extract_package_from_type("root.sub.TypeName");
            assert_eq!(package, Some("root.sub"));
            assert_eq!(type_name, Some("TypeName"));
        }
    }
}
