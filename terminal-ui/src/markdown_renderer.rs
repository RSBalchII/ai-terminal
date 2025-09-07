//! Markdown renderer for the AI Terminal UI
//! 
//! This module provides functionality to parse Markdown text and render it
//! as styled text using ratatui widgets.

use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind};
use ratatui::text::{Line, Span};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::LinesWithEndings;
use syntect::easy::HighlightLines;
use std::borrow::Cow;

/// Renders Markdown text as styled ratatui text
/// 
/// # Arguments
/// 
/// * `markdown` - The Markdown text to render
/// 
/// # Returns
/// 
/// A vector of ratatui Lines representing the rendered Markdown
pub fn render_markdown(markdown: &str) -> Vec<ratatui::text::Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let parser = Parser::new(markdown);
    
    let mut current_spans: Vec<Span> = Vec::new();
    let mut in_code_block = false;
    let mut code_language = String::new();
    let mut code_block_content = String::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    // Start of a new paragraph
                    current_spans.clear();
                },
                Tag::Heading(level, _, _) => {
                    // Handle heading styling
                    let prefix = match level {
                        pulldown_cmark::HeadingLevel::H1 => "██ ",
                        pulldown_cmark::HeadingLevel::H2 => "▓▓▓ ",
                        pulldown_cmark::HeadingLevel::H3 => "▒▒▒▒ ",
                        _ => "░░░░░ ",
                    };
                    current_spans.push(Span::styled(prefix, ratatui::style::Style::default().fg(ratatui::style::Color::Blue)));
                },
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_language = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                    code_block_content.clear();
                },
                Tag::Emphasis => {
                    // Handle emphasis styling
                },
                Tag::Strong => {
                    // Handle strong styling
                },
                _ => {}
            },
            Event::End(tag) => match tag {
                Tag::Paragraph => {
                    // End of paragraph, add to lines
                    if !current_spans.is_empty() {
                        lines.push(Line::from(current_spans.clone()));
                        current_spans.clear();
                    }
                },
                Tag::CodeBlock(_) => {
                    in_code_block = false;
                    // Render code block with syntax highlighting
                    let highlighted_lines = highlight_code(&code_block_content, &code_language);
                    lines.extend(highlighted_lines);
                },
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else {
                    current_spans.push(Span::raw(text.to_string()));
                }
            },
            Event::Code(code) => {
                // Handle inline code
                current_spans.push(Span::styled(code.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)));
            },
            _ => {}
        }
    }
    
    lines
}

/// Highlights code syntax using syntect
/// 
/// # Arguments
/// 
/// * `code` - The code to highlight
/// * `language` - The language of the code (e.g., "rust", "python")
/// 
/// # Returns
/// 
/// A vector of ratatui Lines with syntax highlighting applied
fn highlight_code(code: &str, language: &str) -> Vec<ratatui::text::Line<'static>> {
    // Load syntax and theme sets
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    // Find syntax for the language or use plain text as fallback
    let syntax = ss.find_syntax_by_token(language)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    
    // Create highlighter with a dark theme
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line> = Vec::new();
    
    // Process each line of the code
    for line in LinesWithEndings::from(code) {
        // Highlight the line
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ss).unwrap();
        let mut spans: Vec<Span> = Vec::new();
        
        // Convert highlighted ranges to ratatui spans
        for (style, text) in ranges {
            // Convert syntect color to ratatui color
            let fg_color = ratatui::style::Color::Rgb(
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
            );
            
            // Create span with the appropriate styling
            spans.push(Span::styled(
                Cow::Owned(text.to_string()), 
                ratatui::style::Style::default().fg(fg_color)
            ));
        }
        
        // Add the line to our result
        lines.push(Line::from(spans));
    }
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let markdown = "# Heading\n\nThis is a paragraph.";
        let lines = render_markdown(markdown);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_highlight_code() {
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let lines = highlight_code(code, "rust");
        assert!(!lines.is_empty());
    }
}