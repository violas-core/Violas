
<a name="0x1_VLS"></a>

# Module `0x1::VLS`



-  [Struct `VLS`](#0x1_VLS_VLS)
-  [Resource `Reserve`](#0x1_VLS_Reserve)
-  [Struct `Receiver`](#0x1_VLS_Receiver)
-  [Constants](#@Constants_0)
-  [Function `VIOLAS_ASSOCIATION_ADDRESS`](#0x1_VLS_VIOLAS_ASSOCIATION_ADDRESS)
-  [Function `initialize`](#0x1_VLS_initialize)
-  [Function `initialize_timestamp`](#0x1_VLS_initialize_timestamp)
-  [Function `is_vls`](#0x1_VLS_is_vls)
-  [Function `mint`](#0x1_VLS_mint)
-  [Function `mine`](#0x1_VLS_mine)
-  [Function `get_receivers`](#0x1_VLS_get_receivers)
-  [Function `unpack_receiver`](#0x1_VLS_unpack_receiver)


<pre><code><b>use</b> <a href="AccountLimits.md#0x1_AccountLimits">0x1::AccountLimits</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Diem.md#0x1_Diem">0x1::Diem</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="FixedPoint32.md#0x1_FixedPoint32">0x1::FixedPoint32</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_VLS_VLS"></a>

## Struct `VLS`

The type tag representing the <code><a href="VLS.md#0x1_VLS">VLS</a></code> currency on-chain.


<pre><code><b>struct</b> <a href="VLS.md#0x1_VLS">VLS</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_VLS_Reserve"></a>

## Resource `Reserve`

VLS holds mint capability for mining


<pre><code><b>resource</b> <b>struct</b> <a href="VLS.md#0x1_VLS_Reserve">Reserve</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint_cap: <a href="Diem.md#0x1_Diem_MintCapability">Diem::MintCapability</a>&lt;<a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a>&gt;</code>
</dt>
<dd>
 The mint capability allowing minting of <code><a href="VLS.md#0x1_VLS">VLS</a></code> coins.
</dd>
<dt>
<code>burn_cap: <a href="Diem.md#0x1_Diem_BurnCapability">Diem::BurnCapability</a>&lt;<a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a>&gt;</code>
</dt>
<dd>
 The burn capability for <code><a href="VLS.md#0x1_VLS">VLS</a></code> coins. This is used for the unpacking
 of <code><a href="VLS.md#0x1_VLS">VLS</a></code> coins into the underlying backing currencies.
</dd>
<dt>
<code>preburn_cap: <a href="Diem.md#0x1_Diem_Preburn">Diem::Preburn</a>&lt;<a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a>&gt;</code>
</dt>
<dd>
 The preburn for <code><a href="VLS.md#0x1_VLS">VLS</a></code>. This is an administrative field since we
 need to alway preburn before we burn.
</dd>
<dt>
<code>initial_timestamp: u64</code>
</dt>
<dd>
 Initial timestamp
</dd>
</dl>


</details>

<a name="0x1_VLS_Receiver"></a>

## Struct `Receiver`



<pre><code><b>struct</b> <a href="VLS.md#0x1_VLS_Receiver">Receiver</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>addr: address</code>
</dt>
<dd>

</dd>
<dt>
<code>ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_VLS_EMINTING_ZERO_VLS_IS_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_EMINTING_ZERO_VLS_IS_NOT_ALLOWED">EMINTING_ZERO_VLS_IS_NOT_ALLOWED</a>: u64 = 3;
</code></pre>



<a name="0x1_VLS_E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED">E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED</a>: u64 = 4;
</code></pre>



<a name="0x1_VLS_E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED">E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED</a>: u64 = 5;
</code></pre>



<a name="0x1_VLS_E_RESERVE_HAS_BEEN_INITIALIZED"></a>

The <code><a href="VLS.md#0x1_VLS_Reserve">Reserve</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_E_RESERVE_HAS_BEEN_INITIALIZED">E_RESERVE_HAS_BEEN_INITIALIZED</a>: u64 = 0;
</code></pre>



<a name="0x1_VLS_E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM">E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM</a>: u64 = 6;
</code></pre>



<a name="0x1_VLS_MINING_CAPACITY_PER_MINUTE"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_MINING_CAPACITY_PER_MINUTE">MINING_CAPACITY_PER_MINUTE</a>: u64 = 50000000;
</code></pre>



<a name="0x1_VLS_MINING_PERIOD"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_MINING_PERIOD">MINING_PERIOD</a>: u64 = 1051200;
</code></pre>



<a name="0x1_VLS_VLS_SCALING_FACTOR"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_VLS_SCALING_FACTOR">VLS_SCALING_FACTOR</a>: u64 = 1000000;
</code></pre>



<a name="0x1_VLS_VLS_TOTAL_AMOUNT"></a>



<pre><code><b>const</b> <a href="VLS.md#0x1_VLS_VLS_TOTAL_AMOUNT">VLS_TOTAL_AMOUNT</a>: u64 = 100000000000000;
</code></pre>



<a name="0x1_VLS_VIOLAS_ASSOCIATION_ADDRESS"></a>

## Function `VIOLAS_ASSOCIATION_ADDRESS`

The address of Violas association account


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_VIOLAS_ASSOCIATION_ADDRESS">VIOLAS_ASSOCIATION_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_VIOLAS_ASSOCIATION_ADDRESS">VIOLAS_ASSOCIATION_ADDRESS</a>(): address {
    0x564C5302  //'V' 'L' 'S' 02
}
</code></pre>



</details>

<a name="0x1_VLS_initialize"></a>

## Function `initialize`

Initializes the <code><a href="VLS.md#0x1_VLS">VLS</a></code> module.
This function creates the mint, preburn, and burn's capabilities for <code><a href="VLS.md#0x1_VLS">VLS</a></code> coins and holds them under root account


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();

    // Operational constraint
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_currency_info">CoreAddresses::assert_currency_info</a>(lr_account);

    // <a href="VLS.md#0x1_VLS_Reserve">Reserve</a> must not exist.
    <b>assert</b>(!<b>exists</b>&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="VLS.md#0x1_VLS_E_RESERVE_HAS_BEEN_INITIALIZED">E_RESERVE_HAS_BEEN_INITIALIZED</a>));

    <b>let</b> (mint_cap, burn_cap) = <a href="Diem.md#0x1_Diem_register_currency">Diem::register_currency</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;(
        lr_account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="VLS.md#0x1_VLS">VLS</a>
        <b>false</b>,    // is_synthetic
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
        b"<a href="VLS.md#0x1_VLS">VLS</a>"
    );

    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;(lr_account);
    <b>let</b> preburn_cap = <a href="Diem.md#0x1_Diem_create_preburn">Diem::create_preburn</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;(tc_account);

    move_to(lr_account, <a href="VLS.md#0x1_VLS_Reserve">Reserve</a> { mint_cap, burn_cap, preburn_cap, initial_timestamp: 0 });
}
</code></pre>



</details>

<a name="0x1_VLS_initialize_timestamp"></a>

## Function `initialize_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_initialize_timestamp">initialize_timestamp</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_initialize_timestamp">initialize_timestamp</a>()
<b>acquires</b> <a href="VLS.md#0x1_VLS_Reserve">Reserve</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();

    <b>let</b> reserve = borrow_global_mut&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());

    <b>assert</b>(reserve.initial_timestamp == 0, <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="VLS.md#0x1_VLS_E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED">E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED</a>));

    reserve.initial_timestamp = <a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>();
}
</code></pre>



</details>

<a name="0x1_VLS_is_vls"></a>

## Function `is_vls`

Returns true if <code>CoinType</code> is <code><a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_is_vls">is_vls</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_is_vls">is_vls</a>&lt;CoinType&gt;(): bool {
    <a href="Diem.md#0x1_Diem_is_currency">Diem::is_currency</a>&lt;CoinType&gt;() &&
        <a href="Diem.md#0x1_Diem_currency_code">Diem::currency_code</a>&lt;CoinType&gt;() == <a href="Diem.md#0x1_Diem_currency_code">Diem::currency_code</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;()
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> verify = <b>false</b>, opaque = <b>true</b>;
</code></pre>


The following is correct because currency codes are unique.


<pre><code><b>ensures</b> result == <a href="VLS.md#0x1_VLS_spec_is_vls">spec_is_vls</a>&lt;CoinType&gt;();
</code></pre>


Returns true if CoinType is VLS.


<a name="0x1_VLS_spec_is_vls"></a>


<pre><code><b>define</b> <a href="VLS.md#0x1_VLS_spec_is_vls">spec_is_vls</a>&lt;CoinType&gt;(): bool {
   type&lt;CoinType&gt;() == type&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_VLS_mint"></a>

## Function `mint`

* If <code>amount_vls</code> is zero the function will abort.


<pre><code><b>fun</b> <a href="VLS.md#0x1_VLS_mint">mint</a>(amount_vls: u64): <a href="Diem.md#0x1_Diem_Diem">Diem::Diem</a>&lt;<a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="VLS.md#0x1_VLS_mint">mint</a>(
    amount_vls: u64,
): <a href="Diem.md#0x1_Diem">Diem</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;
<b>acquires</b> <a href="VLS.md#0x1_VLS_Reserve">Reserve</a> {

    <b>assert</b>(amount_vls &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="VLS.md#0x1_VLS_EMINTING_ZERO_VLS_IS_NOT_ALLOWED">EMINTING_ZERO_VLS_IS_NOT_ALLOWED</a>));

    <b>let</b> reserve = borrow_global_mut&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());

    // Once the coins have been deposited in the reserve, we can mint the <a href="VLS.md#0x1_VLS">VLS</a>
    <a href="Diem.md#0x1_Diem_mint_with_capability">Diem::mint_with_capability</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;(amount_vls, &reserve.mint_cap)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>modifies</b> <b>global</b>&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
<b>modifies</b> <b>global</b>&lt;<a href="Diem.md#0x1_Diem_CurrencyInfo">Diem::CurrencyInfo</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>include</b> <a href="VLS.md#0x1_VLS_CreateAbortsIf">CreateAbortsIf</a>;
<a name="0x1_VLS_reserve$10"></a>
<b>let</b> reserve = <b>global</b>&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
<b>include</b> <a href="Diem.md#0x1_Diem_MintEnsures">Diem::MintEnsures</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;{value: amount_vls};
</code></pre>




<a name="0x1_VLS_CreateAbortsIf"></a>


<pre><code><b>schema</b> <a href="VLS.md#0x1_VLS_CreateAbortsIf">CreateAbortsIf</a> {
    amount_vls: u64;
    <a name="0x1_VLS_reserve$9"></a>
    <b>let</b> reserve = <b>global</b>&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
    <b>aborts_if</b> amount_vls == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotOperating">DiemTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Diem.md#0x1_Diem_MintAbortsIf">Diem::MintAbortsIf</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;{value: amount_vls};
}
</code></pre>



</details>

<a name="0x1_VLS_mine"></a>

## Function `mine`

mine VLS, total amount 100,000,000


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_mine">mine</a>(): <a href="Diem.md#0x1_Diem_Diem">Diem::Diem</a>&lt;<a href="VLS.md#0x1_VLS_VLS">VLS::VLS</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_mine">mine</a>() : <a href="Diem.md#0x1_Diem">Diem</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;
<b>acquires</b> <a href="VLS.md#0x1_VLS_Reserve">Reserve</a> {
    <b>let</b> reserve = borrow_global&lt;<a href="VLS.md#0x1_VLS_Reserve">Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
    <b>let</b> initial_timestamp = reserve.initial_timestamp;
    <b>assert</b>(initial_timestamp != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="VLS.md#0x1_VLS_E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED">E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED</a>));

    <b>let</b> now_minutes = (<a href="DiemTimestamp.md#0x1_DiemTimestamp_now_seconds">DiemTimestamp::now_seconds</a>() - initial_timestamp) / 60;
    <b>let</b> step = now_minutes / <a href="VLS.md#0x1_VLS_MINING_PERIOD">MINING_PERIOD</a>;
    <b>let</b> process = now_minutes % <a href="VLS.md#0x1_VLS_MINING_PERIOD">MINING_PERIOD</a>;
    <b>let</b> mining_capacity = <a href="VLS.md#0x1_VLS_MINING_CAPACITY_PER_MINUTE">MINING_CAPACITY_PER_MINUTE</a>;
    <b>let</b> expected_amount : u64 = 0;

    <b>while</b> (step &gt; 0) {
        // calculate and accumulate mining amount for every period
        expected_amount = expected_amount + mining_capacity * <a href="VLS.md#0x1_VLS_MINING_PERIOD">MINING_PERIOD</a>;

        // mining capacity reduces by half per period
        mining_capacity = mining_capacity / 2;

        step = step - 1;
    };

    <b>let</b> expected_amount = expected_amount + mining_capacity * process;

    // the expected amount mustn't be greater than  <a href="VLS.md#0x1_VLS_VLS_TOTAL_AMOUNT">VLS_TOTAL_AMOUNT</a>
    <b>if</b> (expected_amount &gt; <a href="VLS.md#0x1_VLS_VLS_TOTAL_AMOUNT">VLS_TOTAL_AMOUNT</a>)
        expected_amount = <a href="VLS.md#0x1_VLS_VLS_TOTAL_AMOUNT">VLS_TOTAL_AMOUNT</a>;

    <b>let</b> minted_amount : u64 = (<a href="Diem.md#0x1_Diem_market_cap">Diem::market_cap</a>&lt;<a href="VLS.md#0x1_VLS">VLS</a>&gt;() <b>as</b> u64);

    <b>assert</b>(minted_amount &lt; <a href="VLS.md#0x1_VLS_VLS_TOTAL_AMOUNT">VLS_TOTAL_AMOUNT</a>,  <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="VLS.md#0x1_VLS_E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM">E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM</a>));

    <b>let</b> mine_amount = expected_amount - minted_amount;

    <a href="VLS.md#0x1_VLS_mint">mint</a>(mine_amount)
}
</code></pre>



</details>

<a name="0x1_VLS_get_receivers"></a>

## Function `get_receivers`

retrieve all receiver' address and distribution ratio


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_get_receivers">get_receivers</a>(): vector&lt;<a href="VLS.md#0x1_VLS_Receiver">VLS::Receiver</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_get_receivers">get_receivers</a>() : vector&lt;<a href="VLS.md#0x1_VLS_Receiver">Receiver</a>&gt; {
    <b>let</b> receivers = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="VLS.md#0x1_VLS_Receiver">Receiver</a>&gt;();

    <b>let</b> element1 = <a href="VLS.md#0x1_VLS_Receiver">Receiver</a> { addr: 0x564C5301, ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(71,100) };   //<a href="VLS.md#0x1_VLS">VLS</a>-COMM, 'V' 'L' 'S' 01
    <b>let</b> element2 = <a href="VLS.md#0x1_VLS_Receiver">Receiver</a> { addr: <a href="VLS.md#0x1_VLS_VIOLAS_ASSOCIATION_ADDRESS">VIOLAS_ASSOCIATION_ADDRESS</a>(), ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(15,100) };   //<a href="VLS.md#0x1_VLS">VLS</a>-ASSOCA, 'V' 'L' 'S' 02
    <b>let</b> element3 = <a href="VLS.md#0x1_VLS_Receiver">Receiver</a> { addr: 0x564C5303, ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(12,100) };   //<a href="VLS.md#0x1_VLS">VLS</a>-TEAM, 'V' 'L' 'S' 03
    <b>let</b> element4 = <a href="VLS.md#0x1_VLS_Receiver">Receiver</a> { addr: 0x564C5304, ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1,100)  };   //<a href="VLS.md#0x1_VLS">VLS</a>-ADVS, 'V' 'L' 'S' 04
    <b>let</b> element5 = <a href="VLS.md#0x1_VLS_Receiver">Receiver</a> { addr: 0x564C5305, ratio: <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1,100)  };   //<a href="VLS.md#0x1_VLS">VLS</a>-OPEN, 'V' 'L' 'S' 05

    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> receivers, element1);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> receivers, element2);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> receivers, element3);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> receivers, element4);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> receivers, element5);

    receivers
}
</code></pre>



</details>

<a name="0x1_VLS_unpack_receiver"></a>

## Function `unpack_receiver`



<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_unpack_receiver">unpack_receiver</a>(receiver: <a href="VLS.md#0x1_VLS_Receiver">VLS::Receiver</a>): (address, <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="VLS.md#0x1_VLS_unpack_receiver">unpack_receiver</a>(receiver : <a href="VLS.md#0x1_VLS_Receiver">Receiver</a>) : (address, <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>) {
    (receiver.addr, *&receiver.ratio)
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions
