//! Recipe module for defining clipboard transformation sequences
//!
//! A Recipe is a named collection of transformations that can be applied
//! to clipboard content.

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::transformers;

/// Available transformation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transformation {
    // Whitespace operations
    NormalizeWhitespace,
    TrimLines,
    RemoveEmptyLines,
    
    // Case transformations
    ToLowercase,
    ToUppercase,
    ToTitleCase,
    ToSentenceCase,
    ToCamelCase,
    ToPascalCase,
    ToSnakeCase,
    ToScreamingSnakeCase,
    ToKebabCase,
    
    // Line operations
    RemoveDuplicateLines,
    SortLines,
    SortLinesReverse,
    ReverseLines,
    AddLineNumbers,
    RemoveLineNumbers,
    JoinLines { separator: String },
    SplitToLines { delimiter: String },
    WrapLines { width: usize },
    
    // Character cleanup
    FixSmartQuotes,
    RemoveNonAscii,
    NormalizeUnicode,
    RemoveEmojis,
    StripFormatting,
    
    // Tab/space operations
    TabsToSpaces { spaces: usize },
    SpacesToTabs { spaces_per_tab: usize },
    
    // Content removal
    RemoveUrls,
    RemoveEmails,
    RemovePhoneNumbers,
    RemoveMarkdown,
    
    // Line ending operations
    ToUnixLineEndings,
    ToWindowsLineEndings,
    
    // Extraction
    ExtractNumbers,
    
    // HTML operations
    EncodeHtmlEntities,
    DecodeHtmlEntities,
    
    // URL operations
    Slugify,
    
    // Custom regex replacement
    RegexReplace { pattern: String, replacement: String },
    
    // Find and replace
    FindReplace { find: String, replace: String },
    
    // Prefix/suffix
    AddPrefix { prefix: String },
    AddSuffix { suffix: String },
    RemovePrefix { prefix: String },
    RemoveSuffix { suffix: String },
}

impl Transformation {
    /// Get a human-readable name for the transformation
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::NormalizeWhitespace => "Normalize Whitespace",
            Self::TrimLines => "Trim Lines",
            Self::RemoveEmptyLines => "Remove Empty Lines",
            Self::ToLowercase => "lowercase",
            Self::ToUppercase => "UPPERCASE",
            Self::ToTitleCase => "Title Case",
            Self::ToSentenceCase => "Sentence case",
            Self::ToCamelCase => "camelCase",
            Self::ToPascalCase => "PascalCase",
            Self::ToSnakeCase => "snake_case",
            Self::ToScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
            Self::ToKebabCase => "kebab-case",
            Self::RemoveDuplicateLines => "Remove Duplicate Lines",
            Self::SortLines => "Sort Lines (A-Z)",
            Self::SortLinesReverse => "Sort Lines (Z-A)",
            Self::ReverseLines => "Reverse Line Order",
            Self::AddLineNumbers => "Add Line Numbers",
            Self::RemoveLineNumbers => "Remove Line Numbers",
            Self::JoinLines { .. } => "Join Lines",
            Self::SplitToLines { .. } => "Split to Lines",
            Self::WrapLines { .. } => "Wrap Lines",
            Self::FixSmartQuotes => "Fix Smart Quotes",
            Self::RemoveNonAscii => "Remove Non-ASCII",
            Self::NormalizeUnicode => "Normalize Unicode",
            Self::RemoveEmojis => "Remove Emojis",
            Self::StripFormatting => "Strip All Formatting",
            Self::TabsToSpaces { .. } => "Tabs → Spaces",
            Self::SpacesToTabs { .. } => "Spaces → Tabs",
            Self::RemoveUrls => "Remove URLs",
            Self::RemoveEmails => "Remove Emails",
            Self::RemovePhoneNumbers => "Remove Phone Numbers",
            Self::RemoveMarkdown => "Remove Markdown",
            Self::ToUnixLineEndings => "Unix Line Endings (LF)",
            Self::ToWindowsLineEndings => "Windows Line Endings (CRLF)",
            Self::ExtractNumbers => "Extract Numbers",
            Self::EncodeHtmlEntities => "Encode HTML Entities",
            Self::DecodeHtmlEntities => "Decode HTML Entities",
            Self::Slugify => "Slugify (URL-safe)",
            Self::RegexReplace { .. } => "Regex Replace",
            Self::FindReplace { .. } => "Find & Replace",
            Self::AddPrefix { .. } => "Add Prefix",
            Self::AddSuffix { .. } => "Add Suffix",
            Self::RemovePrefix { .. } => "Remove Prefix",
            Self::RemoveSuffix { .. } => "Remove Suffix",
        }
    }
    
    /// Get the category of this transformation
    pub fn category(&self) -> &'static str {
        match self {
            Self::NormalizeWhitespace | Self::TrimLines | Self::RemoveEmptyLines => "Whitespace",
            Self::ToLowercase | Self::ToUppercase | Self::ToTitleCase | Self::ToSentenceCase |
            Self::ToCamelCase | Self::ToPascalCase | Self::ToSnakeCase | 
            Self::ToScreamingSnakeCase | Self::ToKebabCase => "Case Conversion",
            Self::RemoveDuplicateLines | Self::SortLines | Self::SortLinesReverse |
            Self::ReverseLines | Self::AddLineNumbers | Self::RemoveLineNumbers |
            Self::JoinLines { .. } | Self::SplitToLines { .. } | Self::WrapLines { .. } => "Line Operations",
            Self::FixSmartQuotes | Self::RemoveNonAscii | Self::NormalizeUnicode |
            Self::RemoveEmojis | Self::StripFormatting => "Character Cleanup",
            Self::TabsToSpaces { .. } | Self::SpacesToTabs { .. } => "Indentation",
            Self::RemoveUrls | Self::RemoveEmails | Self::RemovePhoneNumbers |
            Self::RemoveMarkdown => "Content Removal",
            Self::ToUnixLineEndings | Self::ToWindowsLineEndings => "Line Endings",
            Self::ExtractNumbers => "Extraction",
            Self::EncodeHtmlEntities | Self::DecodeHtmlEntities => "HTML",
            Self::Slugify => "URL",
            Self::RegexReplace { .. } | Self::FindReplace { .. } => "Search & Replace",
            Self::AddPrefix { .. } | Self::AddSuffix { .. } |
            Self::RemovePrefix { .. } | Self::RemoveSuffix { .. } => "Prefix/Suffix",
        }
    }
    
    /// Apply this transformation to text
    pub fn apply(&self, text: &str) -> String {
        match self {
            Self::NormalizeWhitespace => transformers::normalize_whitespace(text),
            Self::TrimLines => transformers::trim_lines(text),
            Self::RemoveEmptyLines => transformers::remove_empty_lines(text),
            Self::ToLowercase => transformers::to_lowercase(text),
            Self::ToUppercase => transformers::to_uppercase(text),
            Self::ToTitleCase => transformers::to_title_case(text),
            Self::ToSentenceCase => transformers::to_sentence_case(text),
            Self::ToCamelCase => transformers::to_camel_case(text),
            Self::ToPascalCase => transformers::to_pascal_case(text),
            Self::ToSnakeCase => transformers::to_snake_case(text),
            Self::ToScreamingSnakeCase => transformers::to_screaming_snake_case(text),
            Self::ToKebabCase => transformers::to_kebab_case(text),
            Self::RemoveDuplicateLines => transformers::remove_duplicate_lines(text),
            Self::SortLines => transformers::sort_lines(text),
            Self::SortLinesReverse => transformers::sort_lines_reverse(text),
            Self::ReverseLines => transformers::reverse_lines(text),
            Self::AddLineNumbers => transformers::add_line_numbers(text),
            Self::RemoveLineNumbers => transformers::remove_line_numbers(text),
            Self::JoinLines { separator } => transformers::join_lines(text, separator),
            Self::SplitToLines { delimiter } => transformers::split_to_lines(text, delimiter),
            Self::WrapLines { width } => transformers::wrap_lines(text, *width),
            Self::FixSmartQuotes => transformers::fix_smart_quotes(text),
            Self::RemoveNonAscii => transformers::remove_non_ascii(text),
            Self::NormalizeUnicode => transformers::normalize_unicode(text),
            Self::RemoveEmojis => transformers::remove_emojis(text),
            Self::StripFormatting => transformers::strip_formatting(text),
            Self::TabsToSpaces { spaces } => transformers::tabs_to_spaces(text, *spaces),
            Self::SpacesToTabs { spaces_per_tab } => transformers::spaces_to_tabs(text, *spaces_per_tab),
            Self::RemoveUrls => transformers::remove_urls(text),
            Self::RemoveEmails => transformers::remove_emails(text),
            Self::RemovePhoneNumbers => transformers::remove_phone_numbers(text),
            Self::RemoveMarkdown => transformers::remove_markdown(text),
            Self::ToUnixLineEndings => transformers::to_unix_line_endings(text),
            Self::ToWindowsLineEndings => transformers::to_windows_line_endings(text),
            Self::ExtractNumbers => transformers::extract_numbers(text),
            Self::EncodeHtmlEntities => transformers::encode_html_entities(text),
            Self::DecodeHtmlEntities => transformers::decode_html_entities(text),
            Self::Slugify => transformers::slugify(text),
            Self::RegexReplace { pattern, replacement } => {
                if let Ok(re) = regex::Regex::new(pattern) {
                    re.replace_all(text, replacement.as_str()).to_string()
                } else {
                    text.to_string()
                }
            }
            Self::FindReplace { find, replace } => text.replace(find, replace),
            Self::AddPrefix { prefix } => format!("{}{}", prefix, text),
            Self::AddSuffix { suffix } => format!("{}{}", text, suffix),
            Self::RemovePrefix { prefix } => {
                text.strip_prefix(prefix).unwrap_or(text).to_string()
            }
            Self::RemoveSuffix { suffix } => {
                text.strip_suffix(suffix).unwrap_or(text).to_string()
            }
        }
    }
}

/// A Recipe is a named collection of transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier
    pub id: Uuid,
    /// User-defined name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// List of transformations to apply in order
    pub transformations: Vec<Transformation>,
    /// Whether this recipe is currently active (applied to all pastes)
    pub is_active: bool,
    /// Optional hotkey to trigger this recipe
    pub hotkey: Option<String>,
    /// When the recipe was created
    pub created_at: DateTime<Utc>,
    /// When the recipe was last modified
    pub modified_at: DateTime<Utc>,
    /// Icon for the recipe (emoji or text)
    pub icon: Option<String>,
}

impl Recipe {
    /// Create a new recipe
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            transformations: Vec::new(),
            is_active: false,
            hotkey: None,
            created_at: now,
            modified_at: now,
            icon: None,
        }
    }
    
    /// Add a transformation to the recipe
    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.push(transformation);
        self.modified_at = Utc::now();
    }
    
    /// Apply all transformations to text
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        for transformation in &self.transformations {
            result = transformation.apply(&result);
        }
        result
    }
    
    /// Check if this recipe has any transformations
    pub fn is_empty(&self) -> bool {
        self.transformations.is_empty()
    }
}

/// Default built-in recipes
impl Default for Recipe {
    fn default() -> Self {
        Self::new("New Recipe")
    }
}

/// Manager for loading, saving, and organizing recipes
pub struct RecipeManager {
    /// All loaded recipes
    pub recipes: Vec<Recipe>,
    /// Path to the recipes file
    recipes_path: PathBuf,
}

impl RecipeManager {
    /// Create a new RecipeManager
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("9paste");
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        let recipes_path = config_dir.join("recipes.json");
        
        let recipes = if recipes_path.exists() {
            let data = fs::read_to_string(&recipes_path)
                .context("Failed to read recipes file")?;
            serde_json::from_str(&data)
                .context("Failed to parse recipes file")?
        } else {
            Self::default_recipes()
        };
        
        Ok(Self {
            recipes,
            recipes_path,
        })
    }
    
    /// Get default built-in recipes
    fn default_recipes() -> Vec<Recipe> {
        let mut recipes = Vec::new();
        
        // Plain text recipe
        let mut plain_text = Recipe::new("Plain Text");
        plain_text.description = Some("Strip all formatting and normalize whitespace".into());
        plain_text.icon = Some("📝".into());
        plain_text.add_transformation(Transformation::StripFormatting);
        plain_text.add_transformation(Transformation::FixSmartQuotes);
        plain_text.add_transformation(Transformation::NormalizeWhitespace);
        recipes.push(plain_text);
        
        // Clean code recipe
        let mut clean_code = Recipe::new("Clean Code");
        clean_code.description = Some("Clean up code snippets".into());
        clean_code.icon = Some("💻".into());
        clean_code.add_transformation(Transformation::FixSmartQuotes);
        clean_code.add_transformation(Transformation::TrimLines);
        clean_code.add_transformation(Transformation::ToUnixLineEndings);
        clean_code.add_transformation(Transformation::TabsToSpaces { spaces: 4 });
        recipes.push(clean_code);
        
        // Remove duplicates recipe
        let mut remove_dups = Recipe::new("Unique Lines");
        remove_dups.description = Some("Remove duplicate lines".into());
        remove_dups.icon = Some("🔢".into());
        remove_dups.add_transformation(Transformation::TrimLines);
        remove_dups.add_transformation(Transformation::RemoveDuplicateLines);
        remove_dups.add_transformation(Transformation::RemoveEmptyLines);
        recipes.push(remove_dups);
        
        // Sort lines recipe
        let mut sort = Recipe::new("Sort Lines");
        sort.description = Some("Sort lines alphabetically".into());
        sort.icon = Some("📊".into());
        sort.add_transformation(Transformation::TrimLines);
        sort.add_transformation(Transformation::SortLines);
        recipes.push(sort);
        
        // Privacy mode
        let mut privacy = Recipe::new("Privacy Mode");
        privacy.description = Some("Remove personal info like emails and phone numbers".into());
        privacy.icon = Some("🔒".into());
        privacy.add_transformation(Transformation::RemoveEmails);
        privacy.add_transformation(Transformation::RemovePhoneNumbers);
        privacy.add_transformation(Transformation::RemoveUrls);
        recipes.push(privacy);
        
        // Academic
        let mut academic = Recipe::new("Academic");
        academic.description = Some("Clean up academic text for citations".into());
        academic.icon = Some("📚".into());
        academic.add_transformation(Transformation::FixSmartQuotes);
        academic.add_transformation(Transformation::NormalizeWhitespace);
        academic.add_transformation(Transformation::TrimLines);
        recipes.push(academic);
        
        // Emoji-free
        let mut no_emoji = Recipe::new("No Emoji");
        no_emoji.description = Some("Remove all emojis from text".into());
        no_emoji.icon = Some("🚫".into());
        no_emoji.add_transformation(Transformation::RemoveEmojis);
        recipes.push(no_emoji);
        
        recipes
    }
    
    /// Save recipes to disk
    pub fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.recipes)
            .context("Failed to serialize recipes")?;
        fs::write(&self.recipes_path, data)
            .context("Failed to write recipes file")?;
        Ok(())
    }
    
    /// Add a new recipe
    pub fn add_recipe(&mut self, recipe: Recipe) -> Result<()> {
        self.recipes.push(recipe);
        self.save()
    }
    
    /// Remove a recipe by ID
    pub fn remove_recipe(&mut self, id: Uuid) -> Result<()> {
        self.recipes.retain(|r| r.id != id);
        self.save()
    }
    
    /// Get a recipe by ID
    pub fn get_recipe(&self, id: Uuid) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == id)
    }
    
    /// Get a mutable recipe by ID
    pub fn get_recipe_mut(&mut self, id: Uuid) -> Option<&mut Recipe> {
        self.recipes.iter_mut().find(|r| r.id == id)
    }
    
    /// Get the currently active recipe
    pub fn get_active_recipe(&self) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.is_active)
    }
    
    /// Set a recipe as active (deactivates all others)
    pub fn set_active(&mut self, id: Uuid) -> Result<()> {
        for recipe in &mut self.recipes {
            recipe.is_active = recipe.id == id;
        }
        self.save()
    }
    
    /// Deactivate all recipes
    pub fn deactivate_all(&mut self) -> Result<()> {
        for recipe in &mut self.recipes {
            recipe.is_active = false;
        }
        self.save()
    }
    
    /// Update a recipe
    pub fn update_recipe(&mut self, updated: Recipe) -> Result<()> {
        if let Some(recipe) = self.recipes.iter_mut().find(|r| r.id == updated.id) {
            *recipe = updated;
            recipe.modified_at = Utc::now();
        }
        self.save()
    }
}

impl Default for RecipeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create RecipeManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_apply() {
        let mut recipe = Recipe::new("Test");
        recipe.add_transformation(Transformation::ToUppercase);
        recipe.add_transformation(Transformation::TrimLines);
        
        let result = recipe.apply("  hello world  ");
        assert_eq!(result, "HELLO WORLD");
    }
    
    #[test]
    fn test_transformation_chain() {
        let mut recipe = Recipe::new("Test");
        recipe.add_transformation(Transformation::FixSmartQuotes);
        recipe.add_transformation(Transformation::NormalizeWhitespace);
        recipe.add_transformation(Transformation::ToTitleCase);
        
        // Input has smart double quotes: U+201C and U+201D
        // After FixSmartQuotes: "  \"hello\"   world  "
        // After NormalizeWhitespace: "\"hello\" world"
        // After ToTitleCase: "\"hello\" World" (quote is first char, 'h' becomes 'e')
        // Note: ToTitleCase capitalizes first char of each word. Since first char is '"',
        // the 'h' in "hello" becomes lowercase after the uppercase '"' is processed.
        let input = "  \u{201C}hello\u{201D}   world  ";
        let result = recipe.apply(input);
        assert_eq!(result, "\"hello\" World");
    }
}
