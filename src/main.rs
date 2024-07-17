use std::sync::mpsc::Receiver;

use color_eyre::{
    eyre::WrapErr,
    Result,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols::{self, border},
    text::{Line, Text},
    widgets::{
        block::{Position, Title}, Axis, Block, Chart, Dataset, GraphType, Widget
    },
    Frame,
};
use vec2::Vec2;

mod errors;
mod tui;
mod events;
mod vec2;

#[derive(Clone, Copy, Debug, Default)]
pub struct Planet {
    pos: Vec2,
    vel: Vec2,
}

#[derive(Debug)]
pub struct App {
    rx: Receiver<events::Event>,
    planets: Vec<Planet>,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {


        match self.rx.recv()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            events::Event::Console(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            events::Event::DrawInterrupt => {
                self.run_physics()
            }
            _ => Ok(()),
        }?;

        Ok(())
    }

    fn run_physics(&mut self) -> Result<()> {
        for i1 in 0..self.planets.len() {
            let mut acc = Vec2::default();
            for i2 in 0..self.planets.len() {
                if i1 == i2 {
                    continue;
                }

                let direction = (self.planets[i2].pos - self.planets[i1].pos).normalized();
                let dist = direction.dot(&direction);
                let force = dist.powi(-2) * 0.01;
                acc += direction * force;
            }

            self.planets[i1].vel += acc;
            self.planets[i1].pos = self.planets[i1].pos + self.planets[i1].vel;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let positions = self.planets.iter()
            .map(|planet| (planet.pos.x, planet.pos.y))
            .collect::<Vec<_>>();

        let datasets = vec![
            // Scatter chart
            Dataset::default()
                .name("planets")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().cyan())
                .data(&positions),
        ];

        let title = Title::from(" Gravity ".bold());
        let instructions = Title::from(Line::from(vec![
            // " Decrement ".into(),
            // "<Left>".blue().bold(),
            // " Increment ".into(),
            // "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .title("X Axis".red())
            .style(Style::default().white())
            .bounds([-10.0, 10.0])
            .labels(vec!["-10.0".into(), "0.0".into(), "10.0".into()]);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title("Y Axis".red())
            .style(Style::default().white())
            .bounds([-10.0, 10.0])
            .labels(vec!["-10.0".into(), "0.0".into(), "10.0".into()]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(Block::new().title("Planets"))
            .x_axis(x_axis)
            .y_axis(y_axis);

        // Paragraph::new(chart)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);
        
        let inner_area = block.inner(area);
        block.render(area, buf);

        chart.render(inner_area, buf);
    }
}

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    let mut app = App {
        exit: false,
        rx: events::spawn_event_threads().1,
        planets: Vec::new(),
    };
    app.planets.push(Planet { pos: Vec2 {x:  3.0, y:  4.0 }, vel: Vec2 { x: 0.0, y: 0.0 }});
    app.planets.push(Planet { pos: Vec2 {x: -3.0, y:  4.0 }, vel: Vec2 { x: 0.0, y: 0.0 }});
    app.planets.push(Planet { pos: Vec2 {x: -3.0, y: -4.0 }, vel: Vec2 { x: 0.0, y: 0.0 }});
    app.planets.push(Planet { pos: Vec2 {x:  3.0, y: -4.0 }, vel: Vec2 { x: 0.0, y: 0.0 }});
    app.run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ratatui::style::Style;

//     #[test]
//     fn render() {
//         let app = App::default();
//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

//         app.render(buf.area, &mut buf);

//         let mut expected = Buffer::with_lines(vec![
//             "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//             "┃                    Value: 0                    ┃",
//             "┃                                                ┃",
//             "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//         ]);
//         let title_style = Style::new().bold();
//         let counter_style = Style::new().yellow();
//         let key_style = Style::new().blue().bold();
//         expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//         expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//         expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//         expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//         expected.set_style(Rect::new(43, 3, 4, 1), key_style);

//         // note ratatui also has an assert_buffer_eq! macro that can be used to
//         // compare buffers and display the differences in a more readable way
//         assert_eq!(buf, expected);
//     }

//     #[test]
//     fn handle_key_event() {
//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Char('q').into()).unwrap();
//         assert_eq!(app.exit, true);
//     }

//     #[test]
//     #[should_panic(expected = "attempt to subtract with overflow")]
//     fn handle_key_event_panic() {
//         let mut app = App::default();
//         let _ = app.handle_key_event(KeyCode::Left.into());
//     }

//     #[test]
//     fn handle_key_event_overflow() {
//         let mut app = App::default();
//         assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//         assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//         assert_eq!(
//             app.handle_key_event(KeyCode::Right.into())
//                 .unwrap_err()
//                 .to_string(),
//             "counter overflow"
//         );
//     }
// }