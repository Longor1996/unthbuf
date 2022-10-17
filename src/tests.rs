//! Modules containing tests.

#[cfg(test)]
use crate::*;

#[cfg(test)]
const PRIMES: &[usize] = &[2, 5, 13, 29, 61, 113, 251, 509, 1021, 2039, 4093, 8179, 16381, 32749, 65521, 131063, 262139, 524269, 1048573, 2097143, 4194301, 8388593, 16777213, 33554393, 67108859, 134217689, 268435399, 536870909, 1073741789, 2147483629, 4294967291, 8589934583, 17179869143, 34359738337, 68719476731, 137438953447, 274877906899, 549755813881, 1099511627689, 2199023255531, 4398046511093, 8796093022151, 17592186044399, 35184372088777, 70368744177643, 140737488355213, 281474976710597, 562949953421231, 1125899906842597, 2251799813685119, 4503599627370449, 9007199254740881, 18014398509481951, 36028797018963913, 72057594037927931, 144115188075855859, 288230376151711717, 576460752303423433, 1152921504606846883, 2305843009213693921, 4611686018427387847, 9223372036854775783, 18446744073709551557];

#[test]
#[ignore = "simply prints all valid indices"]
fn aligned_location_of() {
    for bits in 1..=64 {
        let buf = UnthBuf::<true>::new(4096, bits);
        println!("--- {bits} BITS");
        
        for loc in buf.get_indices() {
            let loc = buf.aligned_location_of(loc as usize);
            print!("{loc:?} ")
        }
        println!();
    }
}

#[test]
#[ignore = "simply prints all valid indices"]
fn unaligned_location_of() {
    for bits in 1..=64 {
        let buf = UnthBuf::<false>::new(4096, bits);
        println!("--- {bits} BITS");
        
        for loc in buf.get_indices() {
            let loc = buf.unaligned_location_of(loc as usize);
            print!("{loc:?} ")
        }
        println!();
    }
}

#[test]
fn aligned_setget() {
    
    for prime
    in PRIMES.iter().copied().chain(
        (1..64usize).map(|b| 2usize.pow(b as u32)-1)
    ) {
        let bits: u8 = match prime.checked_next_power_of_two() {
            Some(pot) => pot.trailing_zeros() as u8 + 1,
            None => 64
        };
        
        let mut buf = UnthBuf::<true>::new_from_capacity_and_iter(4096, bits, std::iter::repeat(prime));
        if ! buf.fits(prime) {continue;}
        
        println!("--- {bits} BITS / Value {prime}");
        for idx in buf.get_indices() {
            let in_prime = prime;
            buf.set(idx, in_prime).unwrap();
            let out_prime = buf.get(idx).unwrap();
            let loc = buf.aligned_location_of(idx);
            debug_assert!(
                in_prime == out_prime,
                "{in_prime} != {out_prime} {loc:?} = {:b}", buf.data[loc.cell]
            )
        }
    }
    
}


#[test]
fn unaligned_setget() {
    
    for prime
    in PRIMES.iter().copied().chain(
        (1..64usize).map(|b| 2usize.pow(b as u32)-1)
    ) {
        let bits: u8 = match prime.checked_next_power_of_two() {
            Some(pot) => (pot - 1).count_ones() as u8,
            None => 64
        };
        
        let mut buf = UnthBuf::<false>::new_from_capacity_and_iter(4096, bits, std::iter::repeat(prime));
        if ! buf.fits(prime) {continue;}
        
        println!("--- {bits} BITS / Value {prime}");
        for idx in buf.get_indices() {
            let in_prime = prime;
            buf.set(idx, in_prime).unwrap();
            let out_prime = buf.get(idx).unwrap();
            let loc = buf.unaligned_location_of(idx);
            debug_assert!(
                in_prime == out_prime,
                "{in_prime} != {out_prime} {loc:?} = {:b}{:b}",
                buf.data[loc.cell+1], buf.data[loc.cell]
            )
        }
    }
    
}


#[cfg(test)]
const BITSIZE: u8 = 5;

#[cfg(test)]
const RNG_SEED: u64 = 134217728;

#[cfg(test)]
const ITERATIONS: usize = 512usize.pow(3);//134217728; // 100000000;

#[cfg(test)]
type ValType = u8;

#[cfg(test)]
fn test_values(bitsize: u8, rng: &mut rand::prelude::StdRng) -> Vec<ValType> {
    use rand::prelude::*;
    let mut values = vec![0u8; ITERATIONS];
    values.fill_with(|| rng.gen_range(0..2_usize.pow(bitsize as u32)) as ValType);
    values
}

#[cfg(test)]
fn test_indices(rng: &mut rand::prelude::StdRng) -> Vec<usize> {
    use rand::prelude::*;
    let mut indices: Vec<usize> = (0..ITERATIONS).collect();
    //indices.shuffle(rng);
    indices
}

#[test]
pub fn bench_io_unaligned() {
    use std::time::Instant;
    use rand::prelude::*;
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(RNG_SEED);
    
    let n = ITERATIONS;
    let bitsize = BITSIZE;
    let values = test_values(bitsize, &mut rng);
    let indices = test_indices(&mut rng);
    
    // init bench
    println!();
    let now = Instant::now();
    let mut packed = UnthBuf::<false>::new_from_capacity_and_iter(
        values.len(),
        bitsize,
        values.iter().copied().map(|v|v as usize)
    );
    let elapsed = now.elapsed();
    println!("Initia. {} unaligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // write bench
    let values = test_values(bitsize, &mut rng);
    let now = Instant::now();
    for i in indices.iter().copied() {
        packed.set(i, values[i] as usize).unwrap();
    }
    let elapsed = now.elapsed();
    println!("Writing {} unaligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // read bench
    let mut values = vec![0 as ValType; n];
    let now = Instant::now();
    for i in indices.iter().copied() {
        values[i] = packed.get(i).unwrap() as ValType;
    }
    let elapsed = now.elapsed();
    println!("Reading {} unaligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
}


#[test]
pub fn bench_io_aligned() {
    use std::time::Instant;
    use rand::prelude::*;
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(RNG_SEED);
    
    let n = ITERATIONS;
    let bitsize = BITSIZE;
    let values = test_values(bitsize, &mut rng);
    let indices = test_indices(&mut rng);
    
    // init bench
    println!();
    let now = Instant::now();
    let mut packed = UnthBuf::<true>::new_from_capacity_and_iter(
        values.len(),
        bitsize,
        values.iter().copied().map(|v|v as usize)
    );
    let elapsed = now.elapsed();
    println!("Initia. {}   aligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // write bench
    let values = test_values(bitsize, &mut rng);
    let now = Instant::now();
    for i in indices.iter().copied() {
        packed.set(i, values[i] as usize).unwrap();
    }
    let elapsed = now.elapsed();
    println!("Writing {}   aligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // read bench
    let mut values = vec![0 as ValType; n];
    let now = Instant::now();
    for i in indices.iter().copied() {
        values[i] = packed.get(i).unwrap() as ValType;
    }
    let elapsed = now.elapsed();
    println!("Reading {}   aligned values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
}




#[test]
pub fn bench_io_baseline() {
    use std::time::Instant;
    use rand::prelude::*;
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(RNG_SEED);
    
    let n = ITERATIONS;
    let bitsize = BITSIZE;
    
    let values = test_values(bitsize, &mut rng);
    let indices = test_indices(&mut rng);
    
    // init bench
    println!();
    let now = Instant::now();
    let mut packed = Vec::<ValType>::from_iter(values.iter().copied().map(|v|v as ValType));
    let elapsed = now.elapsed();
    println!("Initia. {}  baseline values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // write bench
    let values = test_values(bitsize, &mut rng);
    let now = Instant::now();
    for i in indices.iter().copied() {
        *packed.get_mut(i).unwrap() = values[i];
    }
    let elapsed = now.elapsed();
    println!("Writing {}  baseline values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
    
    // read bench
    let mut values = vec![0 as ValType; n];
    let now = Instant::now();
    for i in indices.iter().copied() {
        values[i] = *packed.get(i).unwrap();
    }
    let elapsed = now.elapsed();
    println!("Reading {}  baseline values took {} ms / {} ns per int.", n, elapsed.as_millis(), elapsed.div_f64(n as f64).as_nanos());
}
