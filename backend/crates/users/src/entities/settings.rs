use chrono::Utc;
use serde::{Deserialize, Serialize};

/// User settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// Database primary key
    pub id: i64,
    /// User ID this settings belongs to
    pub user_id: i64,
    /// User preferences
    pub preferences: UserPreferences,
    /// Privacy settings
    pub privacy: PrivacySettings,
    /// Display settings
    pub display: DisplaySettings,
    /// Notification settings (reference to notification preferences)
    pub notification_settings_id: Option<i64>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Language preference (ISO 639-1 code)
    pub language: String,
    /// Timezone (IANA timezone identifier)
    pub timezone: String,
    /// Date format preference
    pub date_format: DateFormat,
    /// Time format preference
    pub time_format: TimeFormat,
    /// Theme preference
    pub theme: Theme,
    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval_seconds: u32,
    /// Show timestamps in chat
    pub show_timestamps: bool,
    /// Show user avatars
    pub show_avatars: bool,
    /// Compact mode enabled
    pub compact_mode: bool,
    /// Enable keyboard shortcuts
    pub keyboard_shortcuts_enabled: bool,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Profile visibility
    pub profile_visibility: ProfileVisibility,
    /// Show online status
    pub show_online_status: bool,
    /// Allow direct messages from anyone
    pub allow_direct_messages: bool,
    /// Allow chat invitations from anyone
    pub allow_chat_invitations: bool,
    /// Show activity status
    pub show_activity_status: bool,
    /// Share read receipts
    pub share_read_receipts: bool,
    /// Enable two-factor authentication
    pub two_factor_enabled: bool,
    /// Data retention period in days
    pub data_retention_days: u32,
}

/// Display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Font size preference
    pub font_size: FontSize,
    /// Enable high contrast mode
    pub high_contrast_enabled: bool,
    /// Enable reduced motion
    pub reduced_motion_enabled: bool,
    /// Sidebar position
    pub sidebar_position: SidebarPosition,
    /// Show line numbers in code blocks
    pub show_line_numbers: bool,
    /// Enable syntax highlighting
    pub syntax_highlighting_enabled: bool,
    /// Code theme preference
    pub code_theme: String,
    /// Custom CSS URL (optional)
    pub custom_css_url: Option<String>,
}

/// Date format enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DateFormat {
    ISO8601,
    US,
    European,
    Custom,
}

/// Time format enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    TwentyFourHour,
    TwelveHour,
}

/// Theme enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    Auto,
    System,
}

/// Profile visibility enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProfileVisibility {
    Public,
    Friends,
    Private,
}

/// Font size enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// Sidebar position enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
    Hidden,
}

impl From<&str> for DateFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "us" => DateFormat::US,
            "european" => DateFormat::European,
            "custom" => DateFormat::Custom,
            _ => DateFormat::ISO8601,
        }
    }
}

impl From<DateFormat> for String {
    fn from(format: DateFormat) -> Self {
        match format {
            DateFormat::ISO8601 => "iso8601".to_string(),
            DateFormat::US => "us".to_string(),
            DateFormat::European => "european".to_string(),
            DateFormat::Custom => "custom".to_string(),
        }
    }
}

impl From<&str> for TimeFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "12h" => TimeFormat::TwelveHour,
            "12-hour" => TimeFormat::TwelveHour,
            _ => TimeFormat::TwentyFourHour,
        }
    }
}

impl From<TimeFormat> for String {
    fn from(format: TimeFormat) -> Self {
        match format {
            TimeFormat::TwentyFourHour => "24h".to_string(),
            TimeFormat::TwelveHour => "12h".to_string(),
        }
    }
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Auto,
        }
    }
}

impl From<Theme> for String {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => "light".to_string(),
            Theme::Dark => "dark".to_string(),
            Theme::Auto => "auto".to_string(),
            Theme::System => "system".to_string(),
        }
    }
}

impl UserSettings {
    /// Create default user settings
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be set by database
            user_id,
            preferences: UserPreferences::default(),
            privacy: PrivacySettings::default(),
            display: DisplaySettings::default(),
            notification_settings_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validate settings data
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        self.preferences.validate()?;
        self.privacy.validate()?;
        self.display.validate()?;

        Ok(())
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            date_format: DateFormat::ISO8601,
            time_format: TimeFormat::TwentyFourHour,
            theme: Theme::Auto,
            auto_save_enabled: true,
            auto_save_interval_seconds: 30,
            show_timestamps: true,
            show_avatars: true,
            compact_mode: false,
            keyboard_shortcuts_enabled: true,
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_visibility: ProfileVisibility::Private,
            show_online_status: true,
            allow_direct_messages: true,
            allow_chat_invitations: true,
            show_activity_status: false,
            share_read_receipts: false,
            two_factor_enabled: false,
            data_retention_days: 365,
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            font_size: FontSize::Medium,
            high_contrast_enabled: false,
            reduced_motion_enabled: false,
            sidebar_position: SidebarPosition::Left,
            show_line_numbers: true,
            syntax_highlighting_enabled: true,
            code_theme: "github-dark".to_string(),
            custom_css_url: None,
        }
    }
}

impl UserPreferences {
    /// Validate user preferences
    pub fn validate(&self) -> Result<(), String> {
        // Validate language code (basic validation)
        if self.language.len() != 2 || !self.language.is_ascii() {
            return Err("Invalid language code (must be 2 ASCII characters)".to_string());
        }

        // Validate timezone (basic validation)
        if self.timezone.is_empty() {
            return Err("Timezone cannot be empty".to_string());
        }

        if self.timezone.len() > 50 {
            return Err("Timezone too long (max 50 characters)".to_string());
        }

        // Validate auto-save interval
        if self.auto_save_interval_seconds == 0 || self.auto_save_interval_seconds > 3600 {
            return Err("Auto-save interval must be between 1 and 3600 seconds".to_string());
        }

        Ok(())
    }
}

impl PrivacySettings {
    /// Validate privacy settings
    pub fn validate(&self) -> Result<(), String> {
        // Validate data retention period
        if self.data_retention_days == 0 || self.data_retention_days > 365 * 10 {
            return Err("Data retention period must be between 1 and 3650 days".to_string());
        }

        Ok(())
    }
}

impl DisplaySettings {
    /// Validate display settings
    pub fn validate(&self) -> Result<(), String> {
        // Validate custom CSS URL if provided
        if let Some(ref css_url) = self.custom_css_url {
            if !css_url.starts_with("http://") && !css_url.starts_with("https://") {
                return Err("Custom CSS URL must be a valid HTTP/HTTPS URL".to_string());
            }

            if css_url.len() > 500 {
                return Err("Custom CSS URL too long (max 500 characters)".to_string());
            }
        }

        // Validate code theme
        if self.code_theme.is_empty() {
            return Err("Code theme cannot be empty".to_string());
        }

        if self.code_theme.len() > 100 {
            return Err("Code theme too long (max 100 characters)".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_creation() {
        let settings = UserSettings::new(1);

        assert_eq!(settings.user_id, 1);
        assert_eq!(settings.preferences.language, "en");
        assert_eq!(settings.preferences.timezone, "UTC");
        assert_eq!(settings.privacy.profile_visibility, ProfileVisibility::Private);
        assert_eq!(settings.display.font_size, FontSize::Medium);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_date_format_conversion() {
        assert_eq!(DateFormat::from("iso8601"), DateFormat::ISO8601);
        assert_eq!(DateFormat::from("US"), DateFormat::US);
        assert_eq!(DateFormat::from("european"), DateFormat::European);
        assert_eq!(DateFormat::from("custom"), DateFormat::Custom);
        assert_eq!(DateFormat::from("unknown"), DateFormat::ISO8601);

        assert_eq!(String::from(DateFormat::ISO8601), "iso8601");
        assert_eq!(String::from(DateFormat::US), "us");
        assert_eq!(String::from(DateFormat::European), "european");
        assert_eq!(String::from(DateFormat::Custom), "custom");
    }

    #[test]
    fn test_time_format_conversion() {
        assert_eq!(TimeFormat::from("24h"), TimeFormat::TwentyFourHour);
        assert_eq!(TimeFormat::from("12h"), TimeFormat::TwelveHour);
        assert_eq!(TimeFormat::from("12-hour"), TimeFormat::TwelveHour);
        assert_eq!(TimeFormat::from("unknown"), TimeFormat::TwentyFourHour);

        assert_eq!(String::from(TimeFormat::TwentyFourHour), "24h");
        assert_eq!(String::from(TimeFormat::TwelveHour), "12h");
    }

    #[test]
    fn test_theme_conversion() {
        assert_eq!(Theme::from("light"), Theme::Light);
        assert_eq!(Theme::from("dark"), Theme::Dark);
        assert_eq!(Theme::from("system"), Theme::System);
        assert_eq!(Theme::from("auto"), Theme::Auto);
        assert_eq!(Theme::from("unknown"), Theme::Auto);

        assert_eq!(String::from(Theme::Light), "light");
        assert_eq!(String::from(Theme::Dark), "dark");
        assert_eq!(String::from(Theme::System), "system");
        assert_eq!(String::from(Theme::Auto), "auto");
    }

    #[test]
    fn test_user_preferences_validation() {
        let mut prefs = UserPreferences::default();
        assert!(prefs.validate().is_ok());

        // Invalid language code
        prefs.language = "invalid".to_string();
        assert!(prefs.validate().is_err());

        prefs.language = "en".to_string();
        prefs.auto_save_interval_seconds = 0;
        assert!(prefs.validate().is_err());

        prefs.auto_save_interval_seconds = 30;
        assert!(prefs.validate().is_ok());
    }

    #[test]
    fn test_privacy_settings_validation() {
        let mut privacy = PrivacySettings::default();
        assert!(privacy.validate().is_ok());

        // Invalid retention period
        privacy.data_retention_days = 0;
        assert!(privacy.validate().is_err());

        privacy.data_retention_days = 4000; // More than 10 years
        assert!(privacy.validate().is_err());

        privacy.data_retention_days = 365;
        assert!(privacy.validate().is_ok());
    }

    #[test]
    fn test_display_settings_validation() {
        let mut display = DisplaySettings::default();
        assert!(display.validate().is_ok());

        // Invalid custom CSS URL
        display.custom_css_url = Some("invalid-url".to_string());
        assert!(display.validate().is_err());

        display.custom_css_url = Some("https://example.com/style.css".to_string());
        assert!(display.validate().is_ok());

        // Invalid code theme
        display.code_theme = "".to_string();
        assert!(display.validate().is_err());

        display.code_theme = "valid-theme".to_string();
        assert!(display.validate().is_ok());
    }

    #[test]
    fn test_user_settings_validation() {
        let settings = UserSettings::new(1);
        assert!(settings.validate().is_ok());

        let invalid_settings = UserSettings {
            id: 0,
            user_id: 0, // Invalid user ID
            preferences: UserPreferences::default(),
            privacy: PrivacySettings::default(),
            display: DisplaySettings::default(),
            notification_settings_id: None,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        assert!(invalid_settings.validate().is_err());
    }

    #[test]
    fn test_update_timestamp() {
        let mut settings = UserSettings::new(1);
        let original_updated_at = settings.updated_at.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        settings.touch();
        assert_ne!(settings.updated_at, original_updated_at);
    }
}