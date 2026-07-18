use std::{borrow::Cow, ops::Range};

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{IntoSpans, Wrap};

pub fn help_footer(content: &str, normal: Style, highlight: Style) -> Spans {
    if content.is_empty() {
        Spans(vec![])
    } else if content.len() == 1 {
        Spans(vec![Span::raw(content)])
    } else {
        let (index, _) = content.char_indices().nth(1).unwrap();
        let (first_char, rest) = content.split_at(index);
        Spans(vec![
            Span::styled("[", normal),
            Span::styled(first_char, highlight),
            Span::styled("]", normal),
            Span::styled(rest, normal),
        ])
    }
}

pub fn tagged_footer<T: ToString>(label: &str, style: Style, content: T) -> Spans {
    let mut ret = help_footer(label, style, style.add_modifier(Modifier::BOLD)).wrapped();
    ret.0.push(Span::styled(
        content.to_string().wrapped(),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::REVERSED),
    ));
    ret
}

/// Return the complete graphemes contained in a terminal-column range.
pub fn string_window<'a>(string: &'a str, range: &Range<usize>) -> Cow<'a, str> {
    if range.start >= range.end {
        return Cow::Borrowed("");
    }

    if range.start == 0 && UnicodeWidthStr::width(string) <= range.end {
        return Cow::Borrowed(string);
    }

    let mut column: usize = 0;
    let mut result = String::new();

    for grapheme in UnicodeSegmentation::graphemes(string, true) {
        let width = UnicodeWidthStr::width(grapheme);
        let next_column = column.saturating_add(width);

        if width == 0 {
            if range.contains(&column) {
                result.push_str(grapheme);
            }
            continue;
        }

        if next_column <= range.start {
            column = next_column;
            continue;
        }

        // Never render half of a wide grapheme when the viewport starts in
        // the middle of it.
        if column < range.start {
            column = next_column;
            continue;
        }

        if next_column > range.end {
            break;
        }

        result.push_str(grapheme);
        column = next_column;
    }

    Cow::Owned(result)
}

pub fn string_window_owned(string: String, range: &Range<usize>) -> String {
    string_window(&string, range).into_owned()
}

/// Truncate a string to a terminal-cell width without splitting a grapheme.
pub fn truncate_to_width<'a>(string: &'a str, width: usize) -> Cow<'a, str> {
    if UnicodeWidthStr::width(string) <= width {
        return Cow::Borrowed(string);
    }

    if width == 0 {
        return Cow::Borrowed("");
    }

    const ELLIPSIS: &str = "…";
    let content_width = width.saturating_sub(UnicodeWidthStr::width(ELLIPSIS));
    let mut current_width = 0;
    let mut result = String::new();

    for grapheme in UnicodeSegmentation::graphemes(string, true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if grapheme_width > content_width.saturating_sub(current_width) {
            break;
        }
        result.push_str(grapheme);
        current_width = current_width.saturating_add(grapheme_width);
    }

    result.push_str(ELLIPSIS);
    Cow::Owned(result)
}

/// Truncate and right-pad a string to exactly `width` terminal cells.
pub fn fit_to_width(string: &str, width: usize) -> String {
    let mut result = truncate_to_width(string, width).into_owned();
    let padding = width.saturating_sub(UnicodeWidthStr::width(result.as_str()));
    result.extend(std::iter::repeat(' ').take(padding));
    result
}

/// Crop styled text by terminal columns while preserving grapheme boundaries.
pub fn spans_window<'a>(spans: &'a Spans, range: &Range<usize>) -> Spans<'a> {
    if range.start >= range.end {
        return Spans(vec![]);
    }

    let mut column: usize = 0;
    let mut result = Vec::new();

    for grapheme in spans
        .0
        .iter()
        .flat_map(|span| span.styled_graphemes(Style::default()))
    {
        let width = UnicodeWidthStr::width(grapheme.symbol);
        let next_column = column.saturating_add(width);

        if width == 0 {
            if range.contains(&column) {
                result.push(grapheme);
            }
            continue;
        }

        if next_column <= range.start {
            column = next_column;
            continue;
        }

        if column < range.start {
            column = next_column;
            continue;
        }

        if next_column > range.end {
            break;
        }

        result.push(grapheme);
        column = next_column;
    }

    result.into_spans()
}

pub fn spans_window_owned<'a>(spans: Spans<'a>, range: &Range<usize>) -> Spans<'a> {
    let spans = spans_window(&spans, range)
        .0
        .into_iter()
        .map(|span| Span::styled(span.content.into_owned(), span.style))
        .collect();
    Spans(spans)
}

pub fn get_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightBlue))
        .title(Span::raw(format!(" {} ", title)))
}

pub fn get_focused_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::LightGreen),
        ))
        .style(Style::default().fg(Color::Green))
}

pub fn get_text_style() -> Style {
    Style::default().fg(Color::White)
}

#[test]
fn test_string_window() {
    let test = "A代理相关的 API".to_owned();
    assert_eq!("代理", &string_window(&test, &(1..5)));
    assert_eq!("理相关的 API", &string_window(&test, &(3..114)));
}

#[test]
fn test_unicode_window_does_not_split_graphemes() {
    let flag = "🇺🇸";
    let flag_width = UnicodeWidthStr::width(flag);
    let test = format!("A{flag}e\u{301}中国");

    assert_eq!(flag, &string_window(&test, &(1..1 + flag_width)));
    assert_eq!(
        "e\u{301}",
        &string_window(&test, &(1 + flag_width..2 + flag_width))
    );

    let truncated = truncate_to_width(&test, 6);
    assert!(UnicodeWidthStr::width(truncated.as_ref()) <= 6);
    assert!(truncated.ends_with('…'));
    assert!(!truncated.contains('🇺') || truncated.contains(flag));
}

#[test]
fn test_spans_window_uses_terminal_columns() {
    let blue = Style::default().fg(Color::Blue);
    let red = Style::default().fg(Color::Red);
    let flag = "🇺🇸";
    let flag_width = UnicodeWidthStr::width(flag);
    let spans = Spans(vec![
        Span::styled(format!("A{flag}"), blue),
        Span::styled("代理B", red),
    ]);

    let window = spans_window(&spans, &(1..1 + flag_width + 2));
    assert_eq!(window.0.len(), 2);
    assert_eq!(window.0[0].content, flag);
    assert_eq!(window.0[0].style, blue);
    assert_eq!(window.0[1].content, "代");
    assert_eq!(window.0[1].style, red);
}
