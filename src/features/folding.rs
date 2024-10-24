use lsp_types::{FoldingRange, FoldingRangeKind};
use wasm_bindgen::prelude::*;

/// Represents a folding range in the CSS code.
#[wasm_bindgen(js_name = FoldingRange)]
pub struct FoldingRangeWASM(FoldingRange);

#[wasm_bindgen(js_class = FoldingRange)]
impl FoldingRangeWASM {
    #[wasm_bindgen(getter)]
    pub fn start_line(&self) -> u32 {
        self.0.start_line
    }

    #[wasm_bindgen(getter)]
    pub fn start_character(&self) -> Option<u32> {
        self.0.start_character
    }

    #[wasm_bindgen(getter)]
    pub fn end_line(&self) -> u32 {
        self.0.end_line
    }

    #[wasm_bindgen(getter)]
    pub fn end_character(&self) -> Option<u32> {
        self.0.end_character
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> Option<String> {
        self.0.kind.as_ref().map(|k| match k {
            FoldingRangeKind::Comment => "comment".to_string(),
            FoldingRangeKind::Imports => "imports".to_string(),
            FoldingRangeKind::Region => "region".to_string(),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn collapsed_text(&self) -> Option<String> {
        self.0.collapsed_text.clone()
    }
}

impl From<FoldingRange> for FoldingRangeWASM {
    fn from(folding_range: FoldingRange) -> Self {
        FoldingRangeWASM(folding_range)
    }
}

/// Computes the folding ranges for the given CSS source code.
///
/// # Arguments
///
/// * `source` - The original CSS source code as a string slice.
///
/// # Returns
///
/// * A vector of `FoldingRange` indicating the foldable regions in the CSS code.
pub fn get_folding_ranges(source: &str) -> Vec<FoldingRange> {
    let mut folding_ranges = Vec::new();
    let mut stack = Vec::new();

    // Precompute line start offsets
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(idx, _)| idx + 1))
        .collect();

    for (offset, c) in source.char_indices() {
        if c == '{' {
            // Determine line number based on offset
            let line_number = line_starts.partition_point(|&line_start| line_start <= offset) - 1;
            stack.push((offset, line_number));
        } else if c == '}' {
            let line_number = line_starts.partition_point(|&line_start| line_start <= offset) - 1;
            if let Some((_start_offset, start_line)) = stack.pop() {
                if line_number > start_line {
                    let folding_range = FoldingRange {
                        start_line: start_line as u32,
                        start_character: None,
                        end_line: line_number as u32,
                        end_character: None,
                        kind: None,           // You can set FoldingRangeKind if needed
                        collapsed_text: None, // Optionally set collapsed text
                    };
                    folding_ranges.push(folding_range);
                }
            }
        }
    }

    folding_ranges
}

#[wasm_bindgen]
pub fn get_folding_ranges_wasm(source: &str) -> Vec<FoldingRangeWASM> {
    let folding_ranges = get_folding_ranges(source);
    folding_ranges
        .into_iter()
        .map(FoldingRangeWASM::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_folding_ranges_empty() {
        let code = "";
        let folding_ranges = get_folding_ranges(code);

        assert!(
            folding_ranges.is_empty(),
            "No folding ranges should be returned for empty input"
        );
    }

    #[test]
    fn test_get_folding_ranges_single_rule() {
        let code = "body {\n    margin: 0;\n    padding: 0;\n}\n";
        let folding_ranges = get_folding_ranges(code);

        assert_eq!(folding_ranges.len(), 1, "Expected one folding range");
        let range = &folding_ranges[0];
        assert_eq!(range.start_line, 0, "Folding should start at line 0");
        assert_eq!(range.end_line, 3, "Folding should end at line 3");
    }

    #[test]
    fn test_get_folding_ranges_multiple_rules() {
        let code = "body {\n    margin: 0;\n}\n\nh1 {\n    color: red;\n}\n";
        let mut folding_ranges = get_folding_ranges(code);

        assert_eq!(folding_ranges.len(), 2, "Expected two folding ranges");

        folding_ranges.sort_by_key(|fr| fr.start_line);

        let range1 = &folding_ranges[0];
        assert_eq!(range1.start_line, 0, "First folding should start at line 0");
        assert_eq!(range1.end_line, 2, "First folding should end at line 2");

        let range2 = &folding_ranges[1];
        assert_eq!(
            range2.start_line, 4,
            "Second folding should start at line 4"
        );
        assert_eq!(range2.end_line, 6, "Second folding should end at line 6");
    }

    #[test]
    fn test_get_folding_ranges_nested_rules() {
        let code = "@media screen {\n    body {\n        margin: 0;\n    }\n}\n";
        let mut folding_ranges = get_folding_ranges(code);

        assert_eq!(folding_ranges.len(), 2, "Expected two folding ranges");

        // Sort folding ranges by start_line
        folding_ranges.sort_by_key(|fr| fr.start_line);

        let outer_range = &folding_ranges[0];
        assert_eq!(
            outer_range.start_line, 0,
            "Outer folding should start at line 0"
        );
        assert_eq!(
            outer_range.end_line, 4,
            "Outer folding should end at line 4"
        );

        let inner_range = &folding_ranges[1];
        assert_eq!(
            inner_range.start_line, 1,
            "Inner folding should start at line 1"
        );
        assert_eq!(
            inner_range.end_line, 3,
            "Inner folding should end at line 3"
        );
    }

    #[test]
    fn test_get_folding_ranges_single_line_rule() {
        let code = "h1 { color: blue; }\n";
        let folding_ranges = get_folding_ranges(code);

        // Since the rule is on a single line, there should be no folding range
        assert!(
            folding_ranges.is_empty(),
            "No folding ranges expected for single-line rules"
        );
    }

    #[test]
    fn test_get_folding_ranges_unmatched_braces() {
        let code = "body {\n    margin: 0;\n    padding: 0;\n\n";
        let folding_ranges = get_folding_ranges(code);

        // The opening brace does not have a matching closing brace
        // So the folding range should not be added
        assert!(
            folding_ranges.is_empty(),
            "No folding ranges expected when braces are unmatched"
        );
    }

    #[test]
    fn test_get_folding_ranges_with_comments() {
        let code = "/* Comment block\nspanning multiple lines\n*/\nbody {\n    margin: 0;\n}\n";
        let folding_ranges = get_folding_ranges(code);

        assert_eq!(folding_ranges.len(), 1, "Expected one folding range");

        let range = &folding_ranges[0];
        assert_eq!(range.start_line, 3, "Folding should start at line 3");
        assert_eq!(range.end_line, 5, "Folding should end at line 5");
    }

    #[test]
    fn test_get_folding_ranges_complex() {
        let code = "@media screen {\n    @supports (display: grid) {\n        .container {\n            display: grid;\n        }\n    }\n}\n";
        let mut folding_ranges = get_folding_ranges(code);

        assert_eq!(folding_ranges.len(), 3, "Expected three folding ranges");

        // Sort folding ranges by start_line
        folding_ranges.sort_by_key(|fr| fr.start_line);

        let range1 = &folding_ranges[0];
        assert_eq!(range1.start_line, 0, "First folding should start at line 0");
        assert_eq!(range1.end_line, 6, "First folding should end at line 6");

        let range2 = &folding_ranges[1];
        assert_eq!(
            range2.start_line, 1,
            "Second folding should start at line 1"
        );
        assert_eq!(range2.end_line, 5, "Second folding should end at line 5");

        let range3 = &folding_ranges[2];
        assert_eq!(range3.start_line, 2, "Third folding should start at line 2");
        assert_eq!(range3.end_line, 4, "Third folding should end at line 4");
    }
}
