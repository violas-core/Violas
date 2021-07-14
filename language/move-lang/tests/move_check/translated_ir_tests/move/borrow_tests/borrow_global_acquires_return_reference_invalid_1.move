module 0x8675309::A {
    use 0x1::Signer;
    struct T1 has key {v: u64}

    public fun test1(account: &signer) acquires T1 {
        borrow_acquires_t1(account);
    }

    fun borrow_acquires_t1(account: &signer): &mut T1 acquires T1 {
        borrow_global_mut<T1>(Signer::address_of(account))
    }
}

// check: RET_UNSAFE_TO_DESTROY_ERROR
