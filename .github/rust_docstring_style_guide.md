# Rust Docstring Style Guide

## Summary

This style guide outlines best practices for writing docstrings in Rust projects, specifically targeting documentation that will be published on crates.io and docs.rs. The guide provides flexible templates and rules that can be customized to fit your project's needs while maintaining standard Rust documentation practices.

## Introduction

Rust's documentation system uses Markdown formatting inside doc comments (`///` and `//!`). Well-written documentation is crucial for crate adoption and useability. This guide aims to provide a clear template for documenting Rust code elements in a way that is both thorough and adaptable.

## Module Documentation

### Module Template

```rust
//! # Module Name
//!
//! Brief description of what this module provides.
//!
//! ## Overview
//!
//! More detailed explanation about the module's purpose, organization,
//! and how its components work together.
//!
//! ## Examples
//!
//! ```
//! // Simple example showing module usage
//! use crate::module_name;
//!
//! let result = module_name::some_function();
//! assert_eq!(result, expected_value);
//! ```
```

### Module Documentation Rules

- Use `//!` (doc comments that apply to the parent) for module documentation
- **Location**: Place at the top of the module file (lib.rs, mod.rs, or module_name.rs)
- **Structure**:
  - Start with a level 1 heading with the module name
  - Include a brief one-line or one-paragraph summary
  - Optionally add an "Overview" section for more details
  - Include at least one example where appropriate
- **Formatting**: Any of these options are acceptable:
  - Sentence-style capitalization for headings
  - Title-Case for headings
- **rustfmt options**: No specific rustfmt options affect doc comments, but setting `normalize_doc_attributes = true` will normalize doc attributes if used

## Function Documentation

### Function Template

```rust
/// Performs a specific operation on the provided input.
///
/// Longer description explaining what this function does, any algorithms used,
/// performance characteristics, or other relevant details. This section can be
/// omitted for very simple functions.
///
/// # Examples
///
/// ```
/// let result = my_crate::my_function(42);
/// assert_eq!(result, 84);
/// ```
///
/// # Arguments
///
/// * `input` - Description of the input parameter
/// * `config` - Description of the configuration options
///
/// # Returns
///
/// Description of the return value and its significance.
///
/// # Errors
///
/// This function will return an error if:
/// * The input is negative
/// * The operation cannot be completed for X reason
///
/// # Panics
///
/// This function panics if:
/// * The input is zero
///
/// # Safety
///
/// This function is unsafe because:
/// * It dereferences raw pointers
/// * It assumes the caller maintains the invariant that X is true
pub fn my_function(input: i32, config: Config) -> Result<Output, Error> {
    // Implementation
}
```

### Function Documentation Rules

- Use `///` for documenting functions
- **Sections**: Include any of these sections as applicable:
  - Brief summary (always required)
  - Detailed description (optional)
  - Examples (recommended)
  - Arguments (required if function takes parameters)
  - Returns (required unless function returns `()`)
  - Errors (required for functions returning `Result`)
  - Panics (required if function can panic)
  - Safety (required for `unsafe` functions)
- **Common patterns**:
  - For simple functions, a one-line description may be sufficient
  - For complex functions, include more detail and examples
  - If a function is part of a trait implementation, you can use `# See also` to link to the trait documentation

## Struct Documentation

### Struct Template

```rust
/// A structure that represents a specific concept in your domain.
///
/// More detailed explanation about what this struct is for, how it should
/// be used, and any important considerations.
///
/// # Examples
///
/// ```
/// let instance = MyStruct::new(42, "example");
/// assert_eq!(instance.get_value(), 42);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MyStruct {
    /// The primary value stored in this struct.
    pub value: i32,

    /// Configuration string that controls behavior.
    pub name: String,

    // Fields can also have private documentation
    /// Internal state that shouldn't be modified directly.
    state: Vec<String>,
}
```

### Struct Documentation Rules

- Use `///` for the struct itself and each field that should be documented
- **Main documentation should cover**:
  - Purpose of the struct
  - Lifecycle information (if applicable)
  - Relationship to other types
  - Examples of creation and usage
- **Field documentation**:
  - Document each public field
  - For private fields, documentation is optional but helpful for maintainers
- **Constructors**:
  - If the struct has constructors (like `new()`), document them thoroughly
  - Document any invariants that constructors establish

## Enum Documentation

### Enum Template

```rust
/// Represents the different states that a process can be in.
///
/// This enum is used throughout the crate to track and manage process state.
///
/// # Examples
///
/// ```
/// let state = ProcessState::Running { pid: 123 };
/// assert!(state.is_active());
/// ```
pub enum ProcessState {
    /// Process is not yet initialized.
    NotStarted,

    /// Process is currently active.
    ///
    /// The `pid` field contains the process identifier.
    Running {
        /// The process identifier
        pid: u32
    },

    /// Process has completed successfully with an exit code.
    Completed(u8),

    /// Process terminated with an error.
    ///
    /// Contains the error code and description.
    Failed {
        /// Numeric error code
        code: i32,
        /// Human-readable error description
        description: String,
    },
}
```

### Enum Documentation Rules

- Use `///` for the enum itself and each variant
- **Main documentation should include**:
  - Purpose of the enum
  - When/where it's used in your crate
  - Examples showing creation and usage
- **Variant documentation**:
  - Document each variant
  - For variants with fields, document each field (inline or in the variant doc)
- **Usage patterns**:
  - If certain variants should be used in specific cases, document these patterns

## Trait Documentation

### Trait Template

```rust
/// Defines common behavior for types that can process data.
///
/// Implementors of this trait can transform input data and produce output
/// according to their specific processing logic.
///
/// # Examples
///
/// ```
/// struct MyProcessor;
///
/// impl DataProcessor for MyProcessor {
///     fn process(&self, input: &[u8]) -> Vec<u8> {
///         // Process the input
///         input.to_vec()
///     }
/// }
///
/// let processor = MyProcessor;
/// let result = processor.process(b"input data");
/// ```
pub trait DataProcessor {
    /// Processes the provided input data and returns the result.
    ///
    /// # Arguments
    ///
    /// * `input` - The data to process
    ///
    /// # Returns
    ///
    /// The processed data
    fn process(&self, input: &[u8]) -> Vec<u8>;

    /// Optional method with a default implementation.
    ///
    /// This method can be overridden by implementors if needed.
    fn can_process(&self) -> bool {
        true
    }
}
```

### Trait Documentation Rules

- Use `///` for the trait and each method
- **Trait documentation should include**:
  - Purpose of the trait
  - When to implement it
  - How implementations will be used
  - Example implementation
- **Method documentation**:
  - Document required methods thoroughly
  - Document default methods and when they should be overridden
  - Include parameter and return documentation
- **Associated types/constants**:
  - If the trait has associated types or constants, document their purpose and constraints

## Macro Documentation

### Macro Template

```rust
/// Creates a new structure with the specified fields.
///
/// This macro simplifies creation of common data structures by generating
/// boilerplate code.
///
/// # Examples
///
/// ```
/// // Create a new structure with fields
/// create_struct!(
///     struct Point {
///         x: i32 = 0,
///         y: i32 = 0,
///     }
/// );
///
/// let p = Point { x: 10, y: 20 };
/// assert_eq!(p.x, 10);
/// ```
///
/// # Syntax
///
/// ```
/// create_struct!(
///     $(#[$attr:meta])*
///     $vis:vis struct $name:ident {
///         $(
///             $(#[$field_attr:meta])*
///             $field_vis:vis $field_name:ident: $field_type:ty = $default:expr
///         ),* $(,)?
///     }
/// );
/// ```
#[macro_export]
macro_rules! create_struct {
    // Implementation
}
```

### Macro Documentation Rules

- Use `///` for macro documentation
- **Include the following sections**:
  - Purpose of the macro
  - Examples showing usage
  - Syntax pattern with explanation (recommended for complex macros)
- **Special considerations**:
  - Document expansion behavior when relevant
  - Note any hygiene considerations
  - Document limitations or edge cases

## Constants and Static Items

### Constants Template

```rust
/// Maximum number of concurrent connections allowed.
///
/// This value is used to limit resource consumption and prevent
/// overloading the system.
///
/// # Examples
///
/// ```
/// assert_eq!(MAX_CONNECTIONS, 100);
/// ```
pub const MAX_CONNECTIONS: usize = 100;

/// System-wide configuration flags.
///
/// These flags control various behaviors throughout the application.
/// Individual bits have specific meanings as documented.
///
/// Bit 0: Enable verbose logging
/// Bit 1: Use secure mode
/// Bits 2-31: Reserved for future use
pub static SYSTEM_FLAGS: AtomicU32 = AtomicU32::new(0);
```

### Constants and Statics Documentation Rules

- Use `///` for documentation
- **Include**:
  - Purpose and significance of the constant/static
  - Units or range information if applicable
  - How the value affects system behavior
  - For bit flags, document the meaning of each bit
- **For mutable statics**:
  - Document thread-safety considerations
  - Document synchronization requirements if applicable

## Implementation Blocks

### Implementation Block Template

```rust
/// Implementations of core functionality for the `User` struct.
impl User {
    /// Creates a new user with the given name and default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// let user = User::new("username");
    /// assert_eq!(user.name(), "username");
    /// ```
    pub fn new(name: &str) -> Self {
        // Implementation
    }
}

/// Implementation of the `Display` trait for `User`.
impl fmt::Display for User {
    /// Formats the user for display purposes.
    ///
    /// The format is "User: {name}".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User: {}", self.name)
    }
}
```

### Implementation Block Documentation Rules

- You may optionally document impl blocks themselves with `///`
- **Method documentation**: Follow the function documentation rules for each method
- **Trait implementations**: Consider documenting how the trait is implemented, especially for non-trivial implementations
- **Implementation grouping**: If you have multiple impl blocks, consider documenting the purpose of each grouping

## Additional Tips for docs.rs and crates.io

1. **Crate-level documentation**:
   - Add thorough documentation in your `lib.rs` file using `//!` comments
   - This documentation appears on your crate's front page on docs.rs

2. **README integration**:
   - Use `#[doc = include_str!("../README.md")]` to include your README in crate documentation
   - Ensure your README has proper Markdown formatting

3. **Documentation attributes**:
   - Use `#[doc(hidden)]` for items that should not appear in documentation
   - Use `#[doc(alias = "alternative")]` to make items searchable by alternative names
   - Consider `#[deprecated]` for marking deprecated items with reasons

4. **Feature flags**:
   - Document which features enable what functionality
   - Use conditional compilation with `#[cfg(feature = "...")]` for feature-specific docs

5. **Links and cross-references**:
   - Use `[Type]` or `[method]` syntax to link to other items in your crate
   - Use `[crate::path::to::item]` for explicit paths
   - Use intra-doc links like `[`Type`]` for better linking

6. **Categories and metadata**:
   - Properly set up your Cargo.toml with keywords, categories, and description
   - These appear on crates.io and help users find your crate

7. **Documentation organization**:
   - Consider using `#[doc(hidden)]` modules for internal implementation details
   - Create a clear hierarchy with public modules for different functionality areas

Remember, consistency across your codebase is more important than strictly following any specific format. Choose the patterns that work best for your project and apply them consistently.
