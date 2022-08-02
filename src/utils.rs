use rand::{Fill, Rng};
use sha1::{Digest, Sha1};

pub fn sha_from(b: Vec<u8>) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(b);
    let result = hasher.finalize();

    result.into()
}

pub fn random_array<T>(r: &mut T)
where
    T: Fill + ?Sized,
{
    let mut rng = rand::thread_rng();
    rng.fill(r);
}
