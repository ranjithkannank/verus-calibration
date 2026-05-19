use vstd::prelude::*;


fn main() {}

verus! {

proof fn mul_assoc(x: nat, y: nat, z: nat) by (nonlinear_arith)
    ensures (x * y) * z == y * (x * z)
{
}

}
