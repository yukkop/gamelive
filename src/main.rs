use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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

    let noise_map = generate_noise_map();

    let mut camera_x = 0;
    let mut camera_y = 0;

    loop {
        let mut term_width = 1;
        let mut term_height = 1;
        terminal.draw(|f| {
            let area = f.area();
            term_width = area.width as usize;
            term_height = area.height as usize;

            let map_str = render_map(&noise_map, camera_x, camera_y, term_width, term_height);

            let paragraph = Paragraph::new(map_str).block(Block::default());

            f.render_widget(paragraph, area);
        })?;

        let half_height = term_height / 2;

        let height = MAP_HEIGHT - term_height;
        let width = MAP_HEIGHT - term_width;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
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

fn get_char_for_value(value: f64) -> char {
    match value {
        v if v < -0.5 => '░', // Deep water
        v if v < 0.0 => '▒',  // Shallow water
        v if v < 0.5 => '▓',  // Land
        _ => '█',             // Mountain
    }
}

fn render_map(
    map: &Vec<Vec<f64>>,
    camera_x: usize,
    camera_y: usize,
    width: usize,
    height: usize,
) -> String {
    let mut visible_map = String::new();

    for y in 0..height {
        for x in 0..width {
            let map_x = x + camera_x;
            let map_y = y + camera_y;

            if map_y < MAP_HEIGHT && map_x < MAP_WIDTH {
                let value = map[map_y][map_x];
                let ch = get_char_for_value(value);
                visible_map.push(ch);
            } else {
                visible_map.push(' ');
            }
        }
        if y < height - 1 {
            visible_map.push('\n');
        }
    }

    visible_map
}
