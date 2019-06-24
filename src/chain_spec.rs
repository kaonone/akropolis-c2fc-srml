use primitives::{ed25519, sr25519, Pair};
use primitives::crypto::UncheckedInto;
use akropolis_runtime::{/* AuthorityId, */
                    AccountId,
                    GenesisConfig,
                    ConsensusConfig,
                    TimestampConfig,
                    BalancesConfig,
                    GrandpaConfig,
                    SessionConfig,
                    StakingConfig,
                    StakerStatus,
                    SudoConfig,
                    IndicesConfig,
                    // AssetsConfig
                    Perbill};
use substrate_service;
use telemetry::TelemetryEndpoints;
// use sr25519::Public as AccountId;
use ed25519::Public as AuthorityId;

use serde_json::json;


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
	ed25519::Pair::from_string(&format!("//{}", s), None).expect("static values are valid; qed")
	                                                     .public()
}

fn account_key(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", s), None).expect("static values are valid; qed")
	                                                     .public()
}

/// AKT token properties.
fn token_props() -> substrate_service::Properties {
	json!({"tokenDecimals": 18, "tokenSymbol": "AKT"}).as_object()
	                                                  .unwrap()
	                                                  .clone()
}

fn get_initial_authorities() -> Vec<(AccountId, AccountId, AuthorityId)> {
	// 5EE4p6upP21hxqrKZGH1vPr4azoN63eYQT5kszmbKVvK61NL: 5f9c380ad795be476350d9b31f5ad771abfe728d918b7e35021259f66da17470
	// 5FVEDPNip5otFuo47X4JYkfZxUPezf8QuZaYGbHWRidCmgru: 9768c811cf000ce59faec3de5a915193ca87e90224142bbc4117a7201e123ee6
	// 5EJEZLV9UNxv6HpVYxDwfQf7oDtamRU4dXG9gqx1XJ14MzWK: 62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8

	// TODO: XXX: FIX KEYS

	let aira_auth: AuthorityId =
		hex!["5f9c380ad795be476350d9b31f5ad771abfe728d918b7e35021259f66da17470"].unchecked_into();
	let aira_stash: AccountId =
		hex!["9768c811cf000ce59faec3de5a915193ca87e90224142bbc4117a7201e123ee6"].unchecked_into();
	let aira_control: AccountId =
		hex!["62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8"].unchecked_into();

	let akru_auth: AuthorityId =
		hex!["5f9c380ad795be476350d9b31f5ad771abfe728d918b7e35021259f66da17470"].unchecked_into();
	let akru_stash: AccountId =
		hex!["9768c811cf000ce59faec3de5a915193ca87e90224142bbc4117a7201e123ee6"].unchecked_into();
	let akru_control: AccountId =
		hex!["62ca01ab78f2f3e5ba59c3ddd16c1e4d07eb8021687ce22a202eba4984bc94b8"].unchecked_into();

	vec![(aira_stash, aira_control.clone(), aira_auth),
	     (akru_stash, akru_control.clone(), akru_auth),]
}

impl Alternative {
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Akropolis => {
				ChainSpec::from_genesis(
				                        CHAIN_TESTNET_NAME,
				                        CHAIN_TESTNET_ID,
				                        || {
					                        let initial_authorities = get_initial_authorities();
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
					                                        // initial_authorities.iter().map(|id| id.clone()).collect(),
					                                        initial_authorities,
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
				                        Some(token_props()),
				)
			},

			_ => panic!("noooooo"),
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


fn testnet_genesis(initial_authorities: Vec<(AccountId, AccountId, AuthorityId)>,
                   endowed_accounts: Vec<AccountId>, root_key: AccountId)
                   -> GenesisConfig
{
	const MILLICENTS: u128 = 1_000_000_000;
	const CENTS: u128 = 1_000 * MILLICENTS; // assume this is worth about a cent.
	const DOLLARS: u128 = 100 * CENTS;

	const SECS_PER_BLOCK: u64 = 6;
	const MINUTES: u64 = 60 / SECS_PER_BLOCK;
	const HOURS: u64 = MINUTES * 60;
	const DAYS: u64 = HOURS * 24;

	const ENDOWMENT: u128 = 10_000_000 * DOLLARS;
	const STASH: u128 = 100 * DOLLARS;

	GenesisConfig { consensus: Some(ConsensusConfig { code: include_bytes!(
		"../runtime/wasm/target/wasm32-unknown-unknown/release/akropolis_runtime_wasm.compact.wasm"
	).to_vec(),
	                                                  authorities: initial_authorities.iter()
	                                                                                  .map(|x| x.2.clone())
	                                                                                  .collect(), }),
	                system: None,
	                timestamp: Some(TimestampConfig {
			// minimum_period: 15, // 30 second block time.
			minimum_period: 5, // 10 second block time.
		}),
	                indices: Some(IndicesConfig { ids: endowed_accounts.clone() }),
	                balances: Some(BalancesConfig { // transaction_base_fee: 1 * CENTS,
	                                                // transaction_byte_fee: 10 * MILLICENTS,
	                                                transaction_base_fee: 0 * CENTS,
	                                                transaction_byte_fee: 0 * MILLICENTS,
	                                                existential_deposit: 1 * DOLLARS,
	                                                // transfer_fee: 1 * CENTS, creation_fee: 1 * CENTS,
	                                                transfer_fee: 0 * CENTS,
	                                                creation_fee: 0 * CENTS,
	                                                balances: endowed_accounts.iter()
	                                                                          .cloned()
	                                                                          .map(|k| (k, ENDOWMENT))
	                                                                          .collect(),
	                                                vesting: vec![] }),
	                session: Some(SessionConfig { validators: initial_authorities.iter()
	                                                                             .map(|x| x.1.clone())
	                                                                             .collect(),
	                                              session_length: 15,
	                                              keys: initial_authorities.iter()
	                                                                       .map(|x| (x.1.clone(), x.2.clone()))
	                                                                       .collect::<Vec<_>>() }),
	                staking: Some(StakingConfig { current_era: 0,
	                                              minimum_validator_count: 2,
	                                              validator_count: 7,
	                                              sessions_per_era: 10,
	                                              bonding_duration: 10 * MINUTES,
	                                              current_session_reward: 0,
	                                              session_reward: Perbill::from_millionths(200_000),
	                                              offline_slash: Perbill::from_millionths(1_000_000),
	                                              offline_slash_grace: 4,
	                                              stakers: initial_authorities.iter()
	                                                                          .map(|x| {
		                                                                          (x.0.clone(),
		                                                                           x.1.clone(),
		                                                                           STASH,
		                                                                           StakerStatus::Validator)
		                                                                         })
	                                                                          .collect(),
	                                              invulnerables: initial_authorities.iter()
	                                                                                .map(|x| x.1.clone())
	                                                                                .collect() }),
	                grandpa: Some(GrandpaConfig { // authorities: initial_authorities.iter().map(|x| (x.clone(), 1)).collect(),
	                                              authorities: initial_authorities.iter()
	                                                                              .map(|x| (x.2.clone(), 1))
	                                                                              .collect() }),
	                // TODO: assets: Some(AssetsConfig{}),
	                sudo: Some(SudoConfig { key: root_key }) }
}
