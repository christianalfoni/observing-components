#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_should_exclude() {
        // Test with absolute paths
        let abs_path = "/Users/christianalfoni/Development/observing-components/vite-project/src/Test.tsx";
        let patterns = vec!["src/Test.tsx".to_string()];
        assert!(should_exclude(abs_path, &patterns), "Should exclude src/Test.tsx from absolute path");
        
        // Test with file name only
        let patterns = vec!["Test.tsx".to_string()];
        assert!(should_exclude(abs_path, &patterns), "Should exclude Test.tsx by file name");
        
        // Test with glob patterns
        let patterns = vec!["src/*.tsx".to_string()];
        assert!(should_exclude(abs_path, &patterns), "Should exclude with glob pattern");
        
        // Test with pattern that shouldn't match
        let patterns = vec!["src/Other.tsx".to_string()];
        assert!(!should_exclude(abs_path, &patterns), "Shouldn't exclude non-matching path");
        
        // Test with multiple patterns
        let patterns = vec!["src/Other.tsx".to_string(), "src/Test.tsx".to_string()];
        assert!(should_exclude(abs_path, &patterns), "Should exclude with one matching pattern");
    }
    
    #[test]
    fn test_path_components_match() {
        let path = vec!["Users", "christianalfoni", "Development", "observing-components", "vite-project", "src", "Test.tsx"];
        
        // Basic match
        let pattern = vec!["src", "Test.tsx"];
        assert!(path_components_match(&path, &pattern), "Should match subsequence");
        
        // Case insensitive
        let pattern = vec!["SRC", "test.tsx"];
        assert!(path_components_match(&path, &pattern), "Should match case-insensitive");
        
        // Non-match
        let pattern = vec!["src", "Other.tsx"];
        assert!(!path_components_match(&path, &pattern), "Shouldn't match different components");
        
        // Empty pattern should match anything
        let pattern: Vec<&str> = vec![];
        assert!(path_components_match(&path, &pattern), "Empty pattern should match");
    }
}
