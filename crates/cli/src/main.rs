use deterministic_router::{DeterministicRouter, AgentDecision, AgentDomain, TensorSignal};
use sigma_kernel_lib::{
    MultiplicityArena, SigmaVerdict, CANDIDATE_PRIMES, SIGMA_GAP_MAX, SIGMA_RESONANCE_MIN,
};
use serde_json::json;
use std::io::{self, Write, BufRead};

// ---------------------------------------------------------------------------
// ANSI
// ---------------------------------------------------------------------------

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[97m";

// ---------------------------------------------------------------------------
// Banner
// ---------------------------------------------------------------------------

fn banner() {
    println!("{CYAN}{BOLD}╔══════════════════════════════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║  Σ-ROUTER — Sigma Kernel × Deterministic Router Terminal   ║{RESET}");
    println!("{CYAN}{BOLD}║  Admissibility Gate · Tensor Scoring · WORM Decisions       ║{RESET}");
    println!("{CYAN}{BOLD}╚══════════════════════════════════════════════════════════════╝{RESET}");
    println!();
    println!("{BOLD}Sigma Kernel commands:{RESET}");
    println!("  {GREEN}spine{RESET}   <p1> <p2> ...                Build spine & run gates G1-G4");
    println!("  {GREEN}solve{RESET}  <depth>                     Recursive stabilization solver");
    println!();
    println!("{BOLD}Router commands:{RESET}");
    println!("  {GREEN}score{RESET}  <domain> <amount> <payload> Tensor-score a transaction");
    println!("  {GREEN}route{RESET}  <domain> <message>          Classify intent & respond");
    println!("  {GREEN}agent{RESET}  <domain> <event> <amount> <payload>  Full agent decision");
    println!();
    println!("{BOLD}Meta:{RESET}");
    println!("  {GREEN}config{RESET}                             Show kernel config");
    println!("  {GREEN}help{RESET}                               This help");
    println!("  {GREEN}quit{RESET}                               Exit");
    println!();
}

// ---------------------------------------------------------------------------
// Print helpers
// ---------------------------------------------------------------------------

fn print_verdict(label: &str, v: &SigmaVerdict) {
    match v {
        SigmaVerdict::Admit => println!("│ {BOLD}{label}:{RESET}  {GREEN}ADMIT{RESET}"),
        SigmaVerdict::DissonanceGap { index, gap } => {
            println!("│ {BOLD}{label}:{RESET}  {RED}DISSONANCE_GAP @ index {index} gap={gap}{RESET}");
        }
        SigmaVerdict::DissonanceResonance { index, .. } => {
            println!("│ {BOLD}{label}:{RESET}  {RED}DISSONANCE_RESONANCE @ index {index}{RESET}");
        }
        SigmaVerdict::Malformed => println!("│ {BOLD}{label}:{RESET}  {RED}MALFORMED{RESET}"),
        SigmaVerdict::Unsealed => println!("│ {BOLD}{label}:{RESET}  {YELLOW}UNSEALED{RESET}"),
    };
}

fn print_tensor_signal(sig: &TensorSignal) {
    let bc = match sig.budget_head.label.as_str() {
        "GREEN" => GREEN,
        "AMBER" => YELLOW,
        _ => RED,
    };
    let vc = match sig.vendor_trust_head.label.as_str() {
        "TRUSTED" => GREEN,
        "UNVERIFIED" => YELLOW,
        _ => RED,
    };
    let rc = match sig.risk_head.label.as_str() {
        "LOW_RISK" => GREEN,
        "MED_RISK" => YELLOW,
        _ => RED,
    };
    let rec = match sig.recommendation.as_str() {
        "PROCEED" => GREEN,
        "REVIEW" => YELLOW,
        _ => RED,
    };

    println!();
    println!("{BOLD}┌─── TENSOR SIGNAL ──────────────────────────────────┐{RESET}");
    println!("│ {BOLD}budget:{RESET}       {bc}{}{RESET}  ({:.3})", sig.budget_head.label, sig.budget_head.score);
    println!("│ {BOLD}vendor trust:{RESET} {vc}{}{RESET}  ({:.3})", sig.vendor_trust_head.label, sig.vendor_trust_head.score);
    println!("│ {BOLD}risk:{RESET}         {rc}{}{RESET}    ({:.3})", sig.risk_head.label, sig.risk_head.score);
    println!("│");
    println!("│ {BOLD}composite:{RESET}    {:.3}", sig.composite_signal);
    println!("│ {BOLD}recommend:{RESET}    {rec}{}{RESET}", sig.recommendation);
    println!("└──────────────────────────────────────────────────────┘{RESET}");
}

fn print_decision(d: &AgentDecision) {
    let ac = match d.action.as_str() {
        "PROCEED" => GREEN,
        "REVIEW" => YELLOW,
        _ => RED,
    };

    println!();
    println!("{BOLD}┌─── AGENT DECISION ────────────────────────────────┐{RESET}");
    println!("│ {BOLD}action:{RESET}     {ac}{}{RESET}", d.action);
    if d.approved {
        println!("│ {BOLD}approved:{RESET}   {GREEN}✓{RESET}");
    } else {
        println!("│ {BOLD}approved:{RESET}   {RED}✗{RESET}");
    }
    println!("│ {BOLD}confidence:{RESET} {:.3}", d.confidence);
    if let Some(ref seal) = d.seal {
        println!("│ {BOLD}seal:{RESET}       {DIM}{seal}{RESET}");
    }
    println!("│ {BOLD}reasoning:{RESET}  {}", d.reasoning);
    if !d.chain_of_thought.is_empty() {
        println!("│");
        println!("│ {BOLD}chain of thought:{RESET}");
        for step in &d.chain_of_thought {
            println!("│   {CYAN}→{RESET} {step}");
        }
    }
    println!("└──────────────────────────────────────────────────────┘{RESET}");
}

fn print_intent(intent: &deterministic_router::Intent) {
    let ds = match intent.domain {
        AgentDomain::Finance => "FINANCE",
        AgentDomain::Crm => "CRM",
        AgentDomain::Procurement => "PROCUREMENT",
        AgentDomain::Bifrost => "BIFROST",
        AgentDomain::Treasury => "TREASURY",
        AgentDomain::Risk => "RISK",
    };
    println!();
    println!("{BOLD}┌─── INTENT ─────────────────────────────────────────┐{RESET}");
    println!("│ {BOLD}domain:{RESET}      {MAGENTA}{ds}{RESET}");
    println!("│ {BOLD}sub_intent:{RESET}  {CYAN}{}{RESET}", intent.sub_intent);
    println!("│ {BOLD}confidence:{RESET}  {:.2}", intent.confidence);
    println!("└──────────────────────────────────────────────────────┘{RESET}");
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

fn parse_payload(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_else(|_| {
        let mut map = serde_json::Map::new();
        for pair in s.split_whitespace() {
            if let Some((k, v)) = pair.split_once('=') {
                if let Ok(num) = v.parse::<f64>() {
                    map.insert(k.to_string(), json!(num));
                } else {
                    map.insert(k.to_string(), json!(v));
                }
            }
        }
        json!(map)
    })
}

fn parse_args(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';

    for c in line.chars() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
        } else if c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_primes(args: &[String]) -> Vec<u32> {
    args.iter()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

// ---------------------------------------------------------------------------
// Sigma Kernel: spine command
// ---------------------------------------------------------------------------

fn cmd_spine(primes: &[u32]) {
    if primes.is_empty() {
        println!("{RED}usage: spine <p1> <p2> ... (e.g. spine 2 3 5 7){RESET}");
        return;
    }

    unsafe {
        let mut arena = MultiplicityArena::build_spine(primes.len());
        for (i, &p) in primes.iter().enumerate() {
            (*arena.nodes.add(i)).prime_val = p;
            (*arena.nodes.add(i)).multiplicity = 1;
            (*arena.nodes.add(i)).spectral_weight = 1.5;
        }

        println!();
        println!("{BOLD}┌─── SPINE ADJUDICATION ────────────────────────────┐{RESET}");
        println!("│ primes: {primes:?}");
        println!("│");
        print_verdict("G1 Phase Mirror", &arena.gate_phase_mirror());
        print_verdict("G2 Resonance   ", &arena.gate_resonance());
        print_verdict("G3 Sigma Gate  ", &arena.sigma_gate());

        if let SigmaVerdict::Admit = arena.sigma_gate() {
            let gov_hash = 0xdeadbeef_cafebabe;
            let seal_v = arena.seal(gov_hash);
            print_verdict("G4 GOV_HASH    ", &seal_v);
            println!("│ {BOLD}sealed:{RESET}     {GREEN}true{RESET}");
        } else {
            println!("│ {BOLD}sealed:{RESET}     {RED}false{RESET} (gates rejected)");
        }

        println!("└──────────────────────────────────────────────────────┘{RESET}");

        // Print with actual values
        println!();
        println!("{DIM}Spine detail:{RESET}");
        for (i, _p) in primes.iter().enumerate() {
            let node = *arena.nodes.add(i);
            println!("  [{i}] prime={:<2} mult={} weight={:.1} resonance={:.1}",
                     node.prime_val, node.multiplicity, node.spectral_weight, node.resonance());
        }

        arena.destroy();
    }
}

// ---------------------------------------------------------------------------
// Sigma Kernel: solve command
// ---------------------------------------------------------------------------

fn cmd_solve(depth: usize) {
    if depth == 0 || depth > 8 {
        println!("{RED}usage: solve <depth> (1-8){RESET}");
        return;
    }

    unsafe {
        let mut arena = MultiplicityArena::build_spine(depth);
        for i in 0..depth {
            (*arena.nodes.add(i)).spectral_weight = 1.5;
        }

        println!();
        println!("{BOLD}Recursive stabilization solver (depth={depth}){RESET}");
        let t0 = std::time::Instant::now();
        let found = arena.stabilize_tensor_recursive(depth, 0);
        let elapsed = t0.elapsed();

        println!("{BOLD}Result:{RESET}  {} in {:.2?}", if found { "ADMISSIBLE" } else { "NO SOLUTION" }, elapsed);

        if found {
            println!("{BOLD}Spine:{RESET}");
            for i in 0..depth {
                let node = *arena.nodes.add(i);
                println!("  [{i}] prime={:<2} mult={} weight={:.1} resonance={:.1}",
                         node.prime_val, node.multiplicity, node.spectral_weight, node.resonance());
            }

            print_verdict("G1", &arena.gate_phase_mirror());
            print_verdict("G2", &arena.gate_resonance());
            print_verdict("G3", &arena.sigma_gate());
        }

        arena.destroy();
    }
}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

async fn repl() {
    let router = DeterministicRouter::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    banner();

    loop {
        print!("{BOLD}{GREEN}Σ-router{RESET} {DIM}»{RESET} ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("{RED}error: {e}{RESET}");
                break;
            }
        }

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let args = parse_args(&line);
        let cmd = args[0].to_lowercase();

        match cmd.as_str() {
            "quit" | "exit" | "q" => {
                println!("{DIM}exiting{RESET}");
                break;
            }
            "help" | "h" | "?" => banner(),

            "config" => {
                println!();
                println!("{BOLD}Sigma Kernel:{RESET}");
                println!("  SIGMA_GAP_MAX:       {SIGMA_GAP_MAX}");
                println!("  SIGMA_RESONANCE_MIN: {SIGMA_RESONANCE_MIN}");
                println!("  CANDIDATE_PRIMES:    {CANDIDATE_PRIMES:?}");
                println!();
                println!("{BOLD}Router:{RESET}");
                println!("  domains: finance, crm, procurement, bifrost, treasury, risk");
                println!("  scoring: budget 40% + vendor 30% + risk 30%");
                println!("  thresholds: PROCEED >= 0.75, REVIEW >= 0.50, ESCALATE < 0.50");
            }

            "spine" => {
                let primes = parse_primes(&args[1..]);
                cmd_spine(&primes);
            }

            "solve" => {
                if args.len() < 2 {
                    println!("{RED}usage: solve <depth>{RESET}");
                    continue;
                }
                match args[1].parse::<usize>() {
                    Ok(d) => cmd_solve(d),
                    Err(_) => println!("{RED}invalid depth: {}{RESET}", args[1]),
                }
            }

            "score" => {
                if args.len() < 4 {
                    println!("{RED}usage: score <domain> <amount> <payload>{RESET}");
                    continue;
                }
                let amount: f64 = match args[2].parse() {
                    Ok(v) => v,
                    Err(_) => { println!("{RED}invalid amount: {}{RESET}", args[2]); continue; }
                };
                let payload = parse_payload(&args[3..].join(" "));
                let sig = router.tensor_score(amount, &payload);
                print_tensor_signal(&sig);
            }

            "route" => {
                if args.len() < 3 {
                    println!("{RED}usage: route <domain> <message>{RESET}");
                    continue;
                }
                let intent = DeterministicRouter::classify_intent(&args[1], &args[2..].join(" "));
                print_intent(&intent);
                let response = DeterministicRouter::route_message(&args[1], &args[2..].join(" "));
                println!();
                println!("{WHITE}{response}{RESET}");
            }

            "agent" => {
                if args.len() < 5 {
                    println!("{RED}usage: agent <domain> <event> <amount> <payload>{RESET}");
                    continue;
                }
                let amount: f64 = match args[3].parse() {
                    Ok(v) => v,
                    Err(_) => { println!("{RED}invalid amount: {}{RESET}", args[3]); continue; }
                };
                let payload = parse_payload(&args[4..].join(" "));
                let decision = router.process_event(&args[1], &args[2], amount, &payload).await;
                print_decision(&decision);
            }

            _ => {
                println!("{RED}unknown command: {cmd}{RESET}  (type {GREEN}help{RESET})");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CLI (non-interactive)
// ---------------------------------------------------------------------------

async fn cli_dispatch(args: &[String]) {
    match args[0].as_str() {
        "spine" => {
            let primes = parse_primes(&args[1..]);
            cmd_spine(&primes);
        }
        "solve" => {
            let depth = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
            cmd_solve(depth);
        }
        "agent" => {
            if args.len() < 5 {
                eprintln!("{RED}usage: sigma-router agent <domain> <event> <amount> <payload>{RESET}");
                std::process::exit(1);
            }
            let amount: f64 = args[3].parse().expect("invalid amount");
            let router = DeterministicRouter::new();
            let payload = parse_payload(&args[4..].join(" "));
            let decision = router.process_event(&args[1], &args[2], amount, &payload).await;
            print_decision(&decision);
        }
        "score" => {
            if args.len() < 4 {
                eprintln!("{RED}usage: sigma-router score <domain> <amount> <payload>{RESET}");
                std::process::exit(1);
            }
            let amount: f64 = args[2].parse().expect("invalid amount");
            let router = DeterministicRouter::new();
            let payload = parse_payload(&args[3..].join(" "));
            let sig = router.tensor_score(amount, &payload);
            print_tensor_signal(&sig);
        }
        "route" => {
            if args.len() < 3 {
                eprintln!("{RED}usage: sigma-router route <domain> <message>{RESET}");
                std::process::exit(1);
            }
            let intent = DeterministicRouter::classify_intent(&args[1], &args[2..].join(" "));
            print_intent(&intent);
            let resp = DeterministicRouter::route_message(&args[1], &args[2..].join(" "));
            println!("\n{resp}");
        }
        "help" | "-h" | "--help" => banner(),
        _ => {
            eprintln!("{RED}unknown: {}{RESET}", args[0]);
            banner();
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        repl().await;
    } else {
        cli_dispatch(&args).await;
    }
}
