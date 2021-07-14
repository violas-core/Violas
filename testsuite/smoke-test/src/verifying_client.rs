// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{diem_swarm_utils::get_json_rpc_url, setup_swarm_and_client_proxy},
};
use cli::client_proxy::ClientProxy;
use diem_client::{
    Client, InMemoryStorage, MethodRequest, MethodResponse, Response, Result, VerifyingClient,
};
use diem_types::{
    account_address::AccountAddress,
    account_config::constants::addresses::{
        diem_root_address, testnet_dd_account_address, treasury_compliance_account_address,
        validator_set_address,
    },
    event::{EventHandle, EventKey},
    transaction::Version,
    trusted_state::TrustedState,
};
use proptest::{collection::vec, prelude::*, sample::select};
use std::cmp::max;
use tokio::runtime::Builder;

struct Environment {
    _env: SmokeTestEnvironment,
    pub max_batch_size: usize,
    pub client_proxy: ClientProxy,
    pub client: Client,
    pub verifying_client: VerifyingClient<InMemoryStorage>,
}

impl Environment {
    fn new() -> Self {
        let (env, client_proxy) = setup_swarm_and_client_proxy(1, 0);

        let max_batch_size = env
            .validator_swarm
            .get_node(0)
            .unwrap()
            .config()
            .json_rpc
            .batch_size_limit as usize;

        let url = get_json_rpc_url(&env.validator_swarm, 0);
        let client = Client::new(url);

        let genesis_waypoint = env.validator_swarm.config.waypoint;
        let trusted_state = TrustedState::from(genesis_waypoint);
        let storage = InMemoryStorage::new();
        let verifying_client =
            VerifyingClient::new_with_state(client.clone(), trusted_state, storage);

        Self {
            _env: env,
            max_batch_size,
            client_proxy,
            client,
            verifying_client,
        }
    }

    fn fund_new_account(&mut self) -> AccountAddress {
        let account = self.client_proxy.create_next_account(false).unwrap();
        let idx = format!("{}", account.index);
        self.client_proxy
            .mint_coins(&["mintb", &idx, "1000", "XUS"], true)
            .unwrap();
        account.address
    }

    fn latest_observed_version(&self) -> Version {
        let version_nv = self
            .client
            .last_known_state()
            .map(|state| state.version)
            .unwrap_or(0);
        let version_v = self.verifying_client.version();
        max(version_nv, version_v)
    }

    /// Send a request using both verifying and non-verifying clients.
    async fn request(
        &self,
        request: MethodRequest,
    ) -> (
        Result<Response<MethodResponse>>,
        Result<Response<MethodResponse>>,
    ) {
        let send_nv = self.client.request(request.clone());
        let send_v = self.verifying_client.request(request);

        tokio::join!(send_nv, send_v)
    }

    /// Send a request batch using both verifying and non-verifying clients.
    async fn batch(
        &self,
        batch: Vec<MethodRequest>,
    ) -> (
        Result<Vec<Result<Response<MethodResponse>>>>,
        Result<Vec<Result<Response<MethodResponse>>>>,
    ) {
        let send_nv = self.client.batch(batch.clone());
        let send_v = self.verifying_client.batch(batch);

        tokio::join!(send_nv, send_v)
    }

    /// Scan the chain for some event handles. This is not too complicated; we
    /// just look up the first few handle ids for each account that we know about.
    async fn scan_event_handles(&self, accounts: &[AccountAddress]) -> Vec<EventHandle> {
        let mut event_handles = Vec::new();

        for account in accounts {
            // scan for a few handle ids
            for id in 0..10 {
                // TODO(philiphayes): add Order to get_events to allow us to just
                // look up latest event.
                let key = EventKey::new_from_address(account, id);
                let start_seq = 0;
                let limit = 1000;
                let events = self
                    .client
                    .get_events(key, start_seq, limit)
                    .await
                    .unwrap()
                    .into_inner();

                let count = events.len() as u64;
                if count != 0 {
                    event_handles.push(EventHandle::new(key, count));
                }
            }
        }

        event_handles
    }
}

/// Running a request against a `VerifyingClient` and non-verifying `Client`
/// at the same state version should always return the same response.
fn assert_responses_equal(
    recv_nv: Result<Response<MethodResponse>>,
    recv_v: Result<Response<MethodResponse>>,
) {
    match (recv_nv, recv_v) {
        (Ok(resp_nv), Ok(resp_v)) => {
            let (inner_nv, state_nv) = resp_nv.into_parts();
            let (inner_v, state_v) = resp_v.into_parts();

            assert_eq!(state_nv.chain_id, state_v.chain_id);
            assert_eq!(inner_nv, inner_v);
        }
        (Err(_), Err(_)) => (),
        (recv_nv, recv_v) => {
            panic!(
                "client handling mismatch:\n\
                 1. non-verifying client response: {:?}\n\
                 2.     verifying client response: {:?}",
                recv_nv, recv_v,
            );
        }
    }
}

/// Running a batch of requests against a `VerifyingClient` and non-verifying
/// `Client` at the same state version should always return the same responses.
fn assert_batches_equal(
    recv_nv: Result<Vec<Result<Response<MethodResponse>>>>,
    recv_v: Result<Vec<Result<Response<MethodResponse>>>>,
) {
    let (batch_nv, batch_v) = match (recv_nv, recv_v) {
        (Ok(batch_nv), Ok(batch_v)) => (batch_nv, batch_v),
        (Err(_), Err(_)) => return,
        (recv_nv, recv_v) => {
            panic!(
                "client batch handling mismatch:\n\
                 1. non-verifying client batch response: {:?}\n\
                 2.     verifying client batch response: {:?}",
                recv_nv, recv_v,
            );
        }
    };

    if batch_nv.len() != batch_v.len() {
        panic!(
            "clients returned different batch sizes: {} vs {}\n\
             1. non-verifying client batch: {:?}\n\
             2.     verifying client batch: {:?}",
            batch_nv.len(),
            batch_v.len(),
            batch_nv,
            batch_v,
        );
    }

    for (resp_nv, resp_v) in batch_nv.into_iter().zip(batch_v.into_iter()) {
        assert_responses_equal(resp_nv, resp_v);
    }
}

/// A `Strategy` for generating a `MethodRequest` at or before the given version
/// and querying from the given set of accounts.
fn arb_request(
    accounts: &[AccountAddress],
    event_handles: &[EventHandle],
    current_state_version: u64,
) -> impl Strategy<Value = MethodRequest> {
    let arb_account = select(accounts.to_owned());
    let arb_version = prop_oneof! [
        5 => Just(current_state_version),
        // query a historical version
        20 => 0u64..current_state_version,
        // occasionally choose an invalid version
        1 => Just(u64::MAX / 2),
    ];

    let arb_version_and_limit = arb_version.clone().prop_flat_map(move |start_version| {
        // We need to be careful how we choose our limit here, since both requests
        // are handled at different times. In order to avoid a race where
        // start_version..start_version+limit contains more transactions in the
        // second request, we constrain the max limit to always give a valid
        // response no matter the order/timing.
        let max_limit = current_state_version.saturating_sub(start_version) + 1;
        let arb_limit = prop_oneof! [
            // usually pick a normal limit
            20 => 1u64..=max_limit,
            // occasionally pick some weird limits
            1 => Just(0u64),
            1 => Just(u64::MAX / 2),
        ];
        (Just(start_version), arb_limit)
    });
    let arb_include_events = any::<bool>();

    let arb_event_handle = prop_oneof! [
        20 => select(event_handles.to_owned()),
        // occasionally choose some garbage handles
        1 => select(accounts.to_owned()).prop_map(|addr| {
            let key = EventKey::new_from_address(&addr, u64::MAX / 2);
            EventHandle::new(key, 1)
        }),
    ];
    // choose event queries that we know are valid at the time we scanned.
    let arb_events = arb_event_handle.prop_flat_map(|event_handle| {
        let num_events = event_handle.count();
        let arb_seq_num = 0..num_events;
        arb_seq_num.prop_flat_map(move |seq_num| {
            let max_limit = num_events - seq_num;
            let arb_limit = 1..=max_limit;
            (Just(*event_handle.key()), Just(seq_num), arb_limit)
        })
    });

    prop_oneof![
        (arb_account, arb_version).prop_map(|(a, v)| MethodRequest::GetAccount(a, Some(v))),
        (arb_version_and_limit, arb_include_events)
            .prop_map(|((v, l), i)| MethodRequest::GetTransactions(v, l, i)),
        arb_events.prop_map(|(k, s, l)| MethodRequest::GetEvents(k, s, l)),
        Just(MethodRequest::get_currencies()),
    ]
}

/// A `Strategy` for generating a batch of `MethodRequest`s with the added restriction
/// that it will still be accepted by the JSON-RPC server after going through the
/// VerifyingClient request transformation.
fn arb_batch(
    max_batch_size: usize,
    accounts: &[AccountAddress],
    event_handles: &[EventHandle],
    current_state_version: u64,
) -> impl Strategy<Value = Vec<MethodRequest>> {
    vec(
        arb_request(accounts, event_handles, current_state_version),
        1..max_batch_size,
    )
    .prop_filter(
        "batch rejected: actual size too large; won't be accepted by the JSON-RPC server",
        move |batch| {
            let actual_batch_size =
                VerifyingClient::<InMemoryStorage>::actual_batch_size(batch.as_slice());
            actual_batch_size <= max_batch_size
        },
    )
}

#[test]
fn test_client_equivalence() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let mut env = Environment::new();

    // Accounts we can query from
    let accounts = vec![
        // Some standard diem accounts
        diem_root_address(),
        validator_set_address(),
        treasury_compliance_account_address(),
        testnet_dd_account_address(),
        // Fund some new accounts
        env.fund_new_account(),
        env.fund_new_account(),
        env.fund_new_account(),
        // Some random, likely non-existent accounts
        AccountAddress::ZERO,
        AccountAddress::random(),
        AccountAddress::random(),
    ];

    // Scan the accounts for some event handles we can query from
    let event_handles = rt.block_on(env.scan_event_handles(&accounts));

    // Sync the verifying client
    rt.block_on(env.verifying_client.sync()).unwrap();

    // Generate random requests and ensure that both verifying and non-verifying
    // clients handle each request identically.
    proptest!(|(request in arb_request(&accounts, &event_handles, env.latest_observed_version()))| {
        let (recv_nv, recv_v) = rt.block_on(env.request(request));
        assert_responses_equal(recv_nv, recv_v);
    });

    // Do the same but with random request batches instead of single requests.
    proptest!(|(batch in arb_batch(env.max_batch_size, &accounts, &event_handles, env.latest_observed_version()))| {
        let (recv_nv, recv_v) = rt.block_on(env.batch(batch));
        assert_batches_equal(recv_nv, recv_v);
    });
}
