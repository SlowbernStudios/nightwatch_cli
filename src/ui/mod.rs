use crate::{
    app::{App, Command, Overlay},
    editor::Mode,
    engine::world::Point,
    plain,
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

fn accent(no_color: bool) -> Style {
    if no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
    }
}
pub fn draw(frame: &mut Frame, app: &App, no_color: bool) {
    let area = frame.area();
    if area.width < 60 || area.height < 22 {
        frame.render_widget(Paragraph::new("NIGHTWATCH\nPlease resize to at least 60 x 22 cells.\nYour shift is paused.\nCtrl+C, Enter: quit").wrap(Wrap{trim:false}),area);
        return;
    }
    let s = app.session().view();
    let width = 56;
    let x = area.x + (area.width - width) / 2;
    frame.render_widget(
        Paragraph::new(format!(
            "NIGHTWATCH  /  Shift {}  /  Seed {}",
            s.game.shift_number + 1,
            s.game.seed
        ))
        .centered()
        .style(accent(no_color)),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let card = Rect::new(x, area.y + 1, width, 7);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" TASK ")
            .border_style(accent(no_color)),
        card,
    );
    frame.render_widget(
        Paragraph::new(app.task())
            .wrap(Wrap { trim: false })
            .style(accent(no_color)),
        Rect::new(x + 2, area.y + 2, width - 4, 2),
    );
    frame.render_widget(
        Paragraph::new(app.destination()),
        Rect::new(x + 2, area.y + 4, width - 4, 1),
    );
    frame.render_widget(
        Paragraph::new(app.hint()).wrap(Wrap { trim: false }),
        Rect::new(x + 2, area.y + 5, width - 4, 2),
    );
    let map = Rect::new(area.x, area.y + 8, area.width, area.height - 12);
    if let Some(c) = &s.console {
        frame.render_widget(
            Paragraph::new(app.session().console_lesson().unwrap().title)
                .centered()
                .style(accent(no_color)),
            Rect::new(map.x, map.y, map.width, 1),
        );
        let body = Rect::new(map.x + 2, map.y + 1, map.width - 4, map.height - 1);
        let rows = body.height as usize;
        let columns = body.width.saturating_sub(5) as usize;
        let top = c.editor.row.saturating_sub(rows.saturating_sub(1));
        let left = c.editor.col.saturating_sub(columns.saturating_sub(1));
        for (i, line) in c.editor.lines.iter().enumerate().skip(top).take(rows) {
            let y = body.y + (i - top) as u16;
            let mut chars = vec![Span::styled(
                format!("{:>2} | ", i + 1),
                Style::default().add_modifier(Modifier::DIM),
            )];
            for col in left..left + columns {
                let ch = line.as_bytes().get(col).copied().unwrap_or(b' ') as char;
                let style = if i == c.editor.row && col == c.editor.col {
                    accent(no_color).add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                chars.push(Span::styled(ch.to_string(), style));
            }
            frame.render_widget(
                Paragraph::new(Line::from(chars)),
                Rect::new(body.x, y, body.width, 1),
            );
        }
        if app.overlay().is_none() {
            frame.set_cursor_position((
                body.x + 5 + (c.editor.col - left) as u16,
                body.y + (c.editor.row - top) as u16,
            ));
        }
    } else {
        let center = s.game.player;
        let target = app
            .session()
            .game()
            .world()
            .view()
            .stations
            .iter()
            .find(|p| p.id == app.session().game().required())
            .map(|s| s.position);
        for dy in 0..map.height {
            let mut line = Vec::with_capacity(map.width as usize);
            for dx in 0..map.width {
                let p = Point {
                    x: center.x + i32::from(dx) - i32::from(map.width / 2),
                    y: center.y + i32::from(dy) - i32::from(map.height / 2),
                };
                let glyph = plain::glyph(app, p);
                let visible = app.session().game().visible().contains(&p);
                let mut style = Style::default();
                if !visible {
                    style = style.add_modifier(Modifier::DIM);
                }
                if glyph == '@'
                    || (target == Some(p) && visible)
                    || app.session().checkpoint() == Some(p)
                {
                    style = accent(no_color).add_modifier(Modifier::REVERSED);
                } else if !no_color {
                    style = style.fg(match glyph {
                        'f' => Color::LightMagenta,
                        '~' => Color::LightBlue,
                        '!' | '%' => Color::LightGreen,
                        '.' => Color::DarkGray,
                        _ => Color::Gray,
                    });
                }
                line.push(Span::styled(glyph.to_string(), style));
            }
            frame.render_widget(
                Paragraph::new(Line::from(line)),
                Rect::new(map.x, map.y + dy, map.width, 1),
            );
        }
    }
    let status = if let Some(c) = &s.console {
        format!(
            "{} | pending: {} | line {} col {} | {}",
            if c.editor.mode == Mode::Insert {
                "INSERT"
            } else {
                "NORMAL"
            },
            c.editor.pending,
            c.editor.row + 1,
            c.editor.col + 1,
            if c.assisted { "assisted" } else { "unassisted" }
        )
    } else {
        format!(
            "NORMAL | pending: {} | turn {} | Tab: game actions",
            s.pending, s.game.turn
        )
    };
    frame.render_widget(
        Paragraph::new(status).style(accent(no_color)),
        Rect::new(area.x + 1, area.y + area.height - 4, area.width - 2, 1),
    );
    frame.render_widget(
        Paragraph::new(app.feedback()).wrap(Wrap { trim: false }),
        Rect::new(area.x + 1, area.y + area.height - 3, area.width - 2, 3),
    );
    if let Some(overlay) = app.overlay() {
        let popup = Rect::new(area.x + 2, area.y + 3, area.width - 4, area.height - 6);
        frame.render_widget(Clear, popup);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(match overlay {
                Overlay::Actions => " Game actions ",
                Overlay::Quit => " Clocking out ",
                Overlay::Page(_) => " Company paperwork ",
            })
            .border_style(accent(no_color));
        let inner = block.inner(popup);
        frame.render_widget(block, popup);
        match overlay {
            Overlay::Actions=>{
                let capacity=inner.height.saturating_sub(1) as usize;let start=app.selection().saturating_sub(capacity.saturating_sub(1));
                for (index,c) in Command::ALL.into_iter().enumerate().skip(start).take(capacity) {
                    let selected=index==app.selection();let line=format!("{} {}",if selected {">"} else {" "},c.label());
                    frame.render_widget(Paragraph::new(line).style(if selected {accent(no_color).add_modifier(Modifier::REVERSED)} else {Style::default()}),Rect::new(inner.x,inner.y+(index-start) as u16,inner.width,1));
                }
                frame.render_widget(Paragraph::new("j/k or arrows: choose | Enter: select | Esc: close"),Rect::new(inner.x,inner.bottom()-1,inner.width,1));
            },
            Overlay::Quit=>frame.render_widget(Paragraph::new("Quit without saving?\n\nEnter confirms. Any other key returns to your shift.\nSave is available from Tab.").wrap(Wrap{trim:false}),inner),
            Overlay::Page(_)=>frame.render_widget(Paragraph::new(app.page()).wrap(Wrap{trim:false}).scroll((app.scroll().min(u16::MAX as usize) as u16,0)),inner),
        }
    }
}
