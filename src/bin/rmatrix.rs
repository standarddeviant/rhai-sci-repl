
// use nalgebra::DimName;
use rhai::{Engine, FLOAT};
use ndarray::{Array, ArrayD, Shape, IxDynImpl, Ix1, Ix2, Dim, Ix, IxDyn, Array2, Array1};
use ndarray::linalg::{general_mat_mul, general_mat_vec_mul};
// let array = Array::from_vec(vec![1., 2., 3., 4.]);


pub type RVector = Array1<f64>;
pub type RMatrix = Array2<f64>;

// TODO - explore meta programming rust to autogen functions like these...
pub fn rvzeros(i0:i64        ) ->  RVector { RVector::zeros(Ix1(i0 as usize       )) }
pub fn rmzeros(i0:i64, i1:i64) ->  RMatrix { RMatrix::zeros(Ix2(i0 as usize, i1 as usize)) }

// pub fn celadd(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a + b }
// pub fn celsub(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a - b }
// pub fn celmul(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a * b }
// pub fn celdiv(a: CMatrix, b: CMatrix) -> CMatrix { /* TODO - check shape? */ a / b }

pub fn rmatmul(a: RMatrix, b: RMatrix) -> RMatrix { 
    // TODO - check shape?
    let shape = Ix2(a.shape()[0], b.shape()[1]);
    let mut c = RMatrix::zeros(shape);
    general_mat_mul(1_f64, &a, &b, 1_f64, &mut c);
    return c;
}

pub fn reshape_rmat_rmat(a: RMatrix, i0: i64, i1: i64) -> RMatrix {
    let shape = Ix2(i0 as usize, i1 as usize);
    let out = a.into_shape(shape).expect("reshape boo!");
    return out;
}

pub fn reshape_rvec_rmat(a: RVector, i0: i64, i1: i64) -> RMatrix {
    let shape = Ix2(i0 as usize, i1 as usize);
    let out = a.into_shape(shape).expect("reshape boo!");
    return out;
}

pub fn flatten_rmat(a: RMatrix) -> RVector {
    // let L = a.shape().iter().reduce(|x y| x).expect("CMAT Len...");
    let len = (&a).len();
    let out = a.into_shape(Ix1(len)).expect("flatten...");
    return out;
}

pub fn rvec_rmat_register_functions(mut engine: Engine) -> Engine {
    engine.register_type_with_name::<RVector>("RVector")
        .register_fn("zeros", rvzeros)
        .register_fn("reshape", reshape_rvec_rmat)
        .register_fn("range", 
            |n: i64| 
            RVector::range(0.0, n as FLOAT, 1.0))
        .register_fn("range",
            |start: i64, stop: i64|
            RVector::range(start as FLOAT, stop as FLOAT, 1.0))
        .register_fn("range", RVector::range)
        ;

    engine.register_type_with_name::<RMatrix>("RMatrix")
        .register_fn("zeros", rmzeros)
        .register_fn("reshape", reshape_rmat_rmat)
        .register_fn("+", |a: RMatrix, b: i64| a + b as f64)
        .register_fn("-", |a: RMatrix, b: i64| a - b as f64)
        .register_fn("*", |a: RMatrix, b: i64| a * b as f64)
        .register_fn("/", |a: RMatrix, b: i64| a / b as f64)
        .register_fn("+", |a: i64, b: RMatrix| a as f64 + b)
        .register_fn("-", |a: i64, b: RMatrix| a as f64 - b)
        .register_fn("*", |a: i64, b: RMatrix| a as f64 * b)
        .register_fn("/", |a: i64, b: RMatrix| a as f64 / b)
        .register_fn("+", |a: RMatrix, b: f64| a + b)
        .register_fn("-", |a: RMatrix, b: f64| a - b)
        .register_fn("*", |a: RMatrix, b: f64| a * b)
        .register_fn("/", |a: RMatrix, b: f64| a / b)
        .register_fn("+", |a: f64, b: RMatrix| a + b)
        .register_fn("-", |a: f64, b: RMatrix| a - b)
        .register_fn("*", |a: f64, b: RMatrix| a * b)
        .register_fn("/", |a: f64, b: RMatrix| a / b)
        .register_fn("+", |a: RMatrix, b: RMatrix| a + b)
        .register_fn("-", |a: RMatrix, b: RMatrix| a - b)
        .register_fn("*", |a: RMatrix, b: RMatrix| a * b)
        .register_fn("/", |a: RMatrix, b: RMatrix| a / b)
        .register_fn("@", rmatmul)
        ;

    return engine;
}

fn main(){}
