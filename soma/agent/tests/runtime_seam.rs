use neuron_engine::{RuntimeInput, RuntimePort, RuntimeStep};
use neuron_node::Rune;
#[test]
fn actual_rune_task_loop_suspends_restores_and_terminates_without_a_new_vm() {
    let source = soma_agent::TASK_PROGRAM.as_bytes();
    let state = neuron_rune::value("0").unwrap();
    let event = neuron_rune::value("42").unwrap();
    let mut checkpoint = Rune
        .start(RuntimeInput {
            source,
            state: &state,
            event: &event,
            context: None,
            step_limit: 10000,
        })
        .unwrap();
    let mut total = 0;
    let mut reply = None;
    let mut seen = Vec::new();
    loop {
        match Rune
            .step(&checkpoint, reply.as_deref(), 1000, 10000)
            .unwrap()
        {
            RuntimeStep::Act {
                tag,
                arguments,
                checkpoint: next,
                used,
            } => {
                assert_eq!(tag, 0xAC75_0000_0000_0006);
                assert!(seen.len() < 2, "task loop failed to terminate");
                seen.push(neuron_rune::display(&arguments).unwrap());
                checkpoint = next;
                assert!(used >= total);
                total = used;
                Rune.validate_checkpoint(&checkpoint, 10000).unwrap();
                reply = Some(neuron_rune::value(if seen.len() == 1 { "43" } else { "0" }).unwrap());
            }
            RuntimeStep::Yield {
                checkpoint: next,
                used,
            } => {
                checkpoint = next;
                assert!(used >= total);
                total = used;
                reply = None;
            }
            RuntimeStep::Done { result, .. } => {
                assert_eq!(neuron_rune::display(&result).unwrap(), "0");
                break;
            }
            other => panic!("unexpected step {other:?}"),
        }
    }
    assert_eq!(seen, ["42", "43"]);
}
