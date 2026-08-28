fn main() {
    let soma = soma_kernel::Soma::spawn(soma_kernel::SomaConfig {
        max_tokens: 200,
        temperature: 0.7,
        particles: None,
        ..Default::default()
    });
    soma.ask("What is a knowledge graph? Two sentences.");
    let t0 = std::time::Instant::now();
    loop {
        if t0.elapsed().as_secs() > 120 { eprintln!("TIMEOUT"); std::process::exit(1); }
        match soma.poll() {
            Some(soma_kernel::SomaEvent::Answer{answer, tokens, tok_per_s, ..}) => {
                eprintln!("[{:4.1}s] {} tok @ {:.1} tok/s\n{}", t0.elapsed().as_secs_f32(), tokens, tok_per_s, answer);
                break;
            }
            Some(soma_kernel::SomaEvent::Error(e)) => { eprintln!("ERR {e}"); std::process::exit(1); }
            Some(_) => {}
            None => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }
}
