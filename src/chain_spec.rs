use primitives::{ed25519, sr25519, Pair};
use primitives::crypto::UncheckedInto;
use akropolis_runtime::{
	AccountId, GenesisConfig, ConsensusConfig, TimestampConfig, BalancesConfig,
	// GrandpaConfig,
	SudoConfig, IndicesConfig,
	// AssetsConfig
};
use substrate_service;
use telemetry::TelemetryEndpoints;

// use sr25519::Public as AccountId;
use ed25519::Public as AuthorityId;

use crate::consts::{CHAIN_NAME, CHAIN_ID, CHAIN_ID_SHORT};
use crate::consts::{CHAIN_TESTNET_NAME, CHAIN_TESTNET_ID, CHAIN_TESTNET_ID_SHORT};

// Note this is the URL for the telemetry server
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const STAGING_TELEMETRY_LEVEL: u8 = 1;
const DEFAULT_PROTOCOL_ID: &str = "dot"; // sup?

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,

	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,

	/// Akropolis test-chain
	Akropolis,
}

fn authority_key(s: &str) -> AuthorityId {
	ed25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn account_key(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

impl Alternative {
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| {
					let alice = authority_key("Alice");
					testnet_genesis(
						vec![alice, authority_key("Alice")],
						vec![account_key("Alice")],
						account_key("Alice"),
					)
				},
				vec![],
				None,
				None,
				None,
				None,
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| {
					testnet_genesis(
						vec![authority_key("Alice"), authority_key("Bob")],
						vec![
							account_key("Alice"),
							account_key("Bob"),
							account_key("Charlie"),
							account_key("Dave"),
							account_key("Eve"),
							account_key("Ferdie"),
						],
						account_key("Alice"),
					)
				},
				vec![],
				None,
				None,
				None,
				None,
			),

			Alternative::Akropolis => {
				ChainSpec::from_genesis(
					CHAIN_TESTNET_NAME,
					CHAIN_TESTNET_ID,
					|| {
						let initial_authorities: Vec<AuthorityId> = vec![
							// 5EE4p6upP21hxqrKZGH1vPr4azoN63eYQT5kszmbKVvK61NL:
							hex!["5f9c380ad795be476350d9b31f5ad771abfe728d918b7e35021259f66da17470"].unchecked_into(),
							// 5FVEDPNip5otFuo47X4JYkfZxUPezf8QuZaYGbHWRidCmgru:
							hex!["9768c811cf000ce59faec3de5a915193ca87e90224142bbc4117a7201e123ee6"].unchecked_into(),
							// 5EJEZLV9UNxv6HpVYxDwfQf7oDtamRU4dXG9gqx1XJ14MzWK:
							hex!["62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8"].unchecked_into(),
						];

						let endowed_accounts: Vec<AccountId> = vec![
							account_key("Alice"),
							account_key("Bob"),
							account_key("Charlie"),
							account_key("Dave"),
							account_key("Eve"),
							account_key("Ferdie"),

							// 5EE4p6upP21hxqrKZGH1vPr4azoN63eYQT5kszmbKVvK61NL:
							hex!["5f9c380ad795be476350d9b31f5ad771abfe728d918b7e35021259f66da17470"].unchecked_into(),
							// 5FVEDPNip5otFuo47X4JYkfZxUPezf8QuZaYGbHWRidCmgru:
							hex!["9768c811cf000ce59faec3de5a915193ca87e90224142bbc4117a7201e123ee6"].unchecked_into(),
							// 5EJEZLV9UNxv6HpVYxDwfQf7oDtamRU4dXG9gqx1XJ14MzWK:
							hex!["62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8"].unchecked_into(),
						];

						// 5EJEZLV9UNxv6HpVYxDwfQf7oDtamRU4dXG9gqx1XJ14MzWK:
						let root_key = hex!["62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8"].unchecked_into();

						testnet_genesis(
							initial_authorities.iter().map(|id| id.clone()).collect(),
							endowed_accounts.iter().map(|id| id.clone()).collect(),
							root_key,
						)
					},
					vec![],
					// telemetry_endpoints:
					Some(TelemetryEndpoints::new(vec![(
						STAGING_TELEMETRY_URL.to_string(),
						STAGING_TELEMETRY_LEVEL,
					)])),
					// protocol_id:
					Some(DEFAULT_PROTOCOL_ID),
					// consensus_engine:
					// Some("aura"),
					None,
					None,
				)
			}
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			CHAIN_TESTNET_ID_SHORT | CHAIN_TESTNET_ID => Some(Alternative::Akropolis),
			"" | "local" => Some(Alternative::LocalTestnet),
			_ => None,
		}
	}
}


fn testnet_genesis(
	initial_authorities: Vec<AuthorityId>,
	endowed_accounts: Vec<AccountId>,
	root_key: AccountId,
) -> GenesisConfig {
	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!(
				"../runtime/wasm/target/wasm32-unknown-unknown/release/akropolis_runtime_wasm.compact.wasm"
			)
			.to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			// minimum_period: 15, // 30 second block time.
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 0,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
			vesting: vec![],
		}),
		// XXX: temporarily disable the grandpa
		// grandpa: Some(GrandpaConfig {
		// 	authorities: initial_authorities.iter().map(|x| (x.clone(), 1)).collect(),
		// }),
		// TODO: assets: Some(AssetsConfig{}),
		// properties: (
		// 	tokenDecimals: 15,
		// 	tokenSymbol: "AKT"
		// ),
		sudo: Some(SudoConfig { key: root_key }),
	}
}
