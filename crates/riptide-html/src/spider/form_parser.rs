//! HTML form detection and parsing functionality

use crate::spider::traits::{FormData, FormField, DomSpiderConfig};
use anyhow::{Context, Result};
use scraper::{Html, Selector, ElementRef};
use url::Url;

/// HTML form parser for extracting form data and structure
pub struct HtmlFormParser {
    config: DomSpiderConfig,
}

impl HtmlFormParser {
    /// Create a new form parser with configuration
    pub fn new(config: DomSpiderConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(DomSpiderConfig::default())
    }

    /// Extract all forms from HTML content
    pub async fn extract_forms(&self, html: &str, base_url: &Url) -> Result<Vec<FormData>> {
        if !self.config.extract_forms {
            return Ok(Vec::new());
        }

        let document = Html::parse_document(html);
        let form_selector = Selector::parse("form")
            .map_err(|e| anyhow::anyhow!("Invalid form selector: {}", e))?;

        let mut forms = Vec::new();

        for form_element in document.select(&form_selector) {
            if let Ok(form_data) = self.parse_form(form_element, base_url).await {
                forms.push(form_data);
            }
        }

        Ok(forms)
    }

    /// Extract forms that are likely to be search forms
    pub async fn extract_search_forms(&self, html: &str, base_url: &Url) -> Result<Vec<FormData>> {
        let all_forms = self.extract_forms(html, base_url).await?;

        Ok(all_forms
            .into_iter()
            .filter(|form| self.is_search_form(form))
            .collect())
    }

    /// Extract forms that are likely to be login forms
    pub async fn extract_login_forms(&self, html: &str, base_url: &Url) -> Result<Vec<FormData>> {
        let all_forms = self.extract_forms(html, base_url).await?;

        Ok(all_forms
            .into_iter()
            .filter(|form| self.is_login_form(form))
            .collect())
    }

    /// Parse a single form element
    async fn parse_form(&self, form_element: ElementRef<'_>, base_url: &Url) -> Result<FormData> {
        // Extract form attributes
        let action = form_element
            .value()
            .attr("action")
            .and_then(|action_str| {
                if action_str.trim().is_empty() {
                    // Empty action means current page
                    Some(base_url.clone())
                } else {
                    base_url.join(action_str).ok()
                }
            });

        let method = form_element
            .value()
            .attr("method")
            .unwrap_or("GET")
            .to_uppercase();

        let enctype = form_element
            .value()
            .attr("enctype")
            .map(|s| s.to_string());

        let name = form_element
            .value()
            .attr("name")
            .or_else(|| form_element.value().attr("id"))
            .map(|s| s.to_string());

        // Extract form fields
        let fields = self.extract_form_fields(form_element).await?;

        Ok(FormData {
            action,
            method,
            enctype,
            fields,
            name,
        })
    }

    /// Extract all input fields from a form
    async fn extract_form_fields(&self, form_element: ElementRef<'_>) -> Result<Vec<FormField>> {
        let mut fields = Vec::new();

        // Input fields
        let input_selector = Selector::parse("input")
            .map_err(|e| anyhow::anyhow!("Invalid input selector: {}", e))?;

        for input in form_element.select(&input_selector) {
            if let Some(field) = self.parse_input_field(input) {
                fields.push(field);
            }
        }

        // Textarea fields
        let textarea_selector = Selector::parse("textarea")
            .map_err(|e| anyhow::anyhow!("Invalid textarea selector: {}", e))?;

        for textarea in form_element.select(&textarea_selector) {
            if let Some(field) = self.parse_textarea_field(textarea) {
                fields.push(field);
            }
        }

        // Select fields
        let select_selector = Selector::parse("select")
            .map_err(|e| anyhow::anyhow!("Invalid select selector: {}", e))?;

        for select in form_element.select(&select_selector) {
            if let Some(field) = self.parse_select_field(select) {
                fields.push(field);
            }
        }

        // Button fields
        let button_selector = Selector::parse("button")
            .map_err(|e| anyhow::anyhow!("Invalid button selector: {}", e))?;

        for button in form_element.select(&button_selector) {
            if let Some(field) = self.parse_button_field(button) {
                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// Parse an input field
    fn parse_input_field(&self, input: ElementRef<'_>) -> Option<FormField> {
        let field_type = input
            .value()
            .attr("type")
            .unwrap_or("text")
            .to_string();

        // Filter by configured field types
        if !self.config.form_field_types.contains(&field_type) {
            return None;
        }

        let name = input.value().attr("name")?.to_string();

        let value = input
            .value()
            .attr("value")
            .map(|s| s.to_string());

        let required = input.value().attr("required").is_some();

        let placeholder = input
            .value()
            .attr("placeholder")
            .map(|s| s.to_string());

        Some(FormField {
            field_type,
            name,
            value,
            required,
            placeholder,
        })
    }

    /// Parse a textarea field
    fn parse_textarea_field(&self, textarea: ElementRef<'_>) -> Option<FormField> {
        if !self.config.form_field_types.contains(&"textarea".to_string()) {
            return None;
        }

        let name = textarea.value().attr("name")?.to_string();

        let value = textarea
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into();

        let value = if value == Some("".to_string()) {
            None
        } else {
            value
        };

        let required = textarea.value().attr("required").is_some();

        let placeholder = textarea
            .value()
            .attr("placeholder")
            .map(|s| s.to_string());

        Some(FormField {
            field_type: "textarea".to_string(),
            name,
            value,
            required,
            placeholder,
        })
    }

    /// Parse a select field
    fn parse_select_field(&self, select: ElementRef<'_>) -> Option<FormField> {
        if !self.config.form_field_types.contains(&"select".to_string()) {
            return None;
        }

        let name = select.value().attr("name")?.to_string();

        // Find selected option
        let option_selector = Selector::parse("option[selected]").ok()?;
        let value = select
            .select(&option_selector)
            .next()
            .and_then(|option| option.value().attr("value"))
            .map(|s| s.to_string());

        let required = select.value().attr("required").is_some();

        Some(FormField {
            field_type: "select".to_string(),
            name,
            value,
            required,
            placeholder: None,
        })
    }

    /// Parse a button field
    fn parse_button_field(&self, button: ElementRef<'_>) -> Option<FormField> {
        let button_type = button
            .value()
            .attr("type")
            .unwrap_or("button")
            .to_string();

        if !self.config.form_field_types.contains(&button_type) {
            return None;
        }

        let name = button
            .value()
            .attr("name")
            .unwrap_or("")
            .to_string();

        let value = button
            .value()
            .attr("value")
            .or_else(|| {
                // Use button text content as value
                let text = button.text().collect::<String>().trim().to_string();
                if text.is_empty() { None } else { Some(&text) }
            })
            .map(|s| s.to_string());

        Some(FormField {
            field_type: button_type,
            name,
            value,
            required: false,
            placeholder: None,
        })
    }

    /// Check if a form is likely a search form
    fn is_search_form(&self, form: &FormData) -> bool {
        // Check form name/action for search indicators
        let search_indicators = ["search", "query", "q", "find", "lookup"];

        // Check form action URL
        if let Some(action) = &form.action {
            let action_str = action.to_string().to_lowercase();
            if search_indicators.iter().any(|&indicator| action_str.contains(indicator)) {
                return true;
            }
        }

        // Check form name
        if let Some(name) = &form.name {
            let name_lower = name.to_lowercase();
            if search_indicators.iter().any(|&indicator| name_lower.contains(indicator)) {
                return true;
            }
        }

        // Check field names
        for field in &form.fields {
            let field_name_lower = field.name.to_lowercase();
            if search_indicators.iter().any(|&indicator| field_name_lower.contains(indicator)) {
                return true;
            }
        }

        // Check for common search form patterns
        let has_text_input = form.fields.iter().any(|f| f.field_type == "text");
        let has_submit = form.fields.iter().any(|f|
            f.field_type == "submit" || f.field_type == "button"
        );
        let is_get_method = form.method == "GET";

        has_text_input && has_submit && is_get_method && form.fields.len() <= 5
    }

    /// Check if a form is likely a login form
    fn is_login_form(&self, form: &FormData) -> bool {
        let login_indicators = ["login", "signin", "auth", "account"];

        // Check form action URL
        if let Some(action) = &form.action {
            let action_str = action.to_string().to_lowercase();
            if login_indicators.iter().any(|&indicator| action_str.contains(indicator)) {
                return true;
            }
        }

        // Check form name
        if let Some(name) = &form.name {
            let name_lower = name.to_lowercase();
            if login_indicators.iter().any(|&indicator| name_lower.contains(indicator)) {
                return true;
            }
        }

        // Look for username/password field patterns
        let has_username_field = form.fields.iter().any(|f| {
            let name_lower = f.name.to_lowercase();
            name_lower.contains("user")
                || name_lower.contains("email")
                || name_lower.contains("login")
        });

        let has_password_field = form.fields.iter().any(|f| {
            f.field_type == "password"
        });

        let has_submit = form.fields.iter().any(|f|
            f.field_type == "submit" || f.field_type == "button"
        );

        let is_post_method = form.method == "POST";

        has_username_field && has_password_field && has_submit && is_post_method
    }

    /// Extract CSRF tokens from forms
    pub async fn extract_csrf_tokens(&self, html: &str) -> Result<Vec<(String, String)>> {
        let document = Html::parse_document(html);
        let mut tokens = Vec::new();

        // Common CSRF token field names
        let csrf_names = [
            "csrf_token",
            "csrf",
            "_token",
            "_csrf",
            "authenticity_token",
            "_authenticity_token",
            "csrfmiddlewaretoken",
            "_csrfmiddlewaretoken",
        ];

        let input_selector = Selector::parse("input[type=\"hidden\"]")
            .map_err(|e| anyhow::anyhow!("Invalid hidden input selector: {}", e))?;

        for input in document.select(&input_selector) {
            if let Some(name) = input.value().attr("name") {
                let name_lower = name.to_lowercase();

                // Check if this is a CSRF token field
                if csrf_names.iter().any(|&csrf_name| name_lower.contains(csrf_name)) {
                    if let Some(value) = input.value().attr("value") {
                        tokens.push((name.to_string(), value.to_string()));
                    }
                }
            }
        }

        // Also check meta tags for CSRF tokens
        let meta_selector = Selector::parse("meta[name*=\"csrf\"], meta[name*=\"token\"]")
            .map_err(|e| anyhow::anyhow!("Invalid meta selector: {}", e))?;

        for meta in document.select(&meta_selector) {
            if let (Some(name), Some(content)) = (
                meta.value().attr("name"),
                meta.value().attr("content")
            ) {
                tokens.push((name.to_string(), content.to_string()));
            }
        }

        Ok(tokens)
    }

    /// Check if page has any forms that require authentication
    pub async fn has_authenticated_forms(&self, html: &str, base_url: &Url) -> Result<bool> {
        let forms = self.extract_forms(html, base_url).await?;

        Ok(forms.iter().any(|form| {
            self.is_login_form(form) || self.requires_authentication(form)
        }))
    }

    /// Check if a form likely requires authentication to access
    fn requires_authentication(&self, form: &FormData) -> bool {
        // Check for forms with sensitive field types
        let sensitive_types = ["password", "email"];
        let has_sensitive_fields = form.fields.iter().any(|f|
            sensitive_types.contains(&f.field_type.as_str())
        );

        // Check for POST method (often used for authenticated operations)
        let is_post = form.method == "POST";

        // Check action URL for protected areas
        let protected_paths = ["/admin", "/dashboard", "/profile", "/account", "/user"];
        let has_protected_action = form.action
            .as_ref()
            .map(|url| {
                let path = url.path().to_lowercase();
                protected_paths.iter().any(|&protected| path.contains(protected))
            })
            .unwrap_or(false);

        has_sensitive_fields || (is_post && has_protected_action)
    }

    /// Extract form validation patterns (for automated form submission)
    pub async fn extract_validation_patterns(&self, html: &str) -> Result<Vec<(String, String)>> {
        let document = Html::parse_document(html);
        let mut patterns = Vec::new();

        let input_selector = Selector::parse("input[pattern], input[data-pattern]")
            .map_err(|e| anyhow::anyhow!("Invalid pattern selector: {}", e))?;

        for input in document.select(&input_selector) {
            if let Some(name) = input.value().attr("name") {
                // Check for HTML5 pattern attribute
                if let Some(pattern) = input.value().attr("pattern") {
                    patterns.push((name.to_string(), pattern.to_string()));
                }

                // Check for data-pattern attribute
                if let Some(pattern) = input.value().attr("data-pattern") {
                    patterns.push((name.to_string(), pattern.to_string()));
                }
            }
        }

        Ok(patterns)
    }
}