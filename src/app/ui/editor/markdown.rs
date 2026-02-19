use eframe::egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::Editor;

impl Editor {
    pub(super) fn render_markdown_preview(ui: &mut egui::Ui, content: &str) {
        let options = Options::all();
        let parser = Parser::new_ext(content, options);

        let text_color = egui::Color32::from_rgb(220, 220, 220);

        let events: Vec<Event> = parser.collect();
        let mut i = 0;

        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level = *level;
                    i += 1;
                    let mut text = String::new();
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::Heading(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => text.push_str(t),
                            Event::Code(c) => text.push_str(c),
                            Event::SoftBreak => text.push(' '),
                            _ => {}
                        }
                        i += 1;
                    }
                    let size = match level {
                        HeadingLevel::H1 => 28.0,
                        HeadingLevel::H2 => 24.0,
                        HeadingLevel::H3 => 20.0,
                        HeadingLevel::H4 => 18.0,
                        HeadingLevel::H5 => 16.0,
                        HeadingLevel::H6 => 14.0,
                    };
                    let rt = egui::RichText::new(&text)
                        .size(size)
                        .strong()
                        .color(egui::Color32::WHITE);
                    ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                    ui.add_space(4.0);
                }
                Event::Start(Tag::Paragraph) => {
                    i += 1;
                    let mut job = egui::text::LayoutJob::default();
                    let mut is_bold = false;
                    let mut is_italic = false;
                    let mut is_strike = false;
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::Paragraph) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => {
                                let font_id = if is_bold || is_italic {
                                    egui::FontId::new(
                                        14.0,
                                        if is_italic {
                                            egui::FontFamily::Name("Italic".into())
                                        } else {
                                            egui::FontFamily::Proportional
                                        },
                                    )
                                } else {
                                    egui::FontId::proportional(14.0)
                                };
                                job.append(
                                    t,
                                    0.0,
                                    egui::TextFormat {
                                        font_id,
                                        color: text_color,
                                        strikethrough: if is_strike {
                                            egui::Stroke::new(1.5, text_color)
                                        } else {
                                            egui::Stroke::NONE
                                        },
                                        italics: is_italic,
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::Start(Tag::Strong) => is_bold = true,
                            Event::End(TagEnd::Strong) => is_bold = false,
                            Event::Start(Tag::Emphasis) => is_italic = true,
                            Event::End(TagEnd::Emphasis) => is_italic = false,
                            Event::Start(Tag::Strikethrough) => is_strike = true,
                            Event::End(TagEnd::Strikethrough) => is_strike = false,
                            Event::Start(Tag::Link { dest_url, .. }) => {
                                let _url = dest_url.to_string();
                                i += 1;
                                while i < events.len() {
                                    match &events[i] {
                                        Event::Text(t) => {
                                            job.append(
                                                t,
                                                0.0,
                                                egui::TextFormat {
                                                    font_id: egui::FontId::proportional(14.0),
                                                    color: egui::Color32::from_rgb(100, 160, 255),
                                                    underline: egui::Stroke::new(
                                                        1.0,
                                                        egui::Color32::from_rgb(100, 160, 255),
                                                    ),
                                                    ..Default::default()
                                                },
                                            );
                                        }
                                        Event::End(TagEnd::Link) => break,
                                        _ => {}
                                    }
                                    i += 1;
                                }
                            }
                            Event::Code(c) => {
                                job.append(
                                    c,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::monospace(13.0),
                                        color: egui::Color32::from_rgb(230, 180, 100),
                                        background: egui::Color32::from_rgb(50, 55, 65),
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::SoftBreak => {
                                job.append(
                                    " ",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(14.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            Event::HardBreak => {
                                job.append(
                                    "\n",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: egui::FontId::proportional(14.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    job.wrap.max_width = ui.available_width();
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Wrap));
                    ui.add_space(8.0);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    i += 1;
                    let mut code_text = String::new();
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::CodeBlock) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => code_text.push_str(t),
                            _ => {}
                        }
                        i += 1;
                    }
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(30, 33, 40))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            let rt = egui::RichText::new(code_text.trim_end())
                                .family(egui::FontFamily::Monospace)
                                .size(13.0)
                                .color(egui::Color32::from_rgb(180, 210, 170));
                            ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    ui.add_space(4.0);
                }
                Event::Start(Tag::List(start)) => {
                    let mut list_idx = *start;
                    i += 1;
                    while i < events.len() {
                        match &events[i] {
                            Event::End(TagEnd::List(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Start(Tag::Item) => {
                                i += 1;
                                let mut item_text = String::new();
                                let mut depth = 0;
                                while i < events.len() {
                                    match &events[i] {
                                        Event::Start(Tag::Paragraph) if depth == 0 => {
                                            depth += 1;
                                        }
                                        Event::End(TagEnd::Paragraph) if depth > 0 => {
                                            depth -= 1;
                                        }
                                        Event::End(TagEnd::Item) => {
                                            i += 1;
                                            break;
                                        }
                                        Event::Text(t) => item_text.push_str(t),
                                        Event::Code(c) => {
                                            item_text.push('`');
                                            item_text.push_str(c);
                                            item_text.push('`');
                                        }
                                        Event::SoftBreak => item_text.push(' '),
                                        _ => {}
                                    }
                                    i += 1;
                                }
                                let prefix = if let Some(ref mut n) = list_idx {
                                    let p = format!("  {}. ", n);
                                    *n += 1;
                                    p
                                } else {
                                    "  \u{2022} ".to_string()
                                };
                                let rt = egui::RichText::new(format!("{}{}", prefix, item_text))
                                    .size(14.0)
                                    .color(text_color);
                                ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                                continue;
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    ui.add_space(4.0);
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    i += 1;
                    let mut quote_text = String::new();
                    let mut depth = 0;
                    while i < events.len() {
                        match &events[i] {
                            Event::Start(Tag::BlockQuote(_)) => depth += 1,
                            Event::End(TagEnd::BlockQuote(_)) if depth > 0 => depth -= 1,
                            Event::End(TagEnd::BlockQuote(_)) => {
                                i += 1;
                                break;
                            }
                            Event::Text(t) => quote_text.push_str(t),
                            Event::SoftBreak => quote_text.push(' '),
                            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {}
                            _ => {}
                        }
                        i += 1;
                    }
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(50, 55, 65))
                        .inner_margin(egui::Margin {
                            left: 12,
                            right: 8,
                            top: 6,
                            bottom: 6,
                        })
                        .show(ui, |ui| {
                            let rt = egui::RichText::new(&quote_text)
                                .size(14.0)
                                .italics()
                                .color(egui::Color32::from_rgb(180, 180, 190));
                            ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Wrap));
                        });
                    ui.add_space(4.0);
                }
                Event::Rule => {
                    ui.separator();
                    ui.add_space(4.0);
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
}
