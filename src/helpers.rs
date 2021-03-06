use crate::{chain, opt::Solver, prelude::*};
use frame_election_provider_support::{PhragMMS, SequentialPhragmen};
use frame_support::BoundedVec;
use pallet_election_provider_multi_phase::{SolutionOf, SolutionOrSnapshotSize};
use sp_npos_elections::ElectionScore;

macro_rules! mine_solution_for {
	($runtime:tt) => {
		paste::paste! {
				/// The monitor command.
				pub(crate) async fn [<mine_solution_$runtime>](
					api: &chain::$runtime::RuntimeApi,
					hash: Option<Hash>,
					solver: Solver
				) -> Result<(SolutionOf<chain::$runtime::Config>, ElectionScore, SolutionOrSnapshotSize), Error> {

						let (voters, targets, desired_targets) = [<snapshot_$runtime>](&api, hash).await?;

						let blocking_task = tokio::task::spawn_blocking(move || {
							match solver {
								Solver::SeqPhragmen { iterations } => {
									BalanceIterations::set(iterations);
									Miner::<chain::$runtime::Config>::mine_solution_with_snapshot::<
										SequentialPhragmen<AccountId, Accuracy, Balancing>,
									>(voters, targets, desired_targets)
								},
								Solver::PhragMMS { iterations } => {
									BalanceIterations::set(iterations);
									Miner::<chain::$runtime::Config>::mine_solution_with_snapshot::<PhragMMS<AccountId, Accuracy, Balancing>>(
										voters,
										targets,
										desired_targets,
									)
								},
							}
						}).await;

						match blocking_task {
							Ok(Ok(res)) => Ok(res),
							Ok(Err(err)) => Err(Error::Other(format!("{:?}", err))),
							Err(err) => Err(Error::Other(format!("{:?}", err))),
						}
				}
		}
	};
}

macro_rules! snapshot_for { ($runtime:tt) => {
	paste::paste! {
	pub async fn [<snapshot_$runtime>](api: &chain::$runtime::RuntimeApi, hash: Option<Hash>) -> Result<crate::chain::$runtime::epm::Snapshot, Error> {
		use crate::chain::static_types;
		use crate::chain::$runtime::epm::RoundSnapshot;

		let RoundSnapshot { voters, targets } = api
			.storage()
			.election_provider_multi_phase()
			.snapshot(hash)
			.await?
			.unwrap_or_default();

		let desired_targets = api
			.storage()
			.election_provider_multi_phase()
			.desired_targets(hash)
			.await?
			.unwrap_or_default();


		let voters: Vec<_> = voters
			.into_iter()
			.map(|(a, b, mut c)| {
				let mut bounded_vec: BoundedVec<AccountId, static_types::MaxVotesPerVoter> = BoundedVec::default();
				// If this fails just crash the task.
				bounded_vec.try_append(&mut c.0).unwrap_or_else(|_| panic!("BoundedVec capacity: {} failed; `MinerConfig::MaxVotesPerVoter` is different from the chain data; this is a bug please file an issue", static_types::MaxVotesPerVoter::get()));
				(a, b, bounded_vec)
			})
			.collect();

			Ok((voters, targets, desired_targets))
	}
}}}

/// Warning these variables are in thread local storage and may not be accessible when doing
/// `tokio::spawn` or `thread::spawn`.
macro_rules! tls_update_runtime_constants {
	($runtime:tt) => {
		paste::paste! {
			pub fn [<tls_update_runtime_constants_$runtime>](api: &chain::$runtime::RuntimeApi) {
					use chain::static_types;
					use sp_runtime::Perbill;

					const PROOF: &str = "Metadata is outdated; open an issue in staking-miner";

					// maximum weight of the signed submission is exposed from metadata and MUST be this.
					let max_weight = api.constants().election_provider_multi_phase().signed_max_weight().expect(PROOF);
					// allow up to 75% of the block size to be used for signed submission, length-wise. This
					// value can be adjusted a bit if needed.
					let max_length = Perbill::from_rational(90_u32, 100) * api.constants().system().block_length().expect(PROOF).max.normal;
					let db_weight = api.constants().system().db_weight().expect(PROOF);

					let system_db_weight =
						frame_support::weights::RuntimeDbWeight { read: db_weight.read, write: db_weight.write };

					static_types::DbWeight::set(system_db_weight);
					static_types::MaxWeight::set(max_weight);
					static_types::MaxLength::set(max_length);
					static_types::MaxVotesPerVoter::set(max_length);

					log::trace!(target: LOG_TARGET, "Constant `max_votes_per_voter`: {:?}", static_types::MaxVotesPerVoter::get());
					log::trace!(target: LOG_TARGET, "Constant `db_weight`: {:?}", static_types::DbWeight::get());
					log::trace!(target: LOG_TARGET, "Constant `max_weight`: {:?}", static_types::MaxWeight::get());
					log::trace!(target: LOG_TARGET, "Constant `max_length`: {:?}", static_types::MaxLength::get());
			}
		}
	};
}

mine_solution_for!(polkadot);
mine_solution_for!(kusama);
mine_solution_for!(westend);
snapshot_for!(polkadot);
snapshot_for!(kusama);
snapshot_for!(westend);
tls_update_runtime_constants!(polkadot);
tls_update_runtime_constants!(kusama);
tls_update_runtime_constants!(westend);
