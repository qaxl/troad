use serde::{Deserialize, Serialize};
use troad_serde::{from_slice, to_vec};

fn main() {
    #[derive(Debug, Serialize, Deserialize)]
    // pub enum L {
    //     Abc,
    //     Def(String),
    //     Ghi(i32, String),
    //     Jkl(usize),
    //     Man { x: usize, y: String }
    // }
    pub struct L(usize, String);

    let ser = to_vec(&L(423942942, String::from("kdaslkjdlkasdjklklsajdk"))).unwrap();
    let de = from_slice::<L>(&ser).unwrap();

    println!("{de:?}");
}
