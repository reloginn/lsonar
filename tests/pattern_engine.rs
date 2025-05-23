use lsonar::{LUA_MAXCAPTURES, Parser, Result, engine::find_first_match};
use std::ops::Range;

fn find(
    pattern_str: &str,
    text: &str,
) -> Result<Option<(Range<usize>, Vec<Option<Range<usize>>>)>> {
    let mut parser = Parser::new(pattern_str)?;
    let ast = parser.parse()?;
    find_first_match(&ast, text.as_bytes(), 0) // 0-based index only for tests
}

fn assert_match(
    pattern: &str,
    text: &str,
    expected_full: Range<usize>,
    expected_captures: &[Option<Range<usize>>],
) {
    let result = find(pattern, text).expect("find failed");
    match result {
        Some((full_match, captures)) => {
            assert_eq!(full_match, expected_full, "Full match range mismatch");
            let num_expected = expected_captures.len();
            assert_eq!(
                &captures[..num_expected],
                expected_captures,
                "Captures mismatch"
            );
        }
        None => panic!(
            "Expected match, but found none for pattern '{}' in text '{}'",
            pattern, text
        ),
    }
}

fn assert_no_match(pattern: &str, text: &str) {
    let result = find(pattern, text).expect("find failed");
    assert!(
        result.is_none(),
        "Expected no match, but found one for pattern '{}' in text '{}'",
        pattern,
        text
    );
}

#[test]
fn test_literal_match_engine() {
    assert_match("abc", "abc", 0..3, &[]);
    assert_match("abc", "xabc", 1..4, &[]);
    assert_match("abc", "abcy", 0..3, &[]);
    assert_no_match("abc", "axbyc");
    assert_no_match("abc", "ab");
    assert_no_match("abc", "");
}

#[test]
fn test_any_match_engine() {
    assert_match(".", "a", 0..1, &[]);
    assert_match("a.c", "axc", 0..3, &[]);
    assert_match("a.c", "a\nc", 0..3, &[]);
    assert_no_match(".", "");
}

#[test]
fn test_class_match_engine() {
    assert_match("%d", "5", 0..1, &[]);
    assert_match("%a", "Z", 0..1, &[]);
    assert_match("%l", "z", 0..1, &[]);
    assert_match("%s", " ", 0..1, &[]);
    assert_match("%x", "f", 0..1, &[]);
    assert_match("a%dz", "a1z", 0..3, &[]);
    assert_no_match("%d", "a");
    assert_match("%D", "a", 0..1, &[]);
    assert_no_match("%D", "5");
    assert_match("%S", "a", 0..1, &[]);
    assert_no_match("%S", " ");
}

#[test]
fn test_set_match_engine() {
    assert_match("[abc]", "a", 0..1, &[]);
    assert_match("[abc]", "b", 0..1, &[]);
    assert_match("[^abc]", "d", 0..1, &[]);
    assert_match("[a-z]", "m", 0..1, &[]);
    assert_match("[%d%s]", "5", 0..1, &[]);
    assert_match("[%d%s]", " ", 0..1, &[]);
    assert_no_match("[abc]", "d");
    assert_no_match("[^abc]", "a");
    assert_no_match("[a-z]", "A");
    assert_no_match("[a-z]", "5");
    assert_no_match("[%d%s]", "a");
}

#[test]
fn test_anchor_match_engine() {
    assert_match("^abc", "abc", 0..3, &[]);
    assert_no_match("^abc", "xabc");
    assert_match("abc$", "abc", 0..3, &[]);
    assert_no_match("abc$", "abcd");
    assert_match("^abc$", "abc", 0..3, &[]);
    assert_no_match("^abc$", "xabc");
    assert_no_match("^abc$", "abcd");
    assert_match("^", "", 0..0, &[]);
    assert_match("$", "", 0..0, &[]);
    assert_match("^$", "", 0..0, &[]);
}

#[test]
fn test_greedy_quantifiers_engine() {
    assert_match("a*", "aaa", 0..3, &[]);
    assert_match("a*", "", 0..0, &[]);
    assert_match("a*b", "aaab", 0..4, &[]);
    assert_match("a*b", "b", 0..1, &[]);
    assert_match("x*", "y", 0..0, &[]);
    assert_match("a+", "aaa", 0..3, &[]);
    assert_no_match("a+", "");
    assert_match("a+b", "aaab", 0..4, &[]);
    assert_no_match("a+b", "b");
    assert_match("a?", "a", 0..1, &[]);
    assert_match("a?", "", 0..0, &[]);
    assert_match("a?b", "ab", 0..2, &[]);
    assert_match("a?b", "b", 0..1, &[]);
    assert_match("a*a", "aaa", 0..3, &[]);
    assert_match(".*b", "axbyb", 0..5, &[]);
    assert_match("a+a", "aa", 0..2, &[]);
    assert_match("a?a", "aa", 0..2, &[]);
    assert_match("a?a", "a", 0..1, &[]);
}

#[test]
fn test_non_greedy_quantifier_engine() {
    assert_match("a-", "aaa", 0..0, &[]);
    assert_match("a-", "", 0..0, &[]);
    assert_match("a-b", "aaab", 0..4, &[]);
    assert_match("a-b", "b", 0..1, &[]);
    assert_match("x-", "y", 0..0, &[]);
    assert_match(".-b", "axbyb", 0..3, &[]);
    assert_match("a-a", "aaa", 0..1, &[]);
}

#[test]
fn test_captures_simple_engine() {
    assert_match("(a)", "a", 0..1, &[Some(0..1)]);
    assert_match("(.)", "b", 0..1, &[Some(0..1)]);
    assert_match("(%d)", "3", 0..1, &[Some(0..1)]);
    assert_match("a(b)c", "abc", 0..3, &[Some(1..2)]);
    assert_match("a(.)c", "axc", 0..3, &[Some(1..2)]);
    assert_match("(a)(b)", "ab", 0..2, &[Some(0..1), Some(1..2)]);
    assert_match("()(b)", "b", 0..1, &[Some(0..0), Some(0..1)]);
}

#[test]
fn test_captures_quantified_engine() {
    assert_match("(a)*", "aaa", 0..3, &[Some(2..3)]);
    assert_match("(a)+", "aaa", 0..3, &[Some(2..3)]);
    assert_match("(a)?", "a", 0..1, &[Some(0..1)]);
    assert_match("(a)?", "", 0..0, &[None]);
    assert_match("a(b)*c", "abbbc", 0..5, &[Some(3..4)]);
    assert_match("a(b)+c", "abbbc", 0..5, &[Some(3..4)]);
    assert_match("a(b)?c", "abc", 0..3, &[Some(1..2)]);
    assert_match("a(b)?c", "ac", 0..2, &[None]);
    assert_match("a(b)-c", "abbbc", 0..5, &[Some(3..4)]);
    assert_match("a(b)-c", "abbbc", 0..5, &[Some(3..4)]);
}

#[test]
fn test_captures_nested_engine() {
    assert_match("(a(b)c)", "abc", 0..3, &[Some(0..3), Some(1..2)]);
    assert_match("((.)%w*)", "a1 b2", 0..2, &[Some(0..2), Some(0..1)]);
}

#[test]
fn test_balanced_engine() {
    assert_match("%b()", "(inner)", 0..7, &[]);
    assert_match("%b<>", "<<a>>", 0..5, &[]);
    assert_match("a %b() c", "a (bal) c", 0..9, &[]);
    assert_match("%b()", "()", 0..2, &[]);
    assert_no_match("%b()", "(unbalanced");
    assert_match("%b()", "x()y", 1..3, &[]);
}

#[test]
fn test_frontier_engine() {
    assert_match("%f[a]a", " a", 1..2, &[]);
    assert_match("%f[a]a", "ba", 1..2, &[]);

    assert_no_match("%f[^%w]word", "_word");
    assert_no_match("%f[^%w]word", "1word");
    assert_no_match("%f[%s]a", " a");

    assert_match("%f[a]a", "a", 0..1, &[]);
    assert_match("%f[^a]b", "b", 0..1, &[]);
}

#[test]
fn test_backtracking_engine() {
    assert_no_match("a*b", "aaac");
    assert_no_match("a+b", "aaac");
    assert_match("(ab)+a", "abab", 0..3, &[]);
    assert_match("(a*)b", "aaab", 0..4, &[Some(0..3)]);
    assert_match("(a+)b", "aaab", 0..4, &[Some(0..3)]);
    assert_match("a[bc]+d", "abbcd", 0..5, &[]);
}

#[test]
fn test_empty_engine() {
    assert_match("", "", 0..0, &[]);
    assert_match("", "abc", 0..0, &[]);
    assert_no_match("a", "");
    assert_match("a*", "", 0..0, &[]);
    assert_no_match("a+", "");
    assert_match("a?", "", 0..0, &[]);
    assert_match("()", "", 0..0, &[Some(0..0)]);
}

#[test]
fn test_find_offset_engine() {
    let pattern = "b";
    let text = "abc";
    let mut parser = Parser::new(pattern).unwrap();
    let ast = parser.parse().unwrap();
    let result = find_first_match(&ast, text.as_bytes(), 1).unwrap();
    assert_eq!(result, Some((1..2, vec![None; LUA_MAXCAPTURES])));

    let result2 = find_first_match(&ast, text.as_bytes(), 2).unwrap();
    assert!(result2.is_none());
}

#[test]
fn test_real_world_email_validation_engine() {
    assert_match(
        "^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$",
        "user@example.com",
        0..16,
        &[],
    );
    assert_match(
        "^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$",
        "user.name+tag-123@example-site.co.uk",
        0..36,
        &[],
    );

    assert_no_match("^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", "user@example");
    assert_no_match("^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", "@example.com");
    assert_no_match("^[%w%.%+%-]+@[%w%.%+%-]+%.%w+$", "user@.com");
}

#[test]
fn test_extracting_data_with_captures_engine() {
    let result = find("(%d%d?)/(%d%d?)/(%d%d%d%d)", "Date: 25/12/2023")
        .unwrap()
        .unwrap();
    let (full, captures) = result;
    assert_eq!(full, 6..16);
    assert_eq!(captures[0], Some(6..8));
    assert_eq!(captures[1], Some(9..11));
    assert_eq!(captures[2], Some(12..16));

    let result = find(
        "([%w%.%+%-]+)@([%w%.%+%-]+%.%w+)",
        "Contact: john.doe@example.com",
    )
    .unwrap()
    .unwrap();
    let (full, captures) = result;
    assert_eq!(full, 9..29);
    assert_eq!(captures[0], Some(9..17));
    assert_eq!(captures[1], Some(18..29));
}

#[test]
fn test_balanced_delimiters_engine() {
    assert_match("%b<>", "<div><p>text</p></div>", 0..5, &[]);
    assert_match("%b()", "(a + (b * c))", 0..13, &[]);
    assert_match("'%b\"\"'", "'\"nested\"'", 0..10, &[]);
    assert_match("before %b() after", "before (balanced) after", 0..23, &[]);
}

#[test]
fn test_frontier_patterns_engine() {
    assert_match("%f[%a]t%w+", "start the test", 6..9, &[]);
    assert_match("%w+t%f[^%w]", "start the test", 0..5, &[]);
    assert_match("%f[%w]word%f[^%w]", "a word here", 2..6, &[]);
    assert_no_match("%f[%w]word%f[^%w]", "aword here");
}

#[test]
fn test_complex_pattern_combinations_engine() {
    let pattern = "<a%s+href=\"([^\"]+)\"[^>]*>([^<]*)</a>";
    let text = "<p>Visit <a href=\"https://example.com\" class=\"link\">Example Site</a> for more info.</p>";

    let result = find(pattern, text).unwrap().unwrap();
    let (full, captures) = result;
    assert_eq!(full, 9..68);
    assert_eq!(captures[0], Some(18..37));
    assert_eq!(captures[1], Some(52..64));

    assert_match("%f[%w][%u][%l]+%f[^%w]", "This is a Test string", 0..4, &[]);

    let result = find("([^,]+),([^,]+),([^,]+)", "apple,orange,banana")
        .unwrap()
        .unwrap();
    let (_, captures) = result;
    assert_eq!(captures[0], Some(0..5));
    assert_eq!(captures[1], Some(6..12));
    assert_eq!(captures[2], Some(13..19));
}

#[test]
fn test_optimization_cases_engine() {
    let mut parser = Parser::new("^abc").unwrap();
    let ast = parser.parse().unwrap();

    assert!(
        find_first_match(&ast, "abcdef".as_bytes(), 0)
            .unwrap()
            .is_some()
    );
    assert!(
        find_first_match(&ast, "abcdef".as_bytes(), 1)
            .unwrap()
            .is_none()
    );

    let mut parser = Parser::new("abc$").unwrap();
    let ast = parser.parse().unwrap();

    assert!(
        find_first_match(&ast, "xyzabc".as_bytes(), 0)
            .unwrap()
            .is_some()
    );
    assert!(
        find_first_match(&ast, "abcxyz".as_bytes(), 0)
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_pattern_with_utf8_content_engine() {
    assert_match(".", "привет", 0..1, &[]);
    assert_match("..", "привет", 0..2, &[]);

    assert_match("[%w]+", "привет123", 12..15, &[]);

    assert_match("%a+", "hello привет", 0..5, &[]);
}

#[test]
fn test_quantifiers_with_capturing_groups_engine() {
    assert_match("(a)+", "aaa", 0..3, &[Some(2..3)]);
    assert_match("(ab)+", "ababab", 0..6, &[Some(4..6)]);
    assert_match("(a)*", "aaa", 0..3, &[Some(2..3)]);
    assert_match("(a)*", "", 0..0, &[None]);
    assert_match("(a)?", "a", 0..1, &[Some(0..1)]);
    assert_match("(a)?", "", 0..0, &[None]);
    assert_match("(a)-", "aaa", 0..0, &[None]);
}

#[test]
fn test_edge_cases_and_backtracking_engine() {
    assert_match("(a+)+", "aaa", 0..3, &[Some(0..3)]);
    assert_match("[ab][cd]", "ac", 0..2, &[]);
    assert_match("[ab][cd]", "bd", 0..2, &[]);
    assert_no_match("[ab][cd]", "ab");
    assert_match("a.-b", "axxxbyyybzzz", 0..5, &[]);
    assert_match("a.*b", "axxxbyyybzzz", 0..9, &[]);
    assert_match("(a*)(b?)b+", "aaabbb", 0..6, &[Some(0..3), Some(3..4)]);
}

#[test]
fn test_real_world_patterns_advanced_engine() {
    let html = "<div class='item'><span>Product: </span>Laptop</div><div class='price'>$999</div>";
    let pattern = "<div class='([^']+)'>([^<]*<span>[^<]*</span>)?([^<]*)</div>";

    let result = find(pattern, html).unwrap().unwrap();
    let (full, captures) = result;
    assert_eq!(full, 0..52);
    assert_eq!(captures[0], Some(12..16));
    assert_eq!(captures[1], Some(18..40));
    assert_eq!(captures[2], Some(40..46));

    let log_line = "2023-04-15 14:23:45 ERROR [app.service] Failed to connect: timeout";
    let pattern = "(%d+)%-(%d+)%-(%d+) (%d+):(%d+):(%d+) (%u+)";

    let result = find(pattern, log_line).unwrap().unwrap();
    let (full, captures) = result;
    assert_eq!(full, 0..25);
    assert_eq!(captures[0], Some(0..4));
    assert_eq!(captures[1], Some(5..7));
    assert_eq!(captures[2], Some(8..10));
    assert_eq!(captures[3], Some(11..13));
    assert_eq!(captures[4], Some(14..16));
    assert_eq!(captures[5], Some(17..19));
    assert_eq!(captures[6], Some(20..25));
}

#[test]
fn test_subsequent_captures_engine() {
    assert_match(
        "(%d%d%d%d)%-(%d%d)%-(%d%d)",
        "2023-04-15",
        0..10,
        &[Some(0..4), Some(5..7), Some(8..10)],
    );

    assert_match(
        "(%d+)_(%w+)_(%d+)",
        "123_test_456",
        0..12,
        &[Some(0..3), Some(4..8), Some(9..12)],
    );
}
