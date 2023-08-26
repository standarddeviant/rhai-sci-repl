
// use nalgebra::DimName;
use num_complex::Complex;
use rhai::{Engine, FLOAT};

use ndarray::{Array, ArrayD, Shape, IxDynImpl, Ix1, Ix2, Dim, Ix, IxDyn, Array2, Array1};
use ndarray::linalg::{general_mat_mul, general_mat_vec_mul};
// let array = Array::from_vec(vec![1., 2., 3., 4.]);

pub type cpx = Complex<f64>;
// pub type CArray = ArrayD<cpx>;
pub type CVector = Array1<cpx>;
pub type CMatrix = Array2<cpx>;

// TODO - explore meta programming rust to autogen functions like these...
pub fn cvzeros(i0:i64        ) ->  CVector { CVector::zeros(Ix1(i0 as usize             )) }
pub fn cmzeros(i0:i64, i1:i64) ->  CMatrix { CMatrix::zeros(Ix2(i0 as usize, i1 as usize)) }

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
        .register_fn("cpx", cpx::new);


    // Register custom type with friendly name
    engine.register_type_with_name::<CVector>("CVector")
        .register_fn("czeros", cvzeros)
        .register_fn("reshape", reshape_cvec_cmat)
        .register_fn("arange", reshape_cvec_cmat)
        ;

    // Register custom type with friendly name
    engine.register_type_with_name::<CMatrix>("CMatrix")
        .register_fn("czeros", cmzeros)
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
        .register_fn("reshape", reshape_cmat_cmat)
        //pub fn reshape_cvec_cmat(a: CVector, i0: i64, i1: i64) -> CMatrix {
        ;


    return engine;
}

fn main(){}
