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
        /// Initial timestamp
        initial_timestamp: u64,        
    }

    struct Receiver {
        addr : address,
        ratio : FixedPoint32,
    }
    
    /// The `Reserve` resource is in an invalid state
    const E_RESERVE_HAS_BEEN_INITIALIZED: u64 = 0;
    const EMINTING_ZERO_VLS_IS_NOT_ALLOWED: u64 = 3;
    const E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED: u64 = 4;
    const E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED: u64 = 5;
    const E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM: u64 = 6;

    const VLS_SCALING_FACTOR : u64 = 1000000;
    const VLS_TOTAL_AMOUNT: u64 = 100000000 * 1000000;      // 10^8 * 10^6
    const MINING_CAPACITY_PER_MINUTE: u64 = 50 * 1000000;   // 50 * 10^6
    const MINING_PERIOD: u64 = 2 * 365 * 24 * 60;           // two years

    /// Initializes the `VLS` module. 
    /// This function creates the mint, preburn, and burn's capabilities for `VLS` coins and holds them under root account 
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();

        // Operational constraint
        CoreAddresses::assert_currency_info(lr_account);

        // Reserve must not exist.
        assert(!exists<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(E_RESERVE_HAS_BEEN_INITIALIZED));

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
        
        move_to(lr_account, Reserve { mint_cap, burn_cap, preburn_cap, initial_timestamp: 0 });
    }

    public fun initialize_timestamp() 
    acquires Reserve {
        LibraTimestamp::assert_operating();

        let reserve = borrow_global_mut<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        
        assert(reserve.initial_timestamp == 0, Errors::already_published(E_INITIAL_TIMESTAMP_HAS_BEEN_INITIALIED));

        reserve.initial_timestamp = LibraTimestamp::now_seconds();               
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

        assert(amount_vls > 0, Errors::invalid_argument(EMINTING_ZERO_VLS_IS_NOT_ALLOWED));
        
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
        let reserve = borrow_global<Reserve>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let initial_timestamp = reserve.initial_timestamp;
        assert(initial_timestamp != 0, Errors::invalid_argument(E_INITIAL_TIMESTAMP_HAS_NOT_BEEN_INITIALIED));
                
        let now_minutes = (LibraTimestamp::now_seconds() - initial_timestamp) / 60;
        let step = now_minutes / MINING_PERIOD;        
        let process = now_minutes % MINING_PERIOD;
        let mining_capacity = MINING_CAPACITY_PER_MINUTE;
        let expected_amount : u64 = 0;

        while (step > 0) {
            // calculate and accumulate mining amount for every period 
            expected_amount = expected_amount + mining_capacity * MINING_PERIOD;
            
            // mining capacity reduces by half per period 
            mining_capacity = mining_capacity / 2;

            step = step - 1;
        };

        let expected_amount = expected_amount + mining_capacity * process;
                
        // the expected amount mustn't be greater than  VLS_TOTAL_AMOUNT
        if (expected_amount > VLS_TOTAL_AMOUNT)
            expected_amount = VLS_TOTAL_AMOUNT;

        let minted_amount : u64 = (Libra::market_cap<VLS>() as u64);

        assert(minted_amount < VLS_TOTAL_AMOUNT,  Errors::invalid_argument(E_THE_AMOUNT_OF_VLS_HAS_REACHED_MAXIMIUM));

        let mine_amount = expected_amount - minted_amount;
        
        mint(mine_amount)        
    }

    // retrieve all receiver' address and distribution ratio
    public fun get_receivers() : vector<Receiver> {    
        let receivers = Vector::empty<Receiver>();

        let element1 = Receiver { addr: 0xDD01, ratio: FixedPoint32::create_from_rational(71,100) };   //VLS-COMM
        let element2 = Receiver { addr: 0xDD02, ratio: FixedPoint32::create_from_rational(15,100) };   //VLS-ASSOCA
        let element3 = Receiver { addr: 0xDD03, ratio: FixedPoint32::create_from_rational(12,100) };   //VLS-TEAM
        let element4 = Receiver { addr: 0xDD04, ratio: FixedPoint32::create_from_rational(1,100)  };   //VLS-ADVS
        let element5 = Receiver { addr: 0xDD05, ratio: FixedPoint32::create_from_rational(1,100)  };   //VLS-OPEN

        Vector::push_back(&mut receivers, element1);
        Vector::push_back(&mut receivers, element2);
        Vector::push_back(&mut receivers, element3);
        Vector::push_back(&mut receivers, element4);
        Vector::push_back(&mut receivers, element5);
        
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
