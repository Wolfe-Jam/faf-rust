//! FAFB Priority System
//!
//! Priority determines truncation order when context window is constrained.
//! Higher priority = more important = truncated last.

/// Critical priority - never truncate (name, version)
pub const PRIORITY_CRITICAL: u8 = 255;

/// High priority - truncate last (key_files, tech_stack)
pub const PRIORITY_HIGH: u8 = 200;

/// Medium priority - normal truncation (architecture, commands)
pub const PRIORITY_MEDIUM: u8 = 128;

/// Low priority - truncate first (verbose context)
pub const PRIORITY_LOW: u8 = 64;

/// Optional priority - can be omitted entirely
pub const PRIORITY_OPTIONAL: u8 = 0;

/// Priority level with semantic meaning
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

impl Priority {
    /// Create a new priority from raw value
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Critical priority - never truncate
    pub const fn critical() -> Self {
        Self(PRIORITY_CRITICAL)
    }

    /// High priority - truncate last
    pub const fn high() -> Self {
        Self(PRIORITY_HIGH)
    }

    /// Medium priority - normal
    pub const fn medium() -> Self {
        Self(PRIORITY_MEDIUM)
    }

    /// Low priority - truncate first
    pub const fn low() -> Self {
        Self(PRIORITY_LOW)
    }

    /// Optional - can be omitted
    pub const fn optional() -> Self {
        Self(PRIORITY_OPTIONAL)
    }

    /// Get raw priority value
    pub const fn value(&self) -> u8 {
        self.0
    }

    /// Check if this should never be truncated
    pub const fn is_critical(&self) -> bool {
        self.0 == PRIORITY_CRITICAL
    }

    /// Check if this is high priority (>= 200)
    pub const fn is_high(&self) -> bool {
        self.0 >= PRIORITY_HIGH
    }

    /// Check if this is optional (0)
    pub const fn is_optional(&self) -> bool {
        self.0 == PRIORITY_OPTIONAL
    }

    /// Get human-readable level name
    pub const fn level_name(&self) -> &'static str {
        match self.0 {
            255 => "CRITICAL",
            200..=254 => "HIGH",
            128..=199 => "MEDIUM",
            1..=127 => "LOW",
            0 => "OPTIONAL",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::medium()
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Priority> for u8 {
    fn from(priority: Priority) -> Self {
        priority.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_constants() {
        assert_eq!(PRIORITY_CRITICAL, 255);
        assert_eq!(PRIORITY_HIGH, 200);
        assert_eq!(PRIORITY_MEDIUM, 128);
        assert_eq!(PRIORITY_LOW, 64);
        assert_eq!(PRIORITY_OPTIONAL, 0);
    }

    #[test]
    fn test_priority_constructors() {
        assert_eq!(Priority::critical().value(), 255);
        assert_eq!(Priority::high().value(), 200);
        assert_eq!(Priority::medium().value(), 128);
        assert_eq!(Priority::low().value(), 64);
        assert_eq!(Priority::optional().value(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::critical() > Priority::high());
        assert!(Priority::high() > Priority::medium());
        assert!(Priority::medium() > Priority::low());
        assert!(Priority::low() > Priority::optional());
    }

    #[test]
    fn test_is_critical() {
        assert!(Priority::critical().is_critical());
        assert!(!Priority::high().is_critical());
    }

    #[test]
    fn test_is_high() {
        assert!(Priority::critical().is_high());
        assert!(Priority::high().is_high());
        assert!(!Priority::medium().is_high());
    }

    #[test]
    fn test_is_optional() {
        assert!(Priority::optional().is_optional());
        assert!(!Priority::low().is_optional());
    }

    #[test]
    fn test_level_names() {
        assert_eq!(Priority::critical().level_name(), "CRITICAL");
        assert_eq!(Priority::high().level_name(), "HIGH");
        assert_eq!(Priority::medium().level_name(), "MEDIUM");
        assert_eq!(Priority::low().level_name(), "LOW");
        assert_eq!(Priority::optional().level_name(), "OPTIONAL");
    }

    #[test]
    fn test_default_is_medium() {
        assert_eq!(Priority::default(), Priority::medium());
    }
}
