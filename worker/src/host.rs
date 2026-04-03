use ark_ec::VariableBaseMSM;
use ark_pallas::Projective as Pallas;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress};
use ark_vesta::Projective as Vesta;

pub fn host_msm_unchecked(is_pallas: u32, buffer: &mut [u8], buf_len: u32) -> u32 {
    if is_pallas != 0 {
        host_msm_unchecked_impl::<Pallas>(is_pallas, buffer, buf_len)
    } else {
        host_msm_unchecked_impl::<Vesta>(is_pallas, buffer, buf_len)
    }
}

fn host_msm_unchecked_impl<V: VariableBaseMSM>(
    _is_pallas: u32,
    buffer: &mut [u8],
    buf_len: u32,
) -> u32 {
    let buf_len = buf_len as usize;
    let mut cursor = std::io::Cursor::new(&buffer[..buf_len]);
    let bases: Vec<V::MulBase> =
        CanonicalDeserialize::deserialize_uncompressed_unchecked(&mut cursor).unwrap();
    let scalars: Vec<V::ScalarField> =
        CanonicalDeserialize::deserialize_uncompressed_unchecked(&mut cursor).unwrap();
    //println!("host MSM, is_pallas: {}, len = {}", _is_pallas, bases.len());
    let res = V::msm_unchecked_impl(&bases, &scalars);
    let res_len = res.serialized_size(Compress::No);
    res.serialize_uncompressed(&mut buffer[0..res_len]).unwrap();
    res_len as u32
}
