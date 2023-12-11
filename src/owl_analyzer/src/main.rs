use analyzer::{get_trace, Analyzer, Report, Trace};
use clap::Parser;
use std::io::{self, Read, Write};

#[derive(Parser)]
struct Cli {
    /// Log level(0: Off, 1: Error, 2: Warn, 3: Info, 4: Debug, 5: Trace), default level 3
    #[clap(short, long, default_value = "3")]
    log: u8,
    /// Pipe path
    #[clap(short, long)]
    pipe_path: Option<String>,
    /// Commands file
    #[clap(long)]
    cmds_file: Option<String>,
    /// commands list, separate by `:`
    #[clap(short, long)]
    cmds: Option<String>,
    /// sign level (0, 1)
    #[clap(short, long)]
    sign: Option<f64>,
    /// leakage test times > 0
    #[clap(short, long)]
    test_times: Option<usize>,
    /// leakage test rand cmd
    #[clap(short, long)]
    rand_cmd: Option<String>,
    /// test command
    #[arg(last = true)]
    cmd: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if std::env::var("RUST_LOG").is_err() {
        env_logger::builder()
            .filter_level(match cli.log {
                0 => log::LevelFilter::Off,
                1 => log::LevelFilter::Error,
                2 => log::LevelFilter::Warn,
                3 => log::LevelFilter::Info,
                4 => log::LevelFilter::Debug,
                5 => log::LevelFilter::Trace,
                _ => log::LevelFilter::Info,
            })
            .init();
    } else {
        env_logger::init();
    }

    let mut cmds = Vec::default();

    let mut threshold = 2.0;

    // update threshold
    if let Some(sign) = cli.sign {
        if sign < 0.0 || sign > 1.0 {
            log::error!("sign must > 0 and < 1");
            return Ok(());
        }
        threshold = 1.0 - sign;
    }

    // last command
    let cmd = cli.cmd.join(" ");
    if !cmd.is_empty() {
        cmds.push(cmd.as_str())
    }

    if let Some(c_str) = &cli.cmds {
        cmds.extend(c_str.split(':'));
    }

    let mut content = String::new();
    let _ = cli.cmds_file.map(|f_path| {
        std::fs::File::open(f_path)
            .and_then(|mut f| f.read_to_string(&mut content))
            .unwrap();
        cmds.extend(content.split('\n').filter(|slice| !slice.is_empty()));
    });

    log::debug!("cmds content: {:?}", cmds);

    if cmds.is_empty() {
        log::error!("cmds is empty");
    }

    log::info!("--- Finishing trace ---");

    let mut res_root = "./owl_results".to_string();
    if let Ok(path) = std::env::var("OWL_RES") {
        res_root = path;
    }

    if cli.rand_cmd.is_none() {
        log::error!("--rand-cmd required for leakage test");
        return Ok(());
    }

    let rnd_cmd = cli.rand_cmd.as_ref().unwrap();

    log::info!("Leakage test start");

    let filted = if cmds.len() > 1 {
        let traces = stage1(&cmds, &format!("{res_root}/stage1"));

        stage2(&cmds, traces)
    } else {
        log::warn!("Only one command, skip stage 1 and 2");
        cmds
    };

    stage3(
        filted,
        rnd_cmd,
        &res_root,
        cli.test_times.unwrap_or(2),
        threshold,
    );

    log::info!("Analyze finished");

    Ok(())
}

pub fn leakage_test(
    trace_path: &str,
    cmd: &str,
    rand_cmd: &str,
    times: usize,
    threshold: f64,
) -> Report {
    let mut analyzer = Analyzer {
        // pipe_path: pipe_path.to_owned(),
        fix_cmd: cmd.to_owned(),
        rnd_cmd: rand_cmd.to_owned(),
        times,
        threshold,

        trace_path: trace_path.to_owned(),
        kernels: Default::default(),
    };

    analyzer.test()
}

fn stage1(cmds: &Vec<&str>, trace_path: &str) -> Vec<Trace> {
    log::info!("Stage 1 start");
    cmds.iter()
        .enumerate()
        .map(|(idx, cmd)| get_trace(cmd, &format!("{trace_path}/{idx}")))
        .collect()
}

fn stage2<'a>(cmds: &Vec<&'a str>, mut traces: Vec<Trace>) -> Vec<&'a str> {
    log::info!("Stage 2 start");
    let mut filted = Vec::new();
    filted.push((traces.pop().unwrap(), 0usize));

    traces.into_iter().enumerate().for_each(|(idx, trace)| {
        let idx = idx + 1;
        if let Some((_, same_idx)) = filted.iter().find(|(t2, _)| t2.same(&trace)) {
            log::info!("Find same trace, {idx} and {same_idx}")
        } else {
            filted.push((trace, idx));
        }
    });

    filted.into_iter().map(|(_, idx)| cmds[idx]).collect()
}

fn stage3(cmds: Vec<&str>, rnd_cmd: &str, res_root: &str, times: usize, threshold: f64) {
    log::info!("Stage 3 start");
    for (idx, cmd) in cmds.iter().enumerate() {
        log::info!("test idx: {}, cmd: `{}`", idx, cmd);

        let trace_path = format!("{}/{}", &res_root, idx);

        let report = leakage_test(&trace_path, &cmd, &rnd_cmd, times, threshold);

        let mut f = std::fs::File::create(&format!("{}/report.json", &trace_path)).unwrap();

        log::info!("Dumping report to {}/report.json", &trace_path);
        f.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
            .unwrap();
        log::info!("Report saved to {}/report.json", &trace_path);
    }
}
