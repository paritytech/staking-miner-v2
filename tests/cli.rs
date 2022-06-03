use assert_cmd::cargo::cargo_bin;

fn get_staking_miner_name() -> &'static str {
	env!("CARGO_PKG_NAME")
}

#[test]
fn cli_version_works() {
	let output = assert_cmd::Command::new(cargo_bin(get_staking_miner_name()))
		.arg("--version")
		.output()
		.unwrap();

	assert!(output.status.success(), "command returned with non-success exit code");
	let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();

	assert_eq!(version, format!("{} {}", get_staking_miner_name(), env!("CARGO_PKG_VERSION")));
}

/// Requires a `polkadot` binary to in the path to run integration tests against.
//#[cfg(feature = "slow-tests")]
mod slow_tests {
	use super::*;
	use std::{
		io::{BufRead, BufReader, Read},
		ops::{Deref, DerefMut},
		process::{self, Child},
	};
	use tracing_subscriber::EnvFilter;

	#[test]
	fn polkadot_dry_run() {
		let _ = tracing_subscriber::fmt()
			.with_env_filter(EnvFilter::from_default_env())
			.try_init();
		test_dry_run("polkadot-dev");
	}

	#[test]
	fn kusama_dry_run() {
		test_dry_run("kusama-dev");
	}

	#[test]
	fn westend_dry_run() {
		test_dry_run("westend-dev");
	}

	#[test]
	fn polkadot_monitor_works() {
		let _ = tracing_subscriber::fmt()
			.with_env_filter(EnvFilter::from_default_env())
			.try_init();

		let (_cmd, ws_url) = run_polkadot_node_on_chain("polkadot-dev");

		let mut s_cmd = KillChildOnDrop(
			process::Command::new(get_staking_miner_name())
				.stdout(process::Stdio::piped())
				.stderr(process::Stdio::piped())
				.args(&[
					"--uri",
					"ws://127.0.0.1:9944",
					"--seed-or-path",
					"//Alice",
					"monitor",
					"seq-phragmen",
				])
				.spawn()
				.unwrap(),
		);

		let stderr = s_cmd.stderr.take().unwrap();

		find_mined_solution(stderr).unwrap();
	}

	fn run_polkadot_node_on_chain(chain: &str) -> (KillChildOnDrop, String) {
		let mut cmd = KillChildOnDrop(
			process::Command::new("polkadot")
				.stdout(process::Stdio::piped())
				.stderr(process::Stdio::piped())
				.args(&[
					"--chain",
					&chain,
					"--tmp",
					"--alice",
					"--execution",
					"Native",
					"--offchain-worker=Always",
				])
				.spawn()
				.unwrap(),
		);

		let stderr = cmd.stderr.take().unwrap();

		let (ws_url, _) = find_ws_url_from_output(stderr);

		(cmd, ws_url)
	}

	fn test_dry_run(chain: &str) {
		let crate_name = env!("CARGO_PKG_NAME");

		let (_cmd, ws_url) = run_polkadot_node_on_chain(chain);

		let output = assert_cmd::Command::new(cargo_bin(crate_name))
			.args(&["--uri", &ws_url, "--seed-or-path", "//Alice", "dry-run", "seq-phragmen"])
			.output()
			.unwrap();

		println!("output: {}", String::from_utf8(output.stdout).unwrap());
		assert!(output.status.success());
	}

	/// Read the WS address from the output.
	///
	/// This is hack to get the actual binded sockaddr because
	/// substrate assigns a random port if the specified port was already binded.
	fn find_ws_url_from_output(read: impl Read + Send) -> (String, String) {
		let mut data = String::new();

		let ws_url = BufReader::new(read)
			.lines()
			.find_map(|line| {
				let line =
					line.expect("failed to obtain next line from stdout for WS address discovery");
				log::info!("{}", line);

				data.push_str(&line);

				// does the line contain our port (we expect this specific output from substrate).
				let sock_addr = match line.split_once("Running JSON-RPC WS server: addr=") {
					None => return None,
					Some((_, after)) => after.split_once(",").unwrap().0,
				};

				Some(format!("ws://{}", sock_addr))
			})
			.expect("We should get a WebSocket address");

		(ws_url, data)
	}

	fn find_mined_solution(read: impl Read + Send) -> Result<(), String> {
		let now = std::time::Instant::now();
		const MAX_WAIT: u64 = 15 * 60;

		let mut lines = String::new();

		for line in BufReader::new(read).lines() {
			let line = line.map_err(|e| e.to_string())?;

			log::info!("{}", line);

			lines.push_str(&line);

			if now.elapsed().as_secs() > MAX_WAIT {
				return Err("Max wait time exceeded for a solution to complete".to_string())
			}

			if line.contains("Included at") || line.contains("Finalized at") {
				return Ok(())
			}
		}

		Err(format!("All output consumed; previous data: {}", lines))
	}

	pub struct KillChildOnDrop(pub Child);

	impl Drop for KillChildOnDrop {
		fn drop(&mut self) {
			let _ = self.0.kill();
		}
	}

	impl Deref for KillChildOnDrop {
		type Target = Child;

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	impl DerefMut for KillChildOnDrop {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.0
		}
	}
}
