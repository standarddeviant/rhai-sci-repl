
use nalgebra::{Matrix, DMatrix, OMatrix, Complex};

pub type c128 = Complex<f64>;
pub type cpx = c128;
pub type CMatrix = DMatrix<cpx>;

/*
pub fn cpx_new(re: f64, im: f64) -> cpx {
    cpx::new(re, im)
}
*/
// #[derive(Clone)]
// struct CMatrix(DMatrix<c128>);
/*
#[derive(Clone)]
pub struct CMatrix(
impl Display for CMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, c)= self.0.shape();
        write!(f, "CMatrix; [{r}, {c}]")
    }
}
*/
pub fn czeros(r: i64, c: i64) ->  CMatrix {
    // CMatrix(
        DMatrix::<c128>::zeros(
            r as usize, c as usize
        )
    // )
}

fn main() {}
