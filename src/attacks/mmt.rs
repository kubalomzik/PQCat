use ndarray::{Array1, Array2};
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::time::Instant;

pub fn run_mmt_algorithm(
    h: &Array2<u8>,
    syndrome: &Array1<u8>,
    n: usize,
    weight: usize,
    p: usize,
    l1: usize,
    l2: usize,
) -> Option<Vec<u8>> {
    let start = Instant::now();

    // Ensure p is at least 2 (we need at least 2 partitions)
    let p = p.max(2);

    // Calculate weights for each partition
    let weights: Vec<usize> = if p == 2 {
        // For p=2 case (similar to ball-collision)
        vec![weight / 2, weight - weight / 2]
    } else {
        // For p > 2 case, try to distribute weight evenly
        let base_weight = weight / p;
        let remainder = weight % p;
        (0..p)
            .map(|i| {
                if i < remainder {
                    base_weight + 1
                } else {
                    base_weight
                }
            })
            .collect()
    };

    // Create p partitions of roughly equal size
    let partition_size = n / p;
    let mut partitions: Vec<Vec<usize>> = Vec::with_capacity(p);
    for i in 0..p {
        let start_idx = i * partition_size;
        let end_idx = if i == p - 1 {
            n
        } else {
            (i + 1) * partition_size
        };
        partitions.push((start_idx..end_idx).collect());
    }

    let mut rng = thread_rng();
    let r = h.shape()[0]; // Number of rows in H (syndrome length)
    let syndrome_vec: Vec<u8> = syndrome.iter().copied().collect();

    // ===== PHASE 1: Generate lists L1 and L2 =====

    // Generate list L1 (combinations from first p/2 partitions)
    let mut l1_map: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
    for _ in 0..l1 {
        // Select indices from first half of partitions with appropriate weights
        let mut subset: Vec<usize> = Vec::new();
        for i in 0..(p / 2) {
            let partition = &partitions[i];
            let weight_i = weights[i];
            if weight_i > 0 && !partition.is_empty() {
                subset.extend(
                    partition
                        .choose_multiple(&mut rng, weight_i.min(partition.len()))
                        .cloned(),
                );
            }
        }

        // Calculate partial syndrome
        let mut partial_syndrome = vec![0; r];
        for &idx in &subset {
            for j in 0..r {
                partial_syndrome[j] ^= h[[j, idx]];
            }
        }

        // Store in map
        l1_map
            .entry(partial_syndrome)
            .or_insert_with(Vec::new)
            .push(subset);
    }

    // Generate list L2 (combinations from second p/2 partitions)
    let mut l2_map: HashMap<Vec<u8>, Vec<Vec<usize>>> = HashMap::new();
    for _ in 0..l2 {
        // Select indices from second half of partitions with appropriate weights
        let mut subset: Vec<usize> = Vec::new();
        for i in (p / 2)..p {
            let partition = &partitions[i];
            let weight_i = weights[i];
            if weight_i > 0 && !partition.is_empty() {
                subset.extend(
                    partition
                        .choose_multiple(&mut rng, weight_i.min(partition.len()))
                        .cloned(),
                );
            }
        }

        // Calculate partial syndrome
        let mut partial_syndrome = vec![0; r];
        for &idx in &subset {
            for j in 0..r {
                partial_syndrome[j] ^= h[[j, idx]];
            }
        }

        // Store in map
        l2_map
            .entry(partial_syndrome)
            .or_insert_with(Vec::new)
            .push(subset);
    }

    // ===== PHASE 2: Search for a match =====

    // For each syndrome in L1, look for matching syndrome in L2
    for (s1, subsets1) in &l1_map {
        // Calculate the complementary syndrome
        let mut target_s2 = syndrome_vec.clone();
        for i in 0..r {
            target_s2[i] ^= s1[i];
        }

        // Check if we have the complementary syndrome in L2
        if let Some(subsets2) = l2_map.get(&target_s2) {
            // Try combinations
            for subset1 in subsets1 {
                for subset2 in subsets2 {
                    // Combine subsets to form error vector
                    let mut error = vec![0; n];
                    for &idx in subset1.iter().chain(subset2.iter()) {
                        error[idx] = 1;
                    }

                    // Verify weight and syndrome
                    let actual_weight = error.iter().filter(|&&bit| bit == 1).count();
                    if actual_weight == weight {
                        // Calculate full syndrome to verify
                        let mut check_syndrome = vec![0; r];
                        for idx in 0..n {
                            if error[idx] == 1 {
                                for j in 0..r {
                                    check_syndrome[j] ^= h[[j, idx]];
                                }
                            }
                        }

                        if check_syndrome == syndrome_vec {
                            let duration = start.elapsed().as_micros();
                            println!("Time: {} μs", duration);
                            return Some(error);
                        }
                    }
                }
            }
        }
    }

    let duration = start.elapsed().as_micros();
    println!("Time: {} μs", duration);
    None
}
