use std::{fmt::Debug, marker::PhantomData};

use clashctl_core::model::ProxyType;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};
use unicode_width::UnicodeWidthStr;

use crate::ui::{
    components::{Consts, ProxyItem},
    utils::{fit_to_width, get_text_style, spans_window_owned},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProxyGroup<'a> {
    pub(super) name: String,
    pub(super) proxy_type: ProxyType,
    pub(super) members: Vec<ProxyItem>,
    pub(super) current: Option<usize>,
    pub(super) cursor: usize,
    pub(super) _life: PhantomData<&'a ()>,
}

pub enum ProxyGroupFocusStatus {
    None,
    Focused,
    Expanded,
}

impl<'a> ProxyGroup<'a> {
    pub fn proxy_type(&self) -> ProxyType {
        self.proxy_type
    }

    pub fn members(&self) -> &Vec<ProxyItem> {
        &self.members
    }

    pub fn get_summary_widget(&self) -> impl Iterator<Item = Span> {
        self.members.iter().map(|x| {
            if x.proxy_type.is_normal() {
                match x.history {
                    Some(ref history) => Self::get_delay_span(history.delay),
                    None => Consts::NO_LATENCY_SPAN,
                }
            } else {
                Consts::NOT_PROXY_SPAN
            }
        })
    }

    pub fn get_widget(&'a self, width: usize, status: ProxyGroupFocusStatus) -> Vec<Spans<'a>> {
        let delimiter = Span::raw(" ");
        let prefix = if matches!(status, ProxyGroupFocusStatus::Focused) {
            Consts::FOCUSED_INDICATOR_SPAN
        } else {
            Consts::UNFOCUSED_INDICATOR_SPAN
        };
        let name = Span::styled(
            &self.name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        let proxy_type = Span::styled(self.proxy_type.to_string(), Consts::PROXY_TYPE_STYLE);

        let count = self.members.len();
        let proxy_count = Span::styled(
            if matches!(status, ProxyGroupFocusStatus::Expanded) {
                let position = if self.members.is_empty() {
                    0
                } else {
                    self.cursor + 1
                };
                format!("{}/{}", position, count)
            } else {
                count.to_string()
            },
            Style::default().fg(Color::Green),
        );

        let mut ret = Vec::with_capacity(if matches!(status, ProxyGroupFocusStatus::Expanded) {
            self.members.len() + 1
        } else {
            2
        });

        ret.push(spans_window_owned(
            Spans::from(vec![
                prefix.clone(),
                name,
                delimiter.clone(),
                proxy_type,
                delimiter,
                proxy_count,
            ]),
            &(0..width),
        ));

        if matches!(status, ProxyGroupFocusStatus::Expanded) {
            let skipped = self.cursor.saturating_sub(4);
            let text_style = get_text_style();
            let delay = |item: &ProxyItem| match item.history.as_ref() {
                Some(history) if history.delay > 0 => (
                    history.delay.to_string(),
                    Self::get_delay_style(history.delay),
                ),
                Some(_) | None if item.proxy_type.is_normal() => {
                    (Consts::NO_LATENCY_SIGN.to_owned(), Consts::NO_LATENCY_STYLE)
                }
                _ => (String::new(), Consts::NO_LATENCY_STYLE),
            };
            let type_width = self
                .members
                .iter()
                .map(|item| UnicodeWidthStr::width(item.proxy_type.to_string().as_str()))
                .max()
                .unwrap_or_default();
            let delay_width = self
                .members
                .iter()
                .map(|item| delay(item).0)
                .map(|delay| UnicodeWidthStr::width(delay.as_str()))
                .max()
                .unwrap_or_default();
            let max_name_width = self
                .members
                .iter()
                .map(|item| UnicodeWidthStr::width(item.name.as_str()))
                .max()
                .unwrap_or_default();
            let indicator_width = Consts::EXPANDED_INDICATOR_SPAN
                .width()
                .max(Consts::EXPANDED_FOCUSED_INDICATOR_SPAN.width());
            let delimiter_width = Consts::DELIMITER_SPAN.width();
            let fixed_width = indicator_width
                .saturating_add(delimiter_width.saturating_mul(3))
                .saturating_add(type_width)
                .saturating_add(delay_width);
            let name_width = max_name_width.min(width.saturating_sub(fixed_width));
            let is_current =
                |index: usize| self.current.map(|x| x == index + skipped).unwrap_or(false);
            let is_pointed = |index: usize| self.cursor == index + skipped;

            let lines = self.members.iter().skip(skipped).enumerate().map(|(i, x)| {
                let prefix = if self.cursor == i + skipped {
                    Consts::EXPANDED_FOCUSED_INDICATOR_SPAN
                } else {
                    Consts::EXPANDED_INDICATOR_SPAN
                };
                let name = Span::styled(
                    fit_to_width(&x.name, name_width),
                    if is_current(i) {
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else if is_pointed(i) {
                        text_style.fg(Color::LightBlue)
                    } else {
                        text_style
                    },
                );
                let proxy_type = Span::styled(
                    fit_to_width(&x.proxy_type.to_string(), type_width),
                    Consts::PROXY_TYPE_STYLE,
                );

                let (delay_text, delay_style) = delay(x);
                let delay_span = Span::styled(fit_to_width(&delay_text, delay_width), delay_style);
                spans_window_owned(
                    vec![
                        prefix,
                        Consts::DELIMITER_SPAN.clone(),
                        name,
                        Consts::DELIMITER_SPAN.clone(),
                        proxy_type,
                        Consts::DELIMITER_SPAN.clone(),
                        delay_span,
                    ]
                    .into(),
                    &(0..width),
                )
            });
            ret.extend(lines);
        } else {
            ret.extend(
                self.get_summary_widget()
                    .collect::<Vec<_>>()
                    .chunks(
                        width
                            .saturating_sub(Consts::FOCUSED_INDICATOR_SPAN.width() + 2)
                            .saturating_div(2)
                            .max(1),
                    )
                    .map(|x| {
                        std::iter::once(if matches!(status, ProxyGroupFocusStatus::Focused) {
                            Consts::FOCUSED_INDICATOR_SPAN
                        } else {
                            Consts::UNFOCUSED_INDICATOR_SPAN
                        })
                        .chain(x.to_owned())
                        .collect::<Vec<_>>()
                        .into()
                    }),
            )
        }

        ret
    }

    fn get_delay_style(delay: u64) -> Style {
        match delay {
            0 => Consts::NO_LATENCY_STYLE,
            1..=200 => Consts::LOW_LATENCY_STYLE,
            201..=400 => Consts::MID_LATENCY_STYLE,
            401.. => Consts::HIGH_LATENCY_STYLE,
        }
    }

    fn get_delay_span(delay: u64) -> Span<'static> {
        match delay {
            0 => Consts::NO_LATENCY_SPAN,
            1..=200 => Consts::LOW_LATENCY_SPAN,
            201..=400 => Consts::MID_LATENCY_SPAN,
            401.. => Consts::HIGH_LATENCY_SPAN,
        }
    }
}

impl<'a> Default for ProxyGroup<'a> {
    fn default() -> Self {
        Self {
            members: vec![],
            current: None,
            proxy_type: ProxyType::Selector,
            name: String::new(),
            cursor: 0,
            _life: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, proxy_type: ProxyType) -> ProxyItem {
        ProxyItem {
            name: name.to_owned(),
            proxy_type,
            history: None,
            udp: None,
            now: None,
        }
    }

    #[test]
    fn expanded_rows_align_and_clip_unicode_names() {
        let group = ProxyGroup {
            name: "America".to_owned(),
            proxy_type: ProxyType::Selector,
            members: vec![
                item("🇺🇸美国_超长节点_BGP_E", ProxyType::AnyTLS),
                item("e\u{301}短节点", ProxyType::Trojan),
                item("🇺🇸美国 | 011", ProxyType::Unknown),
            ],
            current: Some(0),
            cursor: 0,
            _life: PhantomData,
        };
        let width = 32;
        let rows = group.get_widget(width, ProxyGroupFocusStatus::Expanded);

        assert!(rows.iter().all(|row| row.width() <= width));

        let rendered = rows[1]
            .0
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(rendered.contains("🇺🇸"));
        assert!(rendered.contains('…'));

        let type_columns = rows[1..]
            .iter()
            .map(|row| {
                let type_index = row
                    .0
                    .iter()
                    .position(|span| matches!(span.content.trim(), "AnyTLS" | "Trojan" | "Unknown"))
                    .unwrap();
                row.0[..type_index].iter().map(Span::width).sum::<usize>()
            })
            .collect::<Vec<_>>();
        assert!(type_columns.windows(2).all(|pair| pair[0] == pair[1]));

        let row_widths = rows[1..].iter().map(Spans::width).collect::<Vec<_>>();
        assert!(row_widths.windows(2).all(|pair| pair[0] == pair[1]));

        let wide_rows = group.get_widget(180, ProxyGroupFocusStatus::Expanded);
        let type_column = wide_rows[1]
            .0
            .iter()
            .position(|span| span.content.trim() == "AnyTLS")
            .map(|index| {
                wide_rows[1].0[..index]
                    .iter()
                    .map(Span::width)
                    .sum::<usize>()
            })
            .unwrap();
        assert!(type_column < 50);
    }
}
