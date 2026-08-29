// Measure load + generation speed for a model given on the command line.
fn main() {
    let path = std::env::args().nth(1).expect("model path");
    let soma = soma_kernel::Soma::spawn(soma_kernel::SomaConfig {
        model: path.into(),
        max_tokens: 12,
        temperature: 0.0,
        particles: None,
    });
    soma.ask("Say ready.");
    let t0 = std::time::Instant::now();
    loop {
        if t0.elapsed().as_secs() > 600 { eprintln!("TIMEOUT"); std::process::exit(1); }
        match soma.poll() {
            Some(soma_kernel::SomaEvent::Waking) => eprintln!("[{:5.1}s] waking", t0.elapsed().as_secs_f32()),
            Some(soma_kernel::SomaEvent::Thinking) => eprintln!("[{:5.1}s] thinking", t0.elapsed().as_secs_f32()),
            Some(soma_kernel::SomaEvent::Answer{tokens, tok_per_s, ..}) => {
                eprintln!("[{:5.1}s] {} tok @ {:.1} tok/s", t0.elapsed().as_secs_f32(), tokens, tok_per_s);
                break;
            }
            Some(soma_kernel::SomaEvent::Error(e)) => { eprintln!("ERR {e}"); std::process::exit(1); }
            Some(_) => {}
            None => std::thread::sleep(std::time::Duration::from_millis(200)),
        }
    }
}
