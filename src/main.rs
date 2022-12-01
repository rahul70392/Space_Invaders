use std::error::Error;
use rusty_audio::Audio;
use std::{ io,thread, sync::mpsc::{self}, time::{Duration, Instant},};
use crossterm::{terminal,
terminal::EnterAlternateScreen,
terminal::LeaveAlternateScreen,
ExecutableCommand,
cursor::Hide,
cursor::Show,
event::{self, Event, KeyCode},
};
// use core::time::Duration;
use space_invaders::frame::new_frame;
use space_invaders::render;
use space_invaders::player::Player;
use space_invaders::frame::Drawable;
use space_invaders::invaders::Invaders; 

fn main() -> Result <(), Box<dyn Error>> {
    let mut audio = Audio::new();

    for item in &["explode", "lose", "move", "pew", "startup", "win"] {
        audio.add(item, &format!("sounds/{}.wav", item));
    }
    audio.play("startup");

    //Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //render loop in a seperate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true );
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    //Game Loop
    'gameloop : loop {
        let delta = instant.elapsed();
        instant = Instant::now();

        //per-frame initialization section
        let mut curr_frame = new_frame();

        //input events poll
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter =>{
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => { 
                        audio.play("lose");
                        break 'gameloop;
                    },
                    _ => {}
                }
            }
        }

        //updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        //Draw and render
        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        //win or lose
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    //cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
