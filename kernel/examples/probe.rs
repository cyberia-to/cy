fn main() {
    env_logger::init();
    eprintln!("spawn…");
    let soma = soma_kernel::Soma::spawn(soma_kernel::SomaConfig {
        max_tokens: 32,
        temperature: 0.0,
        particles: None,
        ..Default::default()
    });
    soma.ask("Reply with exactly one word: ready");
    let t0 = std::time::Instant::now();
    loop {
        if t0.elapsed().as_secs() > 180 { eprintln!("TIMEOUT"); break; }
        match soma.poll() {
            Some(ev) => {
                eprintln!("[{:5.1}s] {:?}", t0.elapsed().as_secs_f32(), ev);
                if matches!(ev, soma_kernel::SomaEvent::Answer{..} | soma_kernel::SomaEvent::Error(_)) { break; }
            }
            None => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }
}
