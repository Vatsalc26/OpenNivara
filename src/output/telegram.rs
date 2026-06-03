use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Converts standard Markdown text into Telegram-safe HTML.
/// Only outputs tags allowed by Telegram: <b>, <i>, <code>, <pre><code>, <blockquote>, and <a>.
/// Safely escapes all text using html_escape before injecting it into the HTML structure.
pub fn markdown_to_telegram_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html = String::new();
    let mut list_counters: Vec<usize> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { .. } => html.push_str("<b>"),
                Tag::Paragraph => {}
                Tag::BlockQuote(_) => html.push_str("<blockquote>"),
                Tag::CodeBlock(_) => {
                    html.push_str("<pre><code>");
                }
                Tag::List(start_number) => {
                    let start = start_number.unwrap_or(0) as usize;
                    list_counters.push(start);
                }
                Tag::Item => {
                    if let Some(counter) = list_counters.last_mut() {
                        if *counter > 0 {
                            html.push_str(&format!(" {}. ", counter));
                            *counter += 1;
                        } else {
                            html.push_str(" • ");
                        }
                    } else {
                        html.push_str(" • ");
                    }
                }
                Tag::Strong => html.push_str("<b>"),
                Tag::Emphasis => html.push_str("<i>"),
                Tag::Link { dest_url, .. } => {
                    let url = dest_url.as_ref();
                    if url.starts_with("http://") || url.starts_with("https://") {
                        html.push_str(&format!("<a href=\"{}\">", html_escape::encode_text(url)));
                    } else {
                        html.push_str("<a>");
                    }
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => html.push_str("</b>\n"),
                TagEnd::Paragraph => html.push_str("\n\n"),
                TagEnd::BlockQuote(_) => html.push_str("</blockquote>\n"),
                TagEnd::CodeBlock => {
                    html.push_str("</code></pre>\n");
                }
                TagEnd::List(_) => {
                    list_counters.pop();
                    html.push('\n');
                }
                TagEnd::Item => html.push('\n'),
                TagEnd::Strong => html.push_str("</b>"),
                TagEnd::Emphasis => html.push_str("</i>"),
                TagEnd::Link => html.push_str("</a>"),
                _ => {}
            },
            Event::Text(text) => {
                // Safely escape all textual content
                html.push_str(&html_escape::encode_text(&text));
            }
            Event::Code(code) => {
                html.push_str("<code>");
                html.push_str(&html_escape::encode_text(&code));
                html.push_str("</code>");
            }
            Event::SoftBreak | Event::HardBreak => {
                html.push('\n');
            }
            _ => {}
        }
    }

    html.trim().to_string()
}

/// Splits a Telegram HTML message into multiple parts of max_chars length.
/// Splits on double newlines to preserve HTML structures and avoid cut-offs within tags.
pub fn split_telegram_html_message(html: &str, max_chars: usize) -> Vec<String> {
    if html.len() <= max_chars {
        return vec![html.to_string()];
    }

    let mut parts = Vec::new();
    let mut current_part = String::new();

    // Split on paragraph boundaries
    for block in html.split("\n\n") {
        let trimmed_block = block.trim();
        if trimmed_block.is_empty() {
            continue;
        }

        if current_part.len() + trimmed_block.len() + 2 <= max_chars {
            if !current_part.is_empty() {
                current_part.push_str("\n\n");
            }
            current_part.push_str(trimmed_block);
        } else {
            if !current_part.is_empty() {
                parts.push(current_part.clone());
                current_part.clear();
            }

            if trimmed_block.len() > max_chars {
                // If a single paragraph is too long, strip tags and send as safe plain escaped chunks
                let plain_block = strip_basic_html(trimmed_block);
                let plain_escaped = html_escape::encode_text(&plain_block).to_string();
                let mut char_iter = plain_escaped.chars();
                loop {
                    let chunk: String = char_iter.by_ref().take(max_chars).collect();
                    if chunk.is_empty() {
                        break;
                    }
                    parts.push(chunk);
                }
            } else {
                current_part.push_str(trimmed_block);
            }
        }
    }

    if !current_part.is_empty() {
        parts.push(current_part);
    }

    parts
}

/// Helper that strips basic HTML tags (<...>) and decodes HTML entities back to raw text.
/// Serves as a robust fallback if Telegram's HTML Parse Mode encounters errors.
pub fn strip_basic_html(html: &str) -> String {
    let mut plain = String::new();
    let mut inside_tag = false;

    for c in html.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            plain.push(c);
        }
    }

    html_escape::decode_html_entities(&plain).to_string()
}
