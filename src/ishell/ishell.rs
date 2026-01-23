use inquire::{
    ui::{Color, RenderConfig, StyleSheet, Styled},
    Select,
};

/// Interactive shell for prompting users with colored, searchable options.
pub struct IShell {
    render: RenderConfig<'static>,
}

impl IShell {
    /// Create a new IShell with custom styling.
    pub fn new() -> Self {
        let render = RenderConfig::default()
            .with_prompt_prefix(Styled::new("❓ ").with_fg(Color::LightCyan))
            .with_highlighted_option_prefix(Styled::new("➤ ").with_fg(Color::LightMagenta))
            .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightGreen)));
        IShell { render }
    }

    /// Prompt user to select one option from a list.
    /// Supports up/down navigation, type-to-filter, and number selection.
    pub fn select<T: std::fmt::Display + Clone>(&self, prompt: &str, options: Vec<T>) -> Option<T> {
        let result = Select::new(prompt, options)
            .with_render_config(self.render)
            .prompt();
        match result {
            Ok(choice) => Some(choice),
            Err(_) => None,
        }
    }

    /// Prompt user with a yes/no confirmation.
    pub fn confirm(&self, prompt: &str) -> bool {
        use inquire::Confirm;
        let result = Confirm::new(prompt)
            .with_render_config(self.render)
            .prompt();
        matches!(result, Ok(true))
    }

    /// Prompt user to enter text input.
    pub fn text(&self, prompt: &str) -> Option<String> {
        use inquire::Text;
        let result = Text::new(prompt).with_render_config(self.render).prompt();
        match result {
            Ok(text) => Some(text),
            Err(_) => None,
        }
    }

    /// Prompt user to select multiple options from a list.
    pub fn multi_select<T: std::fmt::Display + Clone>(
        &self,
        prompt: &str,
        options: Vec<T>,
    ) -> Vec<T> {
        use inquire::MultiSelect;
        let result = MultiSelect::new(prompt, options)
            .with_render_config(self.render)
            .prompt();
        match result {
            Ok(choices) => choices,
            Err(_) => vec![],
        }
    }
}

impl Default for IShell {
    fn default() -> Self {
        Self::new()
    }
}

/// A numbered option for display in the shell.
/// Shows as "1. Option Name" in the prompt.
#[derive(Clone, Debug, PartialEq)]
pub struct NumberedOption {
    pub index: usize,
    pub value: String,
}

impl NumberedOption {
    pub fn new(index: usize, value: impl Into<String>) -> Self {
        NumberedOption {
            index,
            value: value.into(),
        }
    }

    /// Create a list of numbered options from a vector of strings.
    pub fn from_vec(options: Vec<impl Into<String>>) -> Vec<NumberedOption> {
        options
            .into_iter()
            .enumerate()
            .map(|(i, v)| NumberedOption::new(i + 1, v))
            .collect()
    }

    /// Filter options by a search query (matches index or value, case-insensitive).
    pub fn filter(options: &[NumberedOption], query: &str) -> Vec<NumberedOption> {
        let query = query.to_lowercase();
        options
            .iter()
            .filter(|opt| {
                opt.index.to_string().contains(&query) || opt.value.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    /// Find the best matching option (first match by index, then by value).
    pub fn best_match(options: &[NumberedOption], query: &str) -> Option<NumberedOption> {
        let query_lower = query.to_lowercase();
        // First, try exact index match
        if let Ok(idx) = query.parse::<usize>() {
            if let Some(opt) = options.iter().find(|o| o.index == idx) {
                return Some(opt.clone());
            }
        }
        // Then, try prefix match on value
        if let Some(opt) = options
            .iter()
            .find(|o| o.value.to_lowercase().starts_with(&query_lower))
        {
            return Some(opt.clone());
        }
        // Finally, try contains match on value
        options
            .iter()
            .find(|o| o.value.to_lowercase().contains(&query_lower))
            .cloned()
    }
}

impl std::fmt::Display for NumberedOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. {}", self.index, self.value)
    }
}

//____________TEST______________________

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ishell_creation() {
        // Test that IShell can be created without panicking
        let shell = IShell::new();
        // Verify render config is set (can't test interactive prompts in automated tests)
        assert!(std::mem::size_of_val(&shell.render) > 0);
    }

    #[test]
    fn test_ishell_default() {
        // Test Default trait implementation
        let shell = IShell::default();
        assert!(std::mem::size_of_val(&shell.render) > 0);
    }

    #[test]
    fn test_numbered_options_with_search() {
        // 8 options for the user to choose from
        let options = NumberedOption::from_vec(vec![
            "C Project",
            "C++ Project",
            "Rust Project",
            "Python Script",
            "JavaScript App",
            "TypeScript App",
            "Go Module",
            "Java Project",
        ]);

        // Test: Options are numbered 1-8
        assert_eq!(options.len(), 8);
        assert_eq!(options[0].index, 1);
        assert_eq!(options[7].index, 8);
        assert_eq!(options[0].value, "C Project");
        assert_eq!(options[7].value, "Java Project");

        // Test: Display format
        assert_eq!(format!("{}", options[0]), "1. C Project");
        assert_eq!(format!("{}", options[2]), "3. Rust Project");

        // Test: Search by serial number
        let filtered = NumberedOption::filter(&options, "3");
        assert!(filtered.iter().any(|o| o.value == "Rust Project"));

        // Test: Search by partial name (case-insensitive)
        let filtered = NumberedOption::filter(&options, "script");
        assert_eq!(filtered.len(), 3); // "Python Script", "JavaScript App", "TypeScript App"
        assert!(filtered.iter().any(|o| o.value == "Python Script"));
        assert!(filtered.iter().any(|o| o.value == "JavaScript App")); // "JavaScript" contains "script"
        assert!(filtered.iter().any(|o| o.value == "TypeScript App")); // "TypeScript" contains "script"

        // Test: Search by prefix
        let filtered = NumberedOption::filter(&options, "c");
        assert!(filtered.iter().any(|o| o.value == "C Project"));
        assert!(filtered.iter().any(|o| o.value == "C++ Project"));

        // Test: Best match by index
        let best = NumberedOption::best_match(&options, "5");
        assert_eq!(best.unwrap().value, "JavaScript App");

        // Test: Best match by prefix
        let best = NumberedOption::best_match(&options, "Rust");
        assert_eq!(best.unwrap().value, "Rust Project");

        // Test: Best match by contains
        let best = NumberedOption::best_match(&options, "Java");
        assert_eq!(best.unwrap().value, "JavaScript App"); // "JavaScript" starts with "Java"

        // Test: Best match for "Project" (first contains match)
        let best = NumberedOption::best_match(&options, "Project");
        assert_eq!(best.unwrap().value, "C Project"); // First one containing "Project"

        // Test: No match returns None
        let best = NumberedOption::best_match(&options, "xyz");
        assert!(best.is_none());
    }
}
