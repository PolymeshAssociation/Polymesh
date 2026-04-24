use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use ark_ec::VariableBaseMSM;
use ark_pallas::Projective as Pallas;
use ark_serialize::CanonicalSerialize;
use ark_std::UniformRand;
use ark_vesta::Projective as Vesta;

use ark_host_msm::CurveMSMId;
use ark_host_msm_impl::host_msm_unchecked;

fn bench_msm<G: VariableBaseMSM>(dname: &str, c: &mut Criterion) {
    let name = G::curve_name().expect("Curve must have a name");
    let curve_id = CurveMSMId::from_curve_name(name);

    let mut group = c.benchmark_group(format!("Curve MSM: {:?}", dname));

    // Generate random points and scalars.
    let mut rng = ark_std::rand::thread_rng();
    let num_points = 1000;
    let points: Vec<G> = (0..num_points).map(|_| G::rand(&mut rng)).collect();
    let scalars: Vec<G::ScalarField> = (0..num_points)
        .map(|_| G::ScalarField::rand(&mut rng))
        .collect();

    // Encode curve id, bases and scalars into a buffer.
    let mut buffer = Vec::new();
    curve_id
        .serialize_uncompressed(&mut buffer)
        .expect("Failed to serialize curve id");
    points
        .serialize_uncompressed(&mut buffer)
        .expect("Failed to serialize points");
    scalars
        .serialize_uncompressed(&mut buffer)
        .expect("Failed to serialize scalars");
    let buf_len = buffer.len() as u32;

    let expected_result = {
        let mut tmp = buffer.clone();
        // Call host msm function with the encoded input and measure time.
        let res_len = host_msm_unchecked(tmp.as_mut_slice(), buf_len);

        // Decode the result back into a group element.
        let result_bytes = &tmp[..res_len as usize];
        let result =
            G::deserialize_uncompressed(result_bytes).expect("Failed to deserialize MSM result");
        (result, res_len)
    };

    // Benchmark the host MSM function.
    group.bench_function(format!("HostMSM: {:?}", dname), |b| {
        b.iter(|| {
            let mut tmp = buffer.clone();
            // Call host msm function with the encoded input and measure time.
            let res_len = host_msm_unchecked(tmp.as_mut_slice(), buf_len);
            assert_eq!(res_len, expected_result.1, "MSM result length mismatch");

            // Decode the result back into a group element.
            let result_bytes = &tmp[..res_len as usize];
            let result = G::deserialize_uncompressed(result_bytes)
                .expect("Failed to deserialize MSM result");
            assert_eq!(result, expected_result.0);
            black_box(res_len);
        });
    });
}

fn bench_msm_pallas(c: &mut Criterion) {
    bench_msm::<Pallas>("Pallas", c);
}

fn bench_msm_vesta(c: &mut Criterion) {
    bench_msm::<Vesta>("Vesta", c);
}

criterion_group!(benches, bench_msm_pallas, bench_msm_vesta);
criterion_main!(benches);
