address 0x1 {

module VLS {
    use 0x1::AccountLimits;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::Libra::{Self, Libra};
    //use 0x1::LibraAccount;
    use 0x1::LibraTimestamp;
    //use 0x1::Signer;
    use 0x1::Vector;
    
    /// The type tag representing the `VLS` currency on-chain.
    struct VLS { }

    /// VLS holds mint capability for mining
    resource struct Reserve {
        /// The mint capability allowing minting of `VLS` coins.
        mint_cap: Libra::MintCapability<VLS>,
        /// The burn capability for `VLS` coins. This is used for the unpacking
        /// of `VLS` coins into the underlying backing currencies.
        burn_cap: Libra::BurnCapability<VLS>,
        /// The preburn for `VLS`. This is an administrative field since we
        /// need to alway preburn before we burn.
        preburn_cap: Libra::Preburn<VLS>,        
    }

    struct Receiver {
        addr : address,
        ratio : FixedPoint32,
    }

    /// The `Reserve` resource is in an invalid state
    const ERESERVE: u64 = 0;
    const EZERO_VLS_MINT_NOT_ALLOWED: u64 = 3;

    const VLS_TOTAL_AMOUNT:u64 = 100000000;   //10^8
    const MINING_CAPACITY_PER_MINUTE :u64 = 50;
    const MINING_PERIOD :u64 = 2 * 365 * 24 * 60;// two years

    /// Initializes the `VLS` module. This creates the mint, preburn, and burn
    /// capabilities for `VLS` coins
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        //LibraTimestamp::assert_genesis();

        // Operational constraint
        CoreAddresses::assert_currency_info(lr_account);
        // Reserve must not exist.
        assert(!exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ERESERVE));
        let (mint_cap, burn_cap) = Libra::register_currency<VLS>(
            lr_account,
            FixedPoint32::create_from_rational(1, 1), // exchange rate to VLS
            false,    // is_synthetic
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
            b"VLS"
        );

        AccountLimits::publish_unrestricted_limits<VLS>(lr_account);
        let preburn_cap = Libra::create_preburn<VLS>(tc_account);
        
        move_to(lr_account, Reserve { mint_cap, burn_cap, preburn_cap });
    }

    /// Returns true if `CoinType` is `VLS::VLS`
    public fun is_vls<CoinType>(): bool {
        Libra::is_currency<CoinType>() &&
            Libra::currency_code<CoinType>() == Libra::currency_code<VLS>()
    }

    spec fun is_vls {
        pragma verify = false, opaque = true;
        /// The following is correct because currency codes are unique.
        ensures result == spec_is_vls<CoinType>();
    }

    /// Returns true if CoinType is VLS.
    spec define spec_is_vls<CoinType>(): bool {
        type<CoinType>() == type<VLS>()
    }

    /// * If `amount_vls` is zero the function will abort.
    fun mint(
        amount_vls: u64,
    ): Libra<VLS>
    acquires Reserve {              

        assert(amount_vls > 0, Errors::invalid_argument(EZERO_VLS_MINT_NOT_ALLOWED));
        
        let reserve = borrow_global_mut<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());              
                
        // Once the coins have been deposited in the reserve, we can mint the VLS
        Libra::mint_with_capability<VLS>(amount_vls, &reserve.mint_cap)
    }

    spec fun mint {
        pragma opaque;
        modifies global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        modifies global<Libra::CurrencyInfo<VLS>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        include CreateAbortsIf;
        let reserve = global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
                
        ensures exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include Libra::MintEnsures<VLS>{value: amount_vls};
    }

    spec schema CreateAbortsIf {
        amount_vls: u64;
        
        let reserve = global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        aborts_if amount_vls == 0 with Errors::INVALID_ARGUMENT;
        
        include LibraTimestamp::AbortsIfNotOperating;
        
        include Libra::MintAbortsIf<VLS>{value: amount_vls};        
    }

    /// mine VLS, total amount 100,000,000    
    public fun mine() : Libra<VLS>
    acquires Reserve {        
        let expected_amount : u64 = 0;
        let now_minutes = LibraTimestamp::now_seconds() / 60;
        let step = now_minutes / MINING_PERIOD;        
        let process = now_minutes % MINING_PERIOD;
        let mining_capacity = MINING_CAPACITY_PER_MINUTE;

        while (step > 0) {
            // calculate and accumulate mining amount for every period 
            expected_amount = expected_amount + mining_capacity * MINING_PERIOD;
            mining_capacity = mining_capacity / 2;

            step = step - 1;
        };

        let expected_amount = expected_amount + mining_capacity * process;
        
        // the expected amount mustn't be greater than  VLS_TOTAL_AMOUNT
        if (expected_amount > VLS_TOTAL_AMOUNT)
            expected_amount = VLS_TOTAL_AMOUNT;

        let mine_amount = expected_amount - (Libra::market_cap<VLS>() as u64);
        
        mint(mine_amount)        
    }

    public fun get_receivers() : vector<Receiver> {
    //(address, FixedPoint32::FixedPoint32) {
        let receivers = Vector::empty<Receiver>();

        let element = Receiver { addr: CoreAddresses::LIBRA_ROOT_ADDRESS(), ratio: FixedPoint32::create_from_rational(1,2)  };
        
        Vector::push_back(&mut receivers, element);

        receivers
    }

    public fun unpack_receiver(receiver : Receiver) : (address, FixedPoint32) {
        (receiver.addr, *&receiver.ratio)
    }

    // Distribute VLS to all the specified account 
    // public fun distribute() 
    // acquires Receivers, Reserve {
    //     LibraTimestamp::assert_operating();
    //     let _time_seconds = LibraTimestamp::now_seconds();

    //     let vls_coin = mint(100);

    //     let miner = *&borrow_global<Receivers>(CoreAddresses::LIBRA_ROOT_ADDRESS()).miner;

    //     //LibraAccount::deposit<VLS>(CoreAddresses::VM_RESERVED_ADDRESS(), receivers.miner, vls_coin, x"", x"")
    //      // Deposit the `to_deposit` coin
    //     //Libra::deposit(LibraAccount::balance<VLS>(miner), vls_coin);
    //     //move_to(miner, LibraAccount::Balance<VLS>{ coin: Libra::zero<VLS>() });

    // }
}

}
