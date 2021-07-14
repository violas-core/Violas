module 0x8675309::A {
    use 0x1::Signer;
    struct T1 has key {v: u64}

    public fun test(account: &signer) acquires T1 {
        borrow_global_mut<T1>(Signer::address_of(account));
        acquires_t1(account);
    }

    fun acquires_t1(account: &signer) acquires T1 {
        T1 { v: _ } = move_from<T1>(Signer::address_of(account));
    }

}
