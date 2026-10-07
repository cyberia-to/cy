//! soma-kernel — the first running piece of soma: one cognition loop.
//!
//! A question comes in as text; soma turns it over to glia (the model layer),
//! gets an answer back, and remembers the exchange as content-addressed
//! particles ready to be linked into a cybergraph. That is the whole of phase
//! 1: think locally, leave a trace in the graph. The four loops, the market,
//! the Body budget — all of that arrives later and *around* this, because
//! whatever else soma becomes, this is the part it does all day.
//!
//! Boundaries, deliberately kept:
//! - soma decides *what* to think about; glia runs the thinking. No tensor,
//!   dtype or backend name crosses into this crate's API.
//! - soma produces particles and the *text* behind them; committing links is
//!   the host's job through its shared graph and explicitly selected neuron.
//!   This computation kernel has no keys or independent persistence writer.
//!
//! The mind runs on its own thread. Model loading takes a second and
//! generation takes a few more, and the caller is typically a UI frame loop
//! that cannot wait for either. [`Soma::ask`] returns immediately;
//! [`Soma::poll`] hands back [`SomaEvent`]s as they happen.

pub mod agent;

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use run::backend::Backend;
use run::generate::{ModelRunner, SampleConfig, SampleKind};
use run::tokenizer::ChatMessage;

/// A particle is the project's canonical hash of the content. Text in,
/// 32 bytes out; the same text is the same particle on every machine, which is
/// what lets independently-running minds converge on a shared graph.
pub fn particle_of(text: &str) -> [u8; 32] {
    let h = hemera::hash(text.as_bytes());
    let b = h.as_bytes();
    let mut out = [0u8; 32];
    let n = b.len().min(32);
    out[..n].copy_from_slice(&b[..n]);
    out
}

/// The anchor every exchange hangs off: `particle_of("soma")`. One well-known
/// particle, so all of a mind's Q→A pairs are reachable from a single root
/// instead of floating as disconnected islands.
pub fn soma_anchor() -> [u8; 32] {
    particle_of("soma")
}

/// What soma has to say back, in the order it says it.
#[derive(Debug, Clone)]
pub enum SomaEvent {
    /// The model is loading. Happens once, on the first question.
    Waking,
    /// The mind is running the question. Sent as generation starts.
    Thinking,
    /// Prefill is stepping the prompt: `done` of `total` tokens already
    /// consumed. The first event is `{0, n}`; the last is `{n, n}` when
    /// decode is about to start. Without this the UI is silent for the
    /// whole prompt pass, which on a phone is long enough to look dead.
    Prefill { done: usize, total: usize },
    /// A piece of the answer, as it is being written. Concatenating every
    /// delta gives the raw generation; [`SomaEvent::Answer`] carries the
    /// cleaned-up whole.
    Delta(String),
    /// The answer, with the question it answers and the concepts it touches.
    Answer {
        question: String,
        answer: String,
        /// Recurring content words of the exchange, most frequent first.
        /// These are how one exchange weaves into the rest of the graph: the
        /// same concept in two conversations is the same particle.
        concepts: Vec<String>,
        tokens: usize,
        tok_per_s: f32,
        /// Prompt tokens / prefill seconds.
        tok_in_s: f32,
        /// Generated tokens / decode seconds.
        tok_out_s: f32,
    },
    /// The mind failed. The text says how; the mind stays up for the next ask.
    Error(String),
    /// The mind will run this model from the next question on.
    ModelChanged(PathBuf),
}

/// Where the mind's weights and habits come from.
#[derive(Debug, Clone)]
pub struct SomaConfig {
    /// Path to a glia `.model` file.
    pub model: PathBuf,
    /// Longest answer soma will produce, in tokens.
    pub max_tokens: usize,
    pub temperature: f32,
}

impl Default for SomaConfig {
    fn default() -> Self {
        Self {
            model: default_model_path(),
            max_tokens: 512,
            temperature: 0.7,
        }
    }
}

/// Which model the mind runs, resolved in order of intent: the `SOMA_MODEL`
/// env var (this run only), then `~/cyb/model` (the choice made in the UI,
/// device config like identity is), then the smallest model in `~/llm` that
/// answers well enough to be worth linking.
pub fn default_model_path() -> PathBuf {
    if let Ok(p) = std::env::var("SOMA_MODEL") {
        return PathBuf::from(p);
    }
    if let Ok(chosen) = std::fs::read_to_string(chosen_model_file()) {
        let chosen = chosen.trim();
        if !chosen.is_empty() {
            return PathBuf::from(chosen);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("llm/qwen3-0.6b-abl.model")
}

/// Where the UI's model choice persists — one path, one line.
pub fn chosen_model_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("cyb").join("model")
}

/// Who soma is, on every question. The commander is nushell; soma is talk.
const SOMA_SELF: &str = "You are soma, the local mind of this cyb. \
The commander on every world is nushell: a typed command runs there \
(`help`, `ls`, pipes). A question (`? …` or a word the shell does not \
know) comes to you. You do not execute commands. If a command would \
help, name it so they can type it.";

/// What the host can tell the mind.
enum Directive {
    Ask {
        question: String,
        /// What the graph already holds on the subject — recalled by the
        /// host, spoken into the prompt here. Empty means answer cold.
        context: Vec<String>,
        /// Extra standing instructions from the host (command catalog, …).
        system: String,
    },
    /// Put down the current weights and pick these up on the next question.
    UseModel(PathBuf),
}

/// A handle to the mind's thread. Cheap to hold, immediate to call.
pub struct Soma {
    ask_tx: Sender<Directive>,
    event_rx: Receiver<SomaEvent>,
}

impl Soma {
    /// Start the mind. The thread is spawned now; the model is loaded lazily
    /// on the first question, so a cyb that is never asked anything never
    /// pays for the weights.
    pub fn spawn(cfg: SomaConfig) -> Self {
        let (ask_tx, ask_rx) = channel::<Directive>();
        let (event_tx, event_rx) = channel::<SomaEvent>();
        std::thread::Builder::new()
            .name("soma".into())
            .spawn(move || mind_loop(cfg, ask_rx, event_tx))
            .expect("spawn soma thread");
        Self { ask_tx, event_rx }
    }

    /// Hand the mind a question. Returns immediately; the answer arrives
    /// through [`Soma::poll`]. Questions queue — a second ask while the first
    /// is generating runs after it.
    pub fn ask(&self, question: impl Into<String>) {
        self.ask_grounded(question, Vec::new());
    }

    /// Ask with recall: `context` is what the graph already holds on the
    /// subject, and the answer is asked to build on it. This is the
    /// difference between chatting *near* a cybergraph and reasoning *over*
    /// one — the mind is told what it already knows.
    pub fn ask_grounded(&self, question: impl Into<String>, context: Vec<String>) {
        self.ask_on(question, context, String::new());
    }

    /// Ask with recall and a host-supplied standing note (the shell's
    /// command names, a body-specific fact). Folded into the system turn.
    pub fn ask_on(
        &self,
        question: impl Into<String>,
        context: Vec<String>,
        system: impl Into<String>,
    ) {
        let _ = self.ask_tx.send(Directive::Ask {
            question: question.into(),
            context,
            system: system.into(),
        });
    }

    /// Switch minds. The current weights are dropped, the new ones load
    /// lazily on the next question — switching costs nothing until it is
    /// used. Queued like an ask, so a switch after a pending question takes
    /// effect after that question is answered.
    pub fn use_model(&self, path: impl Into<PathBuf>) {
        let _ = self.ask_tx.send(Directive::UseModel(path.into()));
    }

    /// The next event, if one has happened. Non-blocking: call it from a
    /// frame loop.
    pub fn poll(&self) -> Option<SomaEvent> {
        match self.event_rx.try_recv() {
            Ok(ev) => Some(ev),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(SomaEvent::Error("soma thread is gone".into())),
        }
    }
}

// ── the mind's thread ────────────────────────────────────────────────────────

/// The loaded model with everything needed to run it. Lives only on the
/// mind's thread; nothing here is shared.
struct LoadedMind {
    model: run::arch::decoder::LlamaModel,
    tokenizer: run::Tokenizer,
    backend: Box<dyn Backend>,
}

fn interactive_thread() {
    #[cfg(target_os = "macos")]
    unsafe {
        extern "C" {
            fn pthread_set_qos_class_self_np(qos: u32, rel: i32) -> i32;
        }
        const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
        pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
    }
}

fn mind_loop(mut cfg: SomaConfig, ask_rx: Receiver<Directive>, event_tx: Sender<SomaEvent>) {
    // A plain spawned thread defaults to a QoS the scheduler is free to park
    // on efficiency cores, and everything downstream — kernel launches,
    // waits, the sampler between them — inherits that. Measured on an
    // M4 Max: the same model, same backend, same weights ran at 12 tok/s
    // from a default thread and 60+ from the main one. Thinking is what the
    // user is waiting for; say so.
    interactive_thread();

    let mut mind: Option<LoadedMind> = None;

    while let Ok(directive) = ask_rx.recv() {
        let (question, context, system) = match directive {
            Directive::Ask {
                question,
                context,
                system,
            } => (question, context, system),
            Directive::UseModel(path) => {
                mind = None;
                cfg.model = path.clone();
                let _ = event_tx.send(SomaEvent::ModelChanged(path));
                continue;
            }
        };
        // Wake on first use.
        if mind.is_none() {
            let _ = event_tx.send(SomaEvent::Waking);
            match wake(&cfg) {
                Ok(m) => mind = Some(m),
                Err(e) => {
                    let _ = event_tx.send(SomaEvent::Error(e));
                    continue;
                }
            }
        }
        let m = mind.as_mut().unwrap();

        let _ = event_tx.send(SomaEvent::Thinking);
        let delta_tx = event_tx.clone();
        let prefill_tx = event_tx.clone();
        match think(
            m,
            &question,
            &context,
            &system,
            &cfg,
            move |d| {
                let _ = delta_tx.send(SomaEvent::Delta(d));
            },
            move |done, total| {
                let _ = prefill_tx.send(SomaEvent::Prefill { done, total });
            },
        ) {
            Ok(stats) => {
                let concepts = concepts_of(&question, &stats.answer);
                let _ = event_tx.send(SomaEvent::Answer {
                    question,
                    answer: stats.answer,
                    concepts,
                    tokens: stats.gen_tokens,
                    tok_per_s: stats.tok_out_s,
                    tok_in_s: stats.tok_in_s,
                    tok_out_s: stats.tok_out_s,
                });
            }
            Err(e) => {
                let _ = event_tx.send(SomaEvent::Error(e));
            }
        }
    }
}

/// Openable and large enough to be a real .model — `exists()` is not
/// enough: an adb-pushed file on Android FUSE can stat and still refuse
/// mmap, which is how soma loads weights.
fn usable_model(path: &std::path::Path) -> bool {
    std::fs::File::open(path)
        .and_then(|f| f.metadata())
        .map(|m| m.len() > 1024 && path.extension().is_some_and(|x| x == "model"))
        .unwrap_or(false)
}

fn resolve_model(preferred: &std::path::Path) -> Result<std::path::PathBuf, String> {
    if usable_model(preferred) {
        return Ok(preferred.to_path_buf());
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let dir = std::path::PathBuf::from(home).join("llm");
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut models: Vec<std::path::PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| usable_model(p))
            .collect();
        models.sort_by_key(|p| p.metadata().map(|m| m.len()).unwrap_or(u64::MAX));
        if let Some(p) = models.into_iter().next() {
            return Ok(p);
        }
    }
    Err("no model on this body — open models and fetch qwen3-0.6b, or put a .model in ~/llm".into())
}

fn wake(cfg: &SomaConfig) -> Result<LoadedMind, String> {
    let model_path = resolve_model(&cfg.model)?;
    let lm = run::LoadedModel::load(&model_path).map_err(|e| format!("model load: {e}"))?;
    let tokenizer = run::tokenizer::build_tokenizer(&lm).map_err(|e| format!("tokenizer: {e}"))?;
    let mut model = run::arch::decoder::LlamaModel::from_loaded(&lm)
        .map_err(|e| format!("model build: {e}"))?;

    // The fastest backend the body has, exactly as the CLI picks it. What
    // looked like an off-main-thread honeycrisp deadlock — and cost soma
    // weeks on the CPU backend — was two stacked bugs of ours: the weights
    // were never uploaded to the backend (the CLI calls `to_backend`, soma
    // did not), and the resulting per-op fallback spun in an infinite
    // self-recursion in glia. Both fixed; the thread was never the problem.
    // `SOMA_BACKEND=cpu|wgpu|honeycrisp` still overrides for experiments.
    let choice = std::env::var("SOMA_BACKEND").unwrap_or_else(|_| "auto".into());
    let backend: Box<dyn Backend> = match choice.as_str() {
        "cpu" => Box::new(run::backend::cpu::CpuBackend::new()),
        "wgpu" => match run::backend::wgpu::WgpuRsBackend::new() {
            Ok(b) => Box::new(b),
            Err(_) => Box::new(run::backend::cpu::CpuBackend::new()),
        },
        #[cfg(target_os = "macos")]
        "honeycrisp" => match run::backend::honeycrisp::HoneycrispBackend::new() {
            Ok(b) => Box::new(b),
            Err(_) => Box::new(run::backend::cpu::CpuBackend::new()),
        },
        _ => {
            #[cfg(target_os = "macos")]
            {
                match run::backend::honeycrisp::HoneycrispBackend::new() {
                    Ok(b) => Box::new(b) as Box<dyn Backend>,
                    Err(_) => Box::new(run::backend::cpu::CpuBackend::new()),
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                Box::new(run::backend::cpu::CpuBackend::new())
            }
        }
    };
    // The upload is what makes the fast paths eligible: without it every
    // fused op sees host tensors and takes the slow road.
    if let Err(e) = model.to_backend(backend.as_ref()) {
        return Err(format!("weight upload: {e}"));
    }
    log::info!("soma: backend {:?}", backend.kind());

    log::info!("soma: awake — {}", cfg.model.display());
    Ok(LoadedMind {
        model,
        tokenizer,
        backend,
    })
}

struct ThinkStats {
    answer: String,
    gen_tokens: usize,
    tok_in_s: f32,
    tok_out_s: f32,
}

fn think(
    m: &mut LoadedMind,
    question: &str,
    context: &[String],
    system: &str,
    cfg: &SomaConfig,
    on_delta: impl FnMut(String),
    on_prefill: impl FnMut(usize, usize),
) -> Result<ThinkStats, String> {
    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut self_msg = SOMA_SELF.to_string();
    if !system.is_empty() {
        self_msg.push('\n');
        self_msg.push_str(system);
    }
    if !context.is_empty() {
        self_msg.push_str(
            "\nYour cybergraph already holds the following, from earlier \
             exchanges. Build on it; do not contradict it without saying why.",
        );
        for c in context {
            self_msg.push_str("\n- ");
            self_msg.push_str(c);
        }
    }
    messages.push(ChatMessage {
        role: "system".into(),
        content: self_msg,
    });
    messages.push(ChatMessage {
        role: "user".into(),
        content: question.into(),
    });

    generate_messages(
        m,
        &messages,
        cfg,
        on_delta,
        on_prefill,
        std::time::Instant::now() + std::time::Duration::from_secs(180),
        8192,
    )
}

fn generate_messages(
    m: &mut LoadedMind,
    messages: &[ChatMessage],
    cfg: &SomaConfig,
    mut on_delta: impl FnMut(String),
    mut on_prefill: impl FnMut(usize, usize),
    deadline: std::time::Instant,
    max_prompt_tokens: usize,
) -> Result<ThinkStats, String> {
    // The model's own chat template, then an empty think block appended. The
    // qwen3 family spends its whole token budget deliberating inside
    // <think> unless the block is already there and closed — this is the
    // documented way to ask it to just answer.
    let mut prompt = run::generate::build_chat_prompt(&m.tokenizer, messages);
    prompt.push_str("<think>\n\n</think>\n\n");

    let sample = SampleConfig {
        method: if cfg.temperature > 0.0 {
            SampleKind::TopP
        } else {
            SampleKind::Greedy
        },
        temperature: cfg.temperature,
        top_p: 0.95,
        top_k: 40,
    };

    // The decode loop lives here rather than in glia's `generate` because the
    // point is to see the answer *as it is written*: every new token, the
    // full sequence is re-decoded and whatever text grew is sent on. Decoding
    // from the start each step is what keeps multi-byte characters honest —
    // a token boundary is not a character boundary, and decoding tokens one
    // at a time tears UTF-8 apart exactly where it matters (any text that is
    // not English).
    m.model.reset();
    let t_prefill = std::time::Instant::now();
    let mut ids = m.tokenizer.encode(&prompt);
    if let Some(bos) = m.tokenizer.bos_token_id {
        if ids.first() != Some(&bos) {
            ids.insert(0, bos);
        }
    }
    if ids.len() > max_prompt_tokens {
        return Err("model prompt token limit".into());
    }
    let prompt_n = ids.len();
    on_prefill(0, prompt_n);
    let mut logits: Vec<f32> = Vec::new();
    let mut last_report = t_prefill;
    for (i, &t) in ids.iter().enumerate() {
        if std::time::Instant::now() >= deadline {
            return Err("model prefill deadline".into());
        }
        logits = m
            .model
            .step(t, m.backend.as_ref())
            .map_err(|e| format!("prefill: {e}"))?;
        let stepped = i + 1;
        let now = std::time::Instant::now();
        if stepped == prompt_n
            || stepped % 4 == 0
            || now.duration_since(last_report).as_millis() >= 200
        {
            on_prefill(stepped, prompt_n);
            last_report = now;
        }
    }

    let prefill_s = t_prefill.elapsed().as_secs_f32().max(1e-3);
    let prompt_tokens = prompt_n;
    let t_decode = std::time::Instant::now();
    let mut generated: Vec<u32> = Vec::new();
    let mut sent = String::new();
    for _ in 0..cfg.max_tokens {
        if std::time::Instant::now() >= deadline {
            return Err("model decode deadline".into());
        }
        let next = run::generate::sample(&logits, sample);
        if m.tokenizer.is_eos(next) {
            break;
        }
        generated.push(next);
        let full = m.tokenizer.decode(&generated, false);
        if full.len() > soma_agent::MAX_TEXT {
            return Err("model text result limit".into());
        }
        if full.len() > sent.len() && full.is_char_boundary(sent.len()) {
            let delta = full[sent.len()..].to_string();
            let _ = on_delta(delta);
            sent = full;
        }
        logits = m
            .model
            .step(next, m.backend.as_ref())
            .map_err(|e| format!("decode: {e}"))?;
    }

    let raw = m.tokenizer.decode(&generated, false);
    let decode_s = t_decode.elapsed().as_secs_f32().max(1e-3);
    let gen = generated.len();
    Ok(ThinkStats {
        answer: tidy(&raw),
        gen_tokens: gen,
        tok_in_s: prompt_tokens as f32 / prefill_s,
        tok_out_s: gen as f32 / decode_s,
    })
}

/// The recurring content words of an exchange, most frequent first — the
/// hooks by which it hangs onto the rest of the graph.
///
/// Deliberately dumb: lowercase words of four letters or more, minus a small
/// list of function words, ranked by how often the exchange uses them. No
/// embedding, no model — because the value is not in the extraction being
/// clever, it is in the extraction being *deterministic*: the same concept in
/// two exchanges, or two minds, hashes to the same particle, and that
/// identity is what makes separate conversations grow into one graph.
pub fn concepts_of(question: &str, answer: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        // en
        "this",
        "that",
        "with",
        "from",
        "have",
        "what",
        "which",
        "your",
        "will",
        "would",
        "could",
        "should",
        "about",
        "there",
        "their",
        "them",
        "then",
        "than",
        "these",
        "those",
        "some",
        "such",
        "also",
        "into",
        "over",
        "more",
        "most",
        "other",
        "when",
        "where",
        "here",
        "does",
        "very",
        "just",
        "like",
        "used",
        "using",
        "each",
        "between",
        "because",
        "while",
        "been",
        "being",
        "only",
        "must",
        "many",
        "much",
        "they",
        "were",
        "your",
        "yours",
        "ours",
        "it's",
        "don't",
        "can't",
        // ru
        "этот",
        "это",
        "эта",
        "как",
        "что",
        "или",
        "для",
        "если",
        "чтобы",
        "который",
        "которая",
        "может",
        "быть",
        "есть",
        "она",
        "оно",
        "они",
        "его",
        "её",
        "их",
        "нас",
        "вас",
        "при",
        "под",
        "над",
        "все",
        "всё",
        "так",
        "тоже",
        "также",
        "когда",
        "где",
        "почему",
        "потому",
    ];

    let mut counts: Vec<(String, usize)> = Vec::new();
    for word in format!("{question} {answer}")
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
    {
        if word.chars().count() < 4 || STOP.contains(&word) {
            continue;
        }
        match counts.iter_mut().find(|(w, _)| w == word) {
            Some((_, n)) => *n += 1,
            None => counts.push((word.to_string(), 1)),
        }
    }
    // Stable: frequency first, then order of first appearance (already the
    // vec order), so equal counts do not reshuffle between runs.
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.into_iter().take(5).map(|(w, _)| w).collect()
}

/// Strip whatever deliberation leaked into the answer anyway, and trim.
fn tidy(raw: &str) -> String {
    let mut s = raw;
    // A closed think block: drop it and keep what follows.
    if let Some(open) = s.find("<think>") {
        if let Some(close) = s.find("</think>") {
            if close > open {
                return format!(
                    "{}{}",
                    &s[..open],
                    s[close + "</think>".len()..].trim_start()
                )
                .trim()
                .to_string();
            }
        }
        // An unclosed think block means the budget ran out mid-thought;
        // everything after the tag is deliberation, not an answer.
        s = &s[..open];
    }
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particles_are_stable_and_distinct() {
        assert_eq!(particle_of("a question"), particle_of("a question"));
        assert_ne!(particle_of("a question"), particle_of("an answer"));
        assert_ne!(soma_anchor(), particle_of(""));
    }

    #[test]
    fn tidy_strips_closed_think_blocks() {
        assert_eq!(tidy("<think>\nhmm\n</think>\n\nParis."), "Paris.");
        assert_eq!(tidy("Paris."), "Paris.");
        // Unclosed: the budget died mid-thought; there is no answer to keep.
        assert_eq!(tidy("prefix <think>endless deliberation"), "prefix");
    }

    #[test]
    fn concepts_are_deterministic_and_ranked() {
        let a = concepts_of(
            "what is a cybergraph?",
            "a cybergraph links particles; particles form the cybergraph",
        );
        let b = concepts_of(
            "what is a cybergraph?",
            "a cybergraph links particles; particles form the cybergraph",
        );
        assert_eq!(a, b);
        assert_eq!(a[0], "cybergraph", "most frequent word ranks first: {a:?}");
        assert!(a.contains(&"particles".to_string()));
        assert!(!a.iter().any(|w| w == "what"), "stopwords excluded: {a:?}");
    }

    /// Switching models mid-flight: the swap is acknowledged, and the next
    /// ask wakes the new weights. Skipped without two models on disk.
    #[test]
    fn switches_models_between_asks() {
        let first = SomaConfig::default().model;
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let second = PathBuf::from(&home).join("llm/qwen2.5-coder-1.5b-q4k.canonical.model");
        if !first.exists() || !second.exists() {
            eprintln!("skipping: needs both {first:?} and {second:?}");
            return;
        }
        let soma = Soma::spawn(SomaConfig {
            max_tokens: 16,
            temperature: 0.0,
            ..Default::default()
        });
        soma.use_model(&second);
        soma.ask("Reply with exactly one word: ready");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        let mut changed_to: Option<PathBuf> = None;
        loop {
            assert!(
                std::time::Instant::now() < deadline,
                "no answer in three minutes"
            );
            match soma.poll() {
                Some(SomaEvent::ModelChanged(p)) => changed_to = Some(p),
                Some(SomaEvent::Answer { answer, .. }) => {
                    assert_eq!(
                        changed_to.as_ref(),
                        Some(&second),
                        "answered before acknowledging the switch"
                    );
                    assert!(!answer.trim().is_empty());
                    break;
                }
                Some(SomaEvent::Error(e)) => panic!("soma error: {e}"),
                Some(_) => {}
                None => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
    }

    /// Grounding is real, not decorative: a fact that exists only in the
    /// context must surface in the answer. Skipped without weights on disk.
    #[test]
    fn grounded_answers_use_the_graph() {
        let cfg = SomaConfig::default();
        if !cfg.model.exists() {
            eprintln!("skipping: no model at {}", cfg.model.display());
            return;
        }
        let soma = Soma::spawn(SomaConfig {
            max_tokens: 48,
            temperature: 0.0,
            ..cfg
        });
        soma.ask_grounded(
            "What color is the sky over cyberia? One word.",
            vec![
                "Q: describe cyberia\nA: In cyberia the sky is bright green at all hours."
                    .to_string(),
            ],
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        loop {
            assert!(
                std::time::Instant::now() < deadline,
                "no answer in two minutes"
            );
            match soma.poll() {
                Some(SomaEvent::Answer { answer, .. }) => {
                    assert!(
                        answer.to_lowercase().contains("green"),
                        "the graph said green; the answer ignored it: {answer:?}"
                    );
                    break;
                }
                Some(SomaEvent::Error(e)) => panic!("soma error: {e}"),
                Some(_) => {}
                None => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
    }

    /// The full loop against the real model: ask, get an answer, see the
    /// events in order. Skipped quietly when no model is installed, because
    /// CI machines do not carry weights.
    #[test]
    fn asks_the_real_model() {
        let cfg = SomaConfig::default();
        if !cfg.model.exists() {
            eprintln!("skipping: no model at {}", cfg.model.display());
            return;
        }
        let soma = Soma::spawn(SomaConfig {
            max_tokens: 64,
            temperature: 0.0,
            ..cfg
        });
        soma.ask("Reply with exactly one word: ready");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let mut saw_waking = false;
        let mut deltas = String::new();
        loop {
            assert!(
                std::time::Instant::now() < deadline,
                "soma did not answer within two minutes"
            );
            match soma.poll() {
                Some(SomaEvent::Waking) => saw_waking = true,
                Some(SomaEvent::Thinking) => {}
                Some(SomaEvent::Prefill { .. }) => {}
                Some(SomaEvent::Delta(d)) => deltas.push_str(&d),
                Some(SomaEvent::ModelChanged(_)) => {}
                Some(SomaEvent::Answer { answer, tokens, .. }) => {
                    assert!(saw_waking, "answered without waking first");
                    assert!(tokens > 0);
                    assert!(!answer.trim().is_empty(), "empty answer");
                    assert!(
                        deltas.contains(answer.trim()),
                        "the streamed text must contain the final answer;\n deltas: {deltas:?}\n answer: {answer:?}"
                    );
                    break;
                }
                Some(SomaEvent::Error(e)) => panic!("soma error: {e}"),
                None => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
    }
}
