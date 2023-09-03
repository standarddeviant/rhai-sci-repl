
// use ndarray_rand::rand_distr::uniform::{SampleUniform, UniformSampler, UniformFloat};
// use nalgebra::DimName;
use num_complex::Complex;
use rhai::{Engine, FLOAT};

use ndarray::{Array, ArrayD, Shape, IxDynImpl, Ix1, Ix2, Dim, Ix, IxDyn, Array2, Array1};
use ndarray::linalg::{general_mat_mul, general_mat_vec_mul};
// use std::
// ndarray-rand = "0.14.0"
// use rand::Rn
// let array = Array::from_vec(vec![1., 2., 3., 4.]);

pub type cpx = Complex<f64>;
// pub type CArray = ArrayD<cpx>;
pub type CVector = Array1<cpx>;
pub type CMatrix = Array2<cpx>;

// use ndarray::Array;
// use ndarray_rand::RandomExt;
// use ndarray_rand::rand_distr::Uniform;
use rand::prelude::*;

/* basic complex random */
struct CpxRandUniform { relo:f64, rehi:f64, imlo:f64, imhi:f64, n: usize, count: usize}
impl CpxRandUniform {
    pub fn from_n(n: usize) -> Self {
        CpxRandUniform{
            relo: -1.0, rehi: 1.0,
            imlo: -1.0, imhi: 1.0,
            n: n, count: 0
        }
    }
}
impl Iterator for CpxRandUniform {
    type Item = cpx;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.n { return None; }

        self.count += 1;
        return Some(cpx::new(
            (rand::random::<f64>() * 2.0 * (self.rehi - self.relo)) - self.relo,
            (rand::random::<f64>() * 2.0 * (self.imhi - self.imlo)) - self.imlo
        ));
    }
}

// TODO - explore meta programming rust to autogen functions like these...
pub fn zeros_cvec(i0:i64        ) -> CVector { CVector::zeros(Ix1(i0 as usize             )) }
pub fn zeros_cmat(i0:i64, i1:i64) -> CMatrix { CMatrix::zeros(Ix2(i0 as usize, i1 as usize)) }
pub fn rand_cvec(i0: i64)         -> CVector { 
    CVector::from_iter(CpxRandUniform::from_n(i0 as usize))
}
pub fn rand_cmat(i0: i64, i1:i64) -> CMatrix {
    let tmpv = rand_cvec(i0*i1);
    return tmpv.into_shape(Ix2(i0 as usize, i1 as usize)).expect("rand_cmat reshaping...");
}

// pub fn celadd(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a + b }
// pub fn celsub(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a - b }
// pub fn celmul(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a * b }
// pub fn celdiv(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a / b }

pub fn cmatmul(a: CMatrix, b: CMatrix) -> CMatrix { 
    // TODO - check shape?
    let shape = Ix2(a.shape()[0], b.shape()[1]);
    let cpx1 = cpx::new(1.0, 0.0);
    let mut c = CMatrix::zeros(shape);
    general_mat_mul(cpx1, &a, &b, cpx1, &mut c);
    return c;
}

pub fn reshape_cmat_cmat(a: CMatrix, i0: i64, i1: i64) -> CMatrix {
    let shape = Ix2(i0 as usize, i1 as usize);
    let out = a.into_shape(shape).expect("reshape boo!");
    return out;
}
pub fn reshape_cvec_cmat(a: CVector, i0: i64, i1: i64) -> CMatrix {
    let shape = Ix2(i0 as usize, i1 as usize);
    let out = a.into_shape(shape).expect("reshape boo!");
    return out;
}
pub fn flatten_cmat(a: CMatrix) -> CVector {
    // let L = a.shape().iter().reduce(|x y| x).expect("CMAT Len...");
    let len = (&a).len();
    let out = a.into_shape(Ix1(len)).expect("flatten...");
    return out;
}


pub fn cvec_cmat_register_functions(mut engine: Engine) -> Engine {
    engine.register_type_with_name::<cpx>("cpx")
        .register_fn("cpx", cpx::new)
        .register_fn("cpx", |re:i64, im:i64| cpx::new(re as f64, im as f64))
        .register_fn("cpx", |re:i64, im:f64| cpx::new(re as f64, im as f64))
        .register_fn("cpx", |re:f64, im:i64| cpx::new(re as f64, im as f64))

        // cpx +
        .register_fn("+"  , |a:cpx, b:cpx| a + b)
        .register_fn("+"  , |a:cpx, b:i64| cpx::new(a.re+(b as FLOAT), a.im+(b as FLOAT)))
        .register_fn("+"  , |b:i64, a:cpx| cpx::new(a.re+(b as FLOAT), a.im+(b as FLOAT)))
        .register_fn("+"  , |a:cpx, b:f64| cpx::new(a.re+(b as FLOAT), a.im+(b as FLOAT)))
        .register_fn("+"  , |b:f64, a:cpx| cpx::new(a.re+(b as FLOAT), a.im+(b as FLOAT)))

        // cpx -
        .register_fn("-"  , |a:cpx, b:cpx| a - b)
        .register_fn("-"  , |a:cpx, b:i64| cpx::new(a.re-(b as FLOAT), a.im-(b as FLOAT)))
        .register_fn("-"  , |b:i64, a:cpx| cpx::new(a.re-(b as FLOAT), a.im-(b as FLOAT)))
        .register_fn("-"  , |a:cpx, b:f64| cpx::new(a.re-(b as FLOAT), a.im-(b as FLOAT)))
        .register_fn("-"  , |b:f64, a:cpx| cpx::new(a.re-(b as FLOAT), a.im-(b as FLOAT)))

        // cpx pow
        .register_fn("**"  , |a:cpx, b:cpx| a.powc(b) )
        .register_fn("**"  , |a:cpx, b:i64| a.powf(b as FLOAT) )
        .register_fn("**"  , |b:i64, a:cpx| cpx::new(b as FLOAT, 0.0).powc(a))
        .register_fn("**"  , |a:cpx, b:f64| a.powf(b) )
        .register_fn("**"  , |b:f64, a:cpx| cpx::new(b, 0.0).powc(a) )

        // cpx *
        .register_fn("*"  , |a:cpx, b:cpx| a * b)
        .register_fn("*"  , |a:cpx, b:i64| cpx::new(a.re*(b as FLOAT), a.im*(b as FLOAT)))
        .register_fn("*"  , |b:i64, a:cpx| cpx::new(a.re*(b as FLOAT), a.im*(b as FLOAT)))
        .register_fn("*"  , |a:cpx, b:f64| cpx::new(a.re*(b as FLOAT), a.im*(b as FLOAT)))
        .register_fn("*"  , |b:f64, a:cpx| cpx::new(a.re*(b as FLOAT), a.im*(b as FLOAT)))

        // cpx /
        .register_fn("/"  , |a:cpx, b:cpx| a / b)
        .register_fn("/"  , |a:cpx, b:i64| cpx::new(a.re/(b as FLOAT), a.im/(b as FLOAT)))
        .register_fn("/"  , |b:i64, a:cpx| cpx::new(a.re/(b as FLOAT), a.im/(b as FLOAT)))
        .register_fn("/"  , |a:cpx, b:f64| cpx::new(a.re/(b as FLOAT), a.im/(b as FLOAT)))
        .register_fn("/"  , |b:f64, a:cpx| cpx::new(a.re/(b as FLOAT), a.im/(b as FLOAT)))

        // cpx abs/angle
        .register_fn("abs", |a:cpx| (a.re*a.re + a.im*a.im).sqrt())
        .register_fn("angle", |a:cpx| (a.im / a.re).atan() )

        // cpx trig
        .register_fn("sin", |a:cpx| (a.im / a.re).atan().sin())
        .register_fn("cos", |a:cpx| (a.im / a.re).atan().cos())
        .register_fn("tan", |a:cpx| (a.im / a.re) /* .atan().tan() */)
        .register_fn("opposite"  , |a:cpx| a.im )
        .register_fn("adjacent"  , |a:cpx| a.re)
        .register_fn("hypotenuse", |a:cpx| (a.re*a.re + a.im*a.im).sqrt())
        ;

        // these don't make sense...
        // .register_fn("asin", |a:cpx| (a.im / (a.re*a.re + a.im*a.im).sqrt()).asin() )
        // .register_fn("acos", |a:cpx| (a.re / (a.re*a.re + a.im*a.im).sqrt()).acos() )
        // .register_fn("atan", |a:cpx| (a.im / a.re).atan() )


    // Register custom type with friendly name
    engine.register_type_with_name::<CVector>("CVector")
        .register_fn("czeros", zeros_cvec)
        .register_fn("reshape", reshape_cvec_cmat)
        .register_fn("crange", reshape_cvec_cmat)
        ;

    // Register custom type with friendly name
    engine.register_type_with_name::<CMatrix>("CMatrix")
        .register_fn("czeros", zeros_cmat)
        .register_fn("crand", rand_cvec)
        .register_fn("crand", rand_cmat)
        .register_fn("reshape", reshape_cmat_cmat)
        .register_fn("+", |a: CMatrix, b: i64| a + b as f64)
        .register_fn("-", |a: CMatrix, b: i64| a - b as f64)
        .register_fn("*", |a: CMatrix, b: i64| a * b as f64)
        .register_fn("/", |a: CMatrix, b: i64| a / b as f64)
        .register_fn("+", |a: i64, b: CMatrix| cpx::new(a as f64, 0.0) + b)
        .register_fn("-", |a: i64, b: CMatrix| cpx::new(a as f64, 0.0) - b)
        .register_fn("*", |a: i64, b: CMatrix| cpx::new(a as f64, 0.0) * b)
        .register_fn("/", |a: i64, b: CMatrix| cpx::new(a as f64, 0.0) / b)

        .register_fn("+", |a: CMatrix, b: f64| a + b)
        .register_fn("-", |a: CMatrix, b: f64| a - b)
        .register_fn("*", |a: CMatrix, b: f64| a * b)
        .register_fn("/", |a: CMatrix, b: f64| a / b)
        .register_fn("+", |a: f64, b: CMatrix| cpx::new(a, 0.0) + b)
        .register_fn("-", |a: f64, b: CMatrix| cpx::new(a, 0.0) - b)
        .register_fn("*", |a: f64, b: CMatrix| cpx::new(a, 0.0) * b)
        .register_fn("/", |a: f64, b: CMatrix| cpx::new(a, 0.0) / b)

        .register_fn("+", |a: CMatrix, b: cpx| a + b)
        .register_fn("-", |a: CMatrix, b: cpx| a - b)
        .register_fn("*", |a: CMatrix, b: cpx| a * b)
        .register_fn("/", |a: CMatrix, b: cpx| a / b)
        .register_fn("+", |a: cpx, b: CMatrix| a + b)
        .register_fn("-", |a: cpx, b: CMatrix| a - b)
        .register_fn("*", |a: cpx, b: CMatrix| a * b)
        .register_fn("/", |a: cpx, b: CMatrix| a / b)
 
        .register_fn("+", |a: CMatrix, b: CMatrix| a + b)
        .register_fn("-", |a: CMatrix, b: CMatrix| a - b)
        .register_fn("*", |a: CMatrix, b: CMatrix| a * b)
        .register_fn("/", |a: CMatrix, b: CMatrix| a / b)
        .register_fn("@", cmatmul)
        //pub fn reshape_cvec_cmat(a: CVector, i0: i64, i1: i64) -> CMatrix {
        ;


    return engine;
}

fn main(){}
