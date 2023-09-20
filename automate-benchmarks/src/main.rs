use plotters::coord::Shift;
use plotters::prelude::*;
use std::fs::File;
use std::io;
use std::process::Command;
use std::thread;
use std::{env, time};

use std::time::{Duration, Instant};
use sysinfo::{CpuExt, System, SystemExt};

struct TestConfig {
    name: String,
    command: String,
    wrk_args: Vec<&'static str>,
    script_dir: String,
    script_args: Vec<&'static str>,
    build_step_command: Option<String>,
    build_step_args: Option<Vec<&'static str>>,
}

fn dir_path_to_string(dir_name: &str) -> String {
    let current_dir = env::current_dir().expect("Failed to get the current directory");
    let parent_dir = current_dir.parent().expect("No parent directory found");
    let dir_path = parent_dir.join(dir_name);
    dir_path.to_string_lossy().to_string()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sys = System::new_all();

    let test_configs = vec![
        TestConfig {
            name: "bun-no-params".to_string(),
            wrk_args: vec!["-t12", "-c400", "-d300s", "http://127.0.0.1:3000/"]
                .into_iter()
                .map(|s| s.into())
                .collect(),
            script_dir: dir_path_to_string("bun_no_params"),
            script_args: vec!["index.ts"].into_iter().map(|s| s.into()).collect(),
            build_step_command: None,
            command: "bun".to_string(),
            build_step_args: None,
        },
        TestConfig {
            name: "node-no-params".to_string(),
            wrk_args: vec!["-t12", "-c400", "-d300s", "http://127.0.0.1:3000/"]
                .into_iter()
                .map(|s| s.into())
                .collect(),
            script_dir: dir_path_to_string("node_no_params"),
            script_args: vec!["index.js"].into_iter().map(|s| s.into()).collect(),
            build_step_command: None,
            command: "node".to_string(),
            build_step_args: None,
        },
        TestConfig {
            name: "bun".to_string(),
            wrk_args: vec![
                "-t12",
                "-c400",
                "-d300s",
                "http://127.0.0.1:3000/?q1=1&q2=2&q3=3&q4=4",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
            script_dir: dir_path_to_string("bun"),
            script_args: vec!["index.ts"].into_iter().map(|s| s.into()).collect(),
            build_step_command: None,
            command: "bun".to_string(),
            build_step_args: None,
        },
        TestConfig {
            name: "node".to_string(),
            wrk_args: vec![
                "-t12",
                "-c400",
                "-d300s",
                "http://127.0.0.1:3000/?q1=1&q2=2&q3=3&q4=4",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
            script_dir: dir_path_to_string("node"),
            script_args: vec!["index.js"].into_iter().map(|s| s.into()).collect(),
            build_step_command: None,
            command: "node".to_string(),
            build_step_args: None,
        },
        TestConfig {
            name: "go".to_string(),
            wrk_args: vec![
                "-t12",
                "-c400",
                "-d300s",
                "http://127.0.0.1:3000/?q1=1&q2=2&q3=3&q4=4",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
            script_dir: dir_path_to_string("go"),
            script_args: vec![],
            command: "./server".to_string(),
            build_step_command: Some("go".to_string()),
            build_step_args: vec!["build", "-o", "server"]
                .into_iter()
                .map(|s| s.into())
                .collect(),
        },
        TestConfig {
            name: "rust".to_string(),
            wrk_args: vec![
                "-t12",
                "-c400",
                "-d300s",
                "http://127.0.0.1:3000/?q1=1&q2=2&q3=3&q4=4",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
            script_dir: dir_path_to_string("rust"),
            script_args: vec![],
            command: "./target/release/rust".to_string(),
            build_step_command: Some("cargo".to_string()),
            build_step_args: Some(
                vec!["build", "--release"]
                    .into_iter()
                    .map(|s| s.into())
                    .collect(),
            ),
        },
    ];
    for config in &test_configs {
        println!("Starting {}", config.name);
        let (x, cpu, ram) = run_test(&config, &mut sys)?;

        let root = BitMapBackend::new("cpu_ram_usage.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;
        create_chart(&root, &x, &cpu, &ram)?;
    }

    Ok(())
}
fn create_chart(
    root: &DrawingArea<BitMapBackend, Shift>,
    x: &[u64],
    cpu: &[u64],
    ram: &[u64],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = ChartBuilder::on(root)
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u64..*x.iter().max().unwrap_or(&0), 0u64..100u64)?;

    chart
        .configure_mesh()
        .x_desc("Time (s)")
        .y_desc("Usage (%)")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            x.iter().zip(cpu.iter()).map(|(&x, &y)| (x, y)),
            &RED,
        ))?
        .label("Cpu");

    chart
        .draw_series(LineSeries::new(
            x.iter().zip(ram.iter()).map(|(&x, &y)| (x, y)),
            &BLUE,
        ))?
        .label("Ram");

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

fn run_test(
    config: &TestConfig,
    sys: &mut System,
) -> Result<(Vec<u64>, Vec<u64>, Vec<u64>), Box<dyn std::error::Error>> {
    env::set_current_dir(config.script_dir.clone())?;
    match config.build_step_command.to_owned() {
        Some(cmd) => match config.build_step_args.to_owned() {
            Some(args) => {
                let build_step = Command::new(cmd).args(args).output()?;
                if !build_step.status.success() {
                    println!("build status for {} has failed", config.name);
                }
            }
            None => (),
        },
        None => println!("no build step for {}, continuing", config.name),
    }

    let mut server_process = Command::new(&config.command)
        .args(&config.script_args)
        .spawn()?;
    let pid = server_process.id();
    println!("sleeping 10s to ensure all servers are *shutdown* and build step has completed");
    thread::sleep(time::Duration::from_secs(10));
    let log = File::create(format!("wrk_{}.log", config.name)).expect("failed to open log");
    let mut wrk_command = Command::new("wrk")
        .args(&config.wrk_args)
        .stdout(log)
        .spawn()?;

    let start_time = Instant::now();
    let finish_at = start_time + Duration::new(300, 0);
    let mut finished = false;
    let mut x_values = Vec::new();
    let mut cpu_values = Vec::new();
    let mut ram_values = Vec::new();
    let total_ram = sys.total_memory() as f32;

    // Collect data for the test
    while !finished {
        let used_memory = sys.used_memory() as f32;
        let mut total_percent: f32 = 0.0;
        let mut total_cpus: f32 = 0.0;
        sys.refresh_memory();
        sys.refresh_cpu();
        for cpu in sys.cpus() {
            total_percent = total_percent + cpu.cpu_usage();
            total_cpus = total_cpus + 1.0;
        }
        let elapsed_time = start_time.elapsed().as_secs();
        x_values.push(elapsed_time);
        cpu_values.push((total_percent / total_cpus) as u64);
        ram_values.push((used_memory / total_ram * 100.0) as u64);
        std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        if Instant::now() >= finish_at {
            finished = true;
        }
    }
    wrk_command.wait().expect("failed to finish echo");
    wrk_command.kill().expect("wrk command couldn't be killed");

    //kill_process(&config.name)?;
    server_process.kill()?; // Doesn't seem to work with golang

    println!("{} | wrk completed, server process (PID {}) & wrk killed ( waiting 15 seconds to clear up i/o operations)", config.name,pid);
    thread::sleep(time::Duration::from_secs(15));

    Ok((x_values, cpu_values, ram_values))
}
// No longer needed, go run . does not stop via process.kill()
#[allow(dead_code)]
fn kill_process(name: &String) -> io::Result<()> {
    //let output = Command::new("killport").arg(port.to_string()).output()?;
    let output = Command::new("pkill").arg(name).output()?;
    if !output.status.success() {
        eprintln!(
            "Failed to kill  {} process: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
