//! Chain specific types for polkadot, kusama and westend.

// A big chunk of this file, annotated with `SYNC` should be in sync with the values that are used in
// the real runtimes. Unfortunately, no way to get around them now.
// TODO: There might be a way to create a new crate called `polkadot-runtime-configs` in
// polkadot, that only has `const` and `type`s that are used in the runtime, and we can import
// that.

macro_rules! impl_atomic_u32_parameter_types {
	($mod:ident, $name:ident) => {
		mod $mod {
			use std::sync::atomic::{AtomicU32, Ordering};

			static VAL: AtomicU32 = AtomicU32::new(0);

			pub struct $name;

			impl $name {
				pub fn get() -> u32 {
					VAL.load(Ordering::SeqCst)
				}
			}

			impl<I: From<u32>> frame_support::traits::Get<I> for $name {
				fn get() -> I {
					I::from(Self::get())
				}
			}

			impl $name {
				pub fn set(val: u32) {
					VAL.store(val, std::sync::atomic::Ordering::SeqCst);
				}
			}
		}

		pub use $mod::$name;
	};
}

mod max_weight {
	use std::sync::atomic::{AtomicU64, Ordering};

	static VAL: AtomicU64 = AtomicU64::new(0);

	pub struct MaxWeight;

	impl MaxWeight {
		pub fn get() -> u64 {
			VAL.load(Ordering::SeqCst)
		}
	}

	impl<I: From<u64>> frame_support::traits::Get<I> for MaxWeight {
		fn get() -> I {
			I::from(Self::get())
		}
	}

	impl MaxWeight {
		pub fn set(val: u64) {
			VAL.store(val, std::sync::atomic::Ordering::SeqCst);
		}
	}
}

mod db_weight {
	use frame_support::weights::RuntimeDbWeight;
	use std::sync::atomic::{AtomicU64, Ordering};

	static READ: AtomicU64 = AtomicU64::new(0);
	static WRITE: AtomicU64 = AtomicU64::new(0);

	pub struct DbWeight;

	impl DbWeight {
		pub fn get() -> RuntimeDbWeight {
			RuntimeDbWeight {
				read: READ.load(Ordering::SeqCst),
				write: WRITE.load(Ordering::SeqCst),
			}
		}

		pub fn set(weight: RuntimeDbWeight) {
			READ.store(weight.read, Ordering::SeqCst);
			WRITE.store(weight.write, Ordering::SeqCst)
		}
	}

	impl<I: From<RuntimeDbWeight>> frame_support::traits::Get<I> for DbWeight {
		fn get() -> I {
			I::from(Self::get())
		}
	}
}

use crate::prelude::*;
use frame_support::{traits::ConstU32, weights::Weight, BoundedVec};

pub mod static_types {
	use super::*;

	impl_atomic_u32_parameter_types!(max_length, MaxLength);
	impl_atomic_u32_parameter_types!(max_votes_per_voter, MaxVotesPerVoter);
	pub use db_weight::DbWeight;
	pub use max_weight::MaxWeight;
}

pub mod westend {
	use super::*;

	// SYNC
	frame_election_provider_support::generate_solution_type!(
		#[compact]
		pub struct NposSolution16::<
			VoterIndex = u32,
			TargetIndex = u16,
			Accuracy = sp_runtime::PerU16,
			MaxVoters = ConstU32::<22500>
		>(16)
	);

	#[derive(Debug)]
	pub struct Config;
	impl pallet_election_provider_multi_phase::unsigned::MinerConfig for Config {
		type AccountId = AccountId;
		type MaxLength = static_types::MaxLength;
		type MaxWeight = static_types::MaxWeight;
		type MaxVotesPerVoter = static_types::MaxVotesPerVoter;
		type Solution = NposSolution16;

		// SYNC
		fn solution_weight(v: u32, _t: u32, a: u32, d: u32) -> Weight {
			// feasibility weight.
			(31_722_000 as Weight)
				// Standard Error: 8_000
				.saturating_add((1_255_000 as Weight).saturating_mul(v as Weight))
				// Standard Error: 28_000
				.saturating_add((8_972_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 42_000
				.saturating_add((966_000 as Weight).saturating_mul(d as Weight))
				.saturating_add(static_types::DbWeight::get().reads(4 as Weight))
		}
	}

	#[subxt::subxt(
		runtime_metadata_path = "artifacts/westend.scale",
		derive_for_all_types = "Clone, Debug, Eq, PartialEq",
		derive_for_type(
			type = "pallet_election_provider_multi_phase::RoundSnapshot",
			derive = "Default"
		)
	)]
	pub mod runtime {
		#[subxt(substitute_type = "westend_runtime::NposCompactSolution16")]
		use crate::chain::westend::NposSolution16;

		#[subxt(substitute_type = "sp_arithmetic::per_things::PerU16")]
		use ::sp_runtime::PerU16;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::RawSolution")]
		use ::pallet_election_provider_multi_phase::RawSolution;

		#[subxt(substitute_type = "sp_npos_elections::ElectionScore")]
		use ::sp_npos_elections::ElectionScore;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::Phase")]
		use ::pallet_election_provider_multi_phase::Phase;
	}

	pub use runtime::runtime_types;

	pub type ExtrinsicParams = subxt::PolkadotExtrinsicParamsBuilder<subxt::DefaultConfig>;

	pub type RuntimeApi = runtime::RuntimeApi<
		subxt::DefaultConfig,
		subxt::PolkadotExtrinsicParams<subxt::DefaultConfig>,
	>;

	pub mod epm {
		use super::*;
		pub type BoundedVoters =
			Vec<(AccountId, VoteWeight, BoundedVec<AccountId, static_types::MaxVotesPerVoter>)>;
		pub type Snapshot = (BoundedVoters, Vec<AccountId>, u32);
		pub use super::{
			runtime::election_provider_multi_phase::*,
			runtime_types::pallet_election_provider_multi_phase::*,
		};
	}
}

pub mod polkadot {
	use super::*;

	// SYNC
	frame_election_provider_support::generate_solution_type!(
		#[compact]
		pub struct NposSolution16::<
			VoterIndex = u32,
			TargetIndex = u16,
			Accuracy = sp_runtime::PerU16,
			MaxVoters = ConstU32::<22500>
		>(16)
	);

	#[derive(Debug)]
	pub struct Config;
	impl pallet_election_provider_multi_phase::unsigned::MinerConfig for Config {
		type AccountId = AccountId;
		type MaxLength = static_types::MaxLength;
		type MaxWeight = static_types::MaxWeight;
		type MaxVotesPerVoter = static_types::MaxVotesPerVoter;
		type Solution = NposSolution16;

		// SYNC
		fn solution_weight(v: u32, _t: u32, a: u32, d: u32) -> Weight {
			// feasibility weight.
			(31_722_000 as Weight)
				// Standard Error: 8_000
				.saturating_add((1_255_000 as Weight).saturating_mul(v as Weight))
				// Standard Error: 28_000
				.saturating_add((8_972_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 42_000
				.saturating_add((966_000 as Weight).saturating_mul(d as Weight))
				.saturating_add(static_types::DbWeight::get().reads(4 as Weight))
		}
	}

	#[subxt::subxt(
		runtime_metadata_path = "artifacts/polkadot.scale",
		derive_for_all_types = "Clone, Debug, Eq, PartialEq",
		derive_for_type(
			type = "pallet_election_provider_multi_phase::RoundSnapshot",
			derive = "Default"
		)
	)]
	pub mod runtime {
		#[subxt(substitute_type = "polkadot_runtime::NposCompactSolution16")]
		use crate::chain::polkadot::NposSolution16;

		#[subxt(substitute_type = "sp_arithmetic::per_things::PerU16")]
		use ::sp_runtime::PerU16;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::RawSolution")]
		use ::pallet_election_provider_multi_phase::RawSolution;

		#[subxt(substitute_type = "sp_npos_elections::ElectionScore")]
		use ::sp_npos_elections::ElectionScore;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::Phase")]
		use ::pallet_election_provider_multi_phase::Phase;
	}

	pub use runtime::runtime_types;

	pub type ExtrinsicParams = subxt::PolkadotExtrinsicParamsBuilder<subxt::DefaultConfig>;

	pub type RuntimeApi = runtime::RuntimeApi<
		subxt::DefaultConfig,
		subxt::PolkadotExtrinsicParams<subxt::DefaultConfig>,
	>;

	pub mod epm {
		use super::*;
		pub type BoundedVoters =
			Vec<(AccountId, VoteWeight, BoundedVec<AccountId, static_types::MaxVotesPerVoter>)>;
		pub type Snapshot = (BoundedVoters, Vec<AccountId>, u32);
		pub use super::{
			runtime::election_provider_multi_phase::*,
			runtime_types::pallet_election_provider_multi_phase::*,
		};
	}
}

pub mod kusama {
	use super::*;

	// SYNC
	frame_election_provider_support::generate_solution_type!(
		#[compact]
		pub struct NposSolution24::<
			VoterIndex = u32,
			TargetIndex = u16,
			Accuracy = sp_runtime::PerU16,
			MaxVoters = ConstU32::<12500>
		>(24)
	);

	#[derive(Debug)]
	pub struct Config;
	impl pallet_election_provider_multi_phase::unsigned::MinerConfig for Config {
		type AccountId = AccountId;
		type MaxLength = static_types::MaxLength;
		type MaxWeight = static_types::MaxWeight;
		type MaxVotesPerVoter = static_types::MaxVotesPerVoter;
		type Solution = NposSolution24;

		// SYNC
		fn solution_weight(v: u32, _t: u32, a: u32, d: u32) -> Weight {
			// feasibility weight.
			(31_722_000 as Weight)
				// Standard Error: 8_000
				.saturating_add((1_255_000 as Weight).saturating_mul(v as Weight))
				// Standard Error: 28_000
				.saturating_add((8_972_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 42_000
				.saturating_add((966_000 as Weight).saturating_mul(d as Weight))
				.saturating_add(static_types::DbWeight::get().reads(4 as Weight))
		}
	}

	#[subxt::subxt(
		runtime_metadata_path = "artifacts/kusama.scale",
		derive_for_all_types = "Clone, Debug, Eq, PartialEq",
		derive_for_type(
			type = "pallet_election_provider_multi_phase::RoundSnapshot",
			derive = "Default"
		)
	)]
	pub mod runtime {
		#[subxt(substitute_type = "kusama_runtime::NposCompactSolution24")]
		use crate::chain::kusama::NposSolution24;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::RawSolution")]
		use ::pallet_election_provider_multi_phase::RawSolution;

		#[subxt(substitute_type = "sp_arithmetic::per_things::PerU16")]
		use ::sp_runtime::PerU16;

		#[subxt(substitute_type = "sp_npos_elections::ElectionScore")]
		use ::sp_npos_elections::ElectionScore;

		#[subxt(substitute_type = "pallet_election_provider_multi_phase::Phase")]
		use ::pallet_election_provider_multi_phase::Phase;
	}

	pub use runtime::runtime_types;

	pub type ExtrinsicParams = subxt::PolkadotExtrinsicParamsBuilder<subxt::DefaultConfig>;

	pub type RuntimeApi = runtime::RuntimeApi<
		subxt::DefaultConfig,
		subxt::PolkadotExtrinsicParams<subxt::DefaultConfig>,
	>;

	pub mod epm {
		use super::*;
		pub type BoundedVoters =
			Vec<(AccountId, VoteWeight, BoundedVec<AccountId, static_types::MaxVotesPerVoter>)>;
		pub type Snapshot = (BoundedVoters, Vec<AccountId>, u32);
		pub use super::{
			runtime::election_provider_multi_phase::*,
			runtime_types::pallet_election_provider_multi_phase::*,
		};
	}
}
