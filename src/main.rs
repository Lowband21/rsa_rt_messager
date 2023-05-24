use num_bigint::{BigUint, RandBigInt, ToBigUint};
use num_traits::{One, ToPrimitive, Zero};
use rand::Rng;
use std::time::Instant;
//extern crate rayon;
use rayon::prelude::*;

fn generate_odd_random_number(bits: u32) -> BigUint {
    let mut rng = rand::thread_rng();
    let mut num = rng.gen_biguint_range(
        &BigUint::from(2u128).pow(bits - 1),
        &BigUint::from(2u128).pow(bits),
    );
    if num.clone() % 2u128 == BigUint::from(0u128) {
        num += BigUint::from(1u128);
    }
    BigUint::from(num)
}

fn jacobi_symbol(mut a: BigUint, mut n: BigUint) -> i32 {
    assert!(n.clone() % 2u8 == 1u64.into());
    let mut s = 1;
    while a != 0u64.into() {
        while a.clone() % 2u8 == 0u64.into() {
            a /= 2u8;
            let n_mod_8: u8 = (&n % 8u8).to_u8().unwrap();
            if n_mod_8 == 3 || n_mod_8 == 5 {
                s = -s;
            }
        }
        std::mem::swap(&mut n, &mut a);
        if (&n % 4u8 == 3u64.into()) && (&a % 4u8 == 3u64.into()) {
            s = -s;
        }
        a %= &n;
    }
    if n == 1u64.into() {
        s
    } else {
        0
    }
}

fn mod_exp(base: BigUint, exponent: BigUint, modulus: BigUint) -> BigUint {
    let mut result: BigUint = BigUint::from(1u64);
    let mut base = base % &modulus;
    let mut exponent = exponent;

    while exponent > 0u8.into() {
        if &exponent % 2u8 == 1u8.into() {
            result = (result * &base) % &modulus;
        }
        base = (&base * &base) % &modulus;
        exponent >>= 1;
    }

    result
}

fn solovay_strassen(n: &BigUint, iterations: u32) -> bool {
    if n == &BigUint::from(2u8) || n == &BigUint::from(3u8) {
        return true;
    }

    let mut rng = rand::thread_rng();
    for _ in 0..iterations {
        let a: BigUint =
            rng.gen_biguint_range(&BigUint::from(2u64), &BigUint::from(n.to_u64_digits()[0]));
        let x = jacobi_symbol(a.clone(), n.clone());
        let expected_result = if x == -1 {
            n - BigUint::one()
        } else {
            BigUint::from(x.abs() as u64)
        };

        if x == 0 || mod_exp(a.clone(), (n - BigUint::one()) >> 1, n.clone()) != expected_result {
            return false;
        }
    }
    true
}

fn main() {
    let now = Instant::now();
    let num_tries = 10000; // number of random numbers to generate and check
    let num_bits = 1024;
    let num_iterations = 55;

    let results: Vec<_> = (0..num_tries)
        .into_par_iter()
        .map(|_| {
            let odd_num = generate_odd_random_number(num_bits);
            let is_prime = solovay_strassen(&odd_num, num_iterations);
            if is_prime {
                println!("Found a large prime");
            }

            (odd_num, is_prime)
        })
        .collect();

    let prime_results: Vec<_> = results
        .into_iter()
        .filter(|(_, is_prime)| *is_prime)
        .collect();

    for i in prime_results.clone() {
        println!("Prime Number: {:?}", i);
    }
    println!(
        "Found {} prime numbers in {} attempts and {}ms",
        prime_results.len(),
        num_tries,
        now.elapsed().as_millis()
    );
}
