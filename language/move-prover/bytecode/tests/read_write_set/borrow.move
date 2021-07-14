address 0x1 {
module Borrow {
    // TODO: figure out how to allow this dependency
    // ensure that borrows get counted as reads when appropriate
    //use 0x1::Vector;

    struct S has key { }

    // expected: read a/S
    fun borrow_s(a: address) acquires S {
        _ = borrow_global<S>(a)
    }

    // expected: read a/S
    fun borrow_s_mut(a: address) acquires S {
        _ = borrow_global_mut<S>(a)
    }

    /*// expected: read v/size
    fun borrow_vec(v: &vector<u64>) {
        let _ = Vector::borrow(v, 7);
    }

    // expected: read v/size
    fun borrow_vec_mut(v: &vector<u64>) {
        let _ = Vector::borrow_mut(v, 7);
    }*/

}
}
