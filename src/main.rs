use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use noise::{NoiseFn, Perlin};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
use std::fs::File;
use std::io;

const MAP_WIDTH: usize = 200;
const MAP_HEIGHT: usize = 200;
const RULLER_LEFT_SIZE: usize = 4;
const RULLER_UP_SIZE: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(
        vec![
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create("gamelive.log").unwrap(),
            ),
        ]
    ).unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut noise_map = empty_map();

    let mut camera_x = 0;
    let mut camera_y = 0;
    let mut term_width = 1;
    let mut term_height = 1;
    let mut show_ruller = true;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            term_width = area.width as usize;
            term_height = area.height as usize;

            let map_str = render_map(&noise_map, camera_x, camera_y, term_width, term_height, show_ruller);

            let paragraph = Paragraph::new(map_str).block(Block::default());

            f.render_widget(paragraph, area);
        })?;

        let half_height = term_height / 2;

        let (width, height) = if show_ruller {
            (MAP_WIDTH.saturating_sub(term_width - RULLER_LEFT_SIZE),
            MAP_HEIGHT.saturating_sub(term_height - RULLER_UP_SIZE))
        } else {
            (MAP_WIDTH.saturating_sub(term_width),
            MAP_HEIGHT.saturating_sub(term_height))
        };

        if camera_y > height { camera_y = height }
        if camera_x > width { camera_x = width }

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('d') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                if camera_y < height - half_height {
                                    camera_y += half_height;
                                } else if camera_y < height {
                                    camera_y = height;
                                }
                            }
                        }
                        KeyCode::Char('u') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                if camera_y > half_height {
                                    camera_y -= half_height;
                                } else if camera_y > 0 {
                                    camera_y = 0;
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                show_ruller = !show_ruller;
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if camera_x > 0 {
                                camera_x -= 1;
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if camera_x < width {
                                camera_x += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if camera_y > 0 {
                                camera_y -= 1;
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if camera_y < height {
                                camera_y += 1;
                            }
                        }
                        _ => {}
                    }
                },
                Event::Mouse(mouse_event) => {
                    //match mouse_event.kind {
                    //    MouseEventKind::Down(_) => {
                    //        handle_mouse_click(
                    //            mouse_event.column,
                    //            mouse_event.row,
                    //            &mut noise_map,
                    //            camera_x,
                    //            camera_y,
                    //            width,
                    //            height,
                    //        );
                    //    },
                    //    _ => {}
                    //}

                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}

fn handle_mouse_click(
    mouse_x: u16,
    mouse_y: u16,
    map: &mut Vec<Vec<f64>>,
    camera_x: usize,
    camera_y: usize,
    width: usize,
    height: usize,
) {
    // Adjust mouse coordinates to map indices
    let map_x = mouse_x as usize - 4 + camera_x; // Subtract ruler width and add camera offset
    let map_y = mouse_y as usize - 1 + camera_y; // Subtract ruler height and add camera offset

    // Check if the click is within the map area
    if map_x < MAP_WIDTH && map_y < MAP_HEIGHT {
        // Modify the map data at the clicked position
        map[map_y][map_x] = 1.0; // For example, set to maximum value (mountain)
    }
}

fn generate_noise_map() -> Vec<Vec<f64>> {
    let perlin = Perlin::new(10);
    let mut map = vec![vec![0.0; MAP_WIDTH]; MAP_HEIGHT];

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let nx = x as f64 / MAP_WIDTH as f64;
            let ny = y as f64 / MAP_HEIGHT as f64;
            let noise_value = perlin.get([nx * 10.0, ny * 10.0]);
            map[y][x] = noise_value;
        }
    }

    map
}

fn empty_map() -> Vec<Vec<f64>> {
    let mut map = vec![vec![0.; MAP_WIDTH]; MAP_HEIGHT];

    map[0][0] = 1.;
    map[MAP_WIDTH - 1][MAP_HEIGHT - 1] = 1.;
    map[MAP_WIDTH - 5][MAP_HEIGHT - 2] = 1.;
    map[MAP_WIDTH - 10][MAP_HEIGHT - 10] = 1.;

    map
}

fn get_char_for_value(value: f64) -> char {
    match value {
        v if v <= 0. => '░', // Deep water
        _ => '█',             // Mountain
    }
}

fn render_map(
    map: &Vec<Vec<f64>>,
    camera_x: usize,
    camera_y: usize,
    width: usize,
    height: usize,
    show_ruller: bool,
) -> String {
    let mut visible_map = String::new();

    // Adjust width and height to account for rulers
    let (map_width, map_height) = if show_ruller {
        (width - RULLER_LEFT_SIZE, height - RULLER_UP_SIZE)
    } else {
        (width, height)
    };

    // Top ruler (X-axis)
    if show_ruller {
        visible_map.push_str(" ".repeat(RULLER_LEFT_SIZE).as_str()); // Space for Y-axis labels
        for x in 0..map_width {
            let map_x = x + camera_x;
            if map_x % 10 == 0 {
                let label = format!("{:>2}", map_x % 100);
                visible_map.push_str(&label);
            } else {
                visible_map.push_str("  ");
            }
        }
        visible_map.push('\n');
    }

    for y in 0..map_height {
        let map_y = y + camera_y;

        // Left ruler (Y-axis)
        if show_ruller {
            if map_y % 5 == 0 {
                let label = format!("{:>3} ", map_y % 100);
                visible_map.push_str(&label);
            } else {
                visible_map.push_str(" ".repeat(RULLER_LEFT_SIZE).as_str());
            }
        }

        for x in 0..map_width {
            let map_x = x + camera_x;

            if map_y < MAP_HEIGHT && map_x < MAP_WIDTH {
                let value = map[map_y][map_x];
                let ch = get_char_for_value(value);
                visible_map.push(ch);
            } else {
                visible_map.push(' ');
            }
        }
        visible_map.push('\n');
    }

    visible_map
}
