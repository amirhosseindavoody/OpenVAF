use anyhow::Result;
use camino::Utf8PathBuf;
use stdx::project_root;

use crate::expr::CircuitParam;
use crate::simulation::SimConfig;
use crate::utils::PrettyPrint;
use crate::{veriloga, Arena, Circuit, ExprEvalCtx};

const ATOL: f64 = 1e-9;
const RTOL: f64 = 1e-2;
fn approx_eq(val: f64, ref_val: f64) -> bool {
    let atol = (val - ref_val).abs();
    atol <= ATOL || atol / ref_val.abs() <= RTOL
}
macro_rules! assert_approx_eq {
    ($val: expr, $ref: expr, $($fmt: tt)+) => {{
        if !approx_eq($val, $ref) {
            panic!("assertion failed (left == right): {}\n left: {}\n right: {}", format_args!($($fmt)*),$val.pretty_str(), $refl.pretty_str())
        }
    }};
    ($val: expr, $ref: expr) => {{
        if !approx_eq($val, $ref) {
            panic!("assertion failed (left == right)\n left: {}\n right: {}", $val.pretty_str(), $ref.pretty_str())
        }
    }};
}

macro_rules! assert_approx_eq_cmplx {
    ($val: expr, $ref_real: literal + j $ref_imag: literal, $($fmt: tt)+) => {{
        if !approx_eq($val.re, $ref_real) || !approx_eq($val.im, $ref_imag) ||  {
            panic!("assertion failed (left == right): {}\n left: {}\n right: {}", format_args!($($fmt)*),$vall.pretty_str(), Complex64::new($ref_real, $ref_imag).pretty_str())
        }
    }};

        ($val: expr, $ref_real: literal + j $ref_imag: literal) => {{
        if !approx_eq($val.re, $ref_real) || !approx_eq($val.im, $ref_imag) {
            panic!("assertion failed (left == right)\n left: {}\n right: {}", $val.pretty_str(), num_complex::Complex64::new($ref_real, $ref_imag).pretty_str())
        }
    }};

    ($val: expr, $ref_real: literal - j $ref_imag: literal, $($fmt: tt)+) => {{
        if !approx_eq($val.re, $ref_real) || !approx_eq($val.im, -$ref_imag) {
            panic!("assertion failed (left == right): {}\n left: {}\n right: {}", format_args!($($fmt)*),$vall.pretty_str(), Complex64::new($ref_real, - $ref_imag).pretty_str())
        }
    }};

        ($val: expr, $ref_real: literal - j $ref_imag: literal) => {{
        if !approx_eq($val.re, $ref_real) || !approx_eq($val.im, -$ref_imag) {
            panic!("assertion failed (left == right)\n left: {}\n right: {}", $val.pretty_str(), num_complex::Complex64::new($ref_real, - $ref_imag).pretty_str())
        }
    }};

}

#[test]
fn smoke_test() -> Result<()> {
    let mut arena = Arena::new();
    let mut circ = Circuit::new("test_circ".to_owned(), &mut arena);

    let gnd = circ.lookup_node("ground").expect("ground node");
    let node_x = circ.node("X".to_owned());

    let (vsrc1, _) =
        circ.new_device_instance_by_name("vsrc1".to_owned(), "vsource", vec![node_x, gnd])?;
    circ.set_instance_param(vsrc1, "dc", 1f64.into())?;

    let (res1, _) =
        circ.new_device_instance_by_name("res1".to_owned(), "resistor", vec![node_x, gnd])?;
    circ.set_instance_param(res1, "r", 1e3.into())?;

    let mut ctx = ExprEvalCtx::new(&arena);
    ctx.set_param(CircuitParam::TEMPERATURE, 300.0.into());
    let mut sim = circ.prepare_simulation(ctx.borrow(), &arena, SimConfig::default())?;
    let solution = sim.dc_op()?;
    assert_approx_eq!(solution[gnd], 0.0);
    assert_approx_eq!(solution[node_x], 1.0);
    let curr = sim.dc_lead_current(vsrc1)?[0];
    assert_approx_eq!(curr, -1e-3);

    Ok(())
}

#[test]
fn veriloga() -> Result<()> {
    let mut arena = Arena::new();
    let mut circ = Circuit::new("test_circ".to_owned(), &mut arena);

    let gnd = circ.lookup_node("ground").expect("ground node");
    let node_x = circ.node("X".to_owned());

    let path = Utf8PathBuf::from_path_buf(project_root())
        .expect("only utf8 paths are supported")
        .join("integration_tests")
        .join("DIODE")
        .join("diode.va");
    circ.load_veriloga_file(path, &veriloga::Opts::default())?;
    let (vdc_param, vdc) = arena.def_param(circ.ctx, "vdc".to_owned())?;

    let (vsrc1, _) =
        circ.new_device_instance_by_name("vsrc1".to_owned(), "vsource", vec![node_x, gnd])?;
    circ.set_instance_param(vsrc1, "dc", vdc)?;
    circ.set_instance_param(vsrc1, "mag", 1.0.into())?;

    let (_, diode1) =
        circ.new_device_instance_by_name("diode1".to_owned(), "diode_va", vec![node_x, gnd])?;

    circ.set_model_param(diode1, "rs", 5f64.into())?;
    circ.set_model_param(diode1, "is", 1e-13.into())?;
    circ.set_model_param(diode1, "n", 1.05.into())?;
    circ.set_model_param(diode1, "rth", 100.into())?;
    circ.set_model_param(diode1, "cj0", 1e-15.into())?;
    circ.set_model_param(diode1, "vj", 0.5.into())?;
    circ.set_model_param(diode1, "m", 0.6.into())?;

    let mut ctx = ExprEvalCtx::new(&arena);

    let vdc = 0.9;
    let delta = 0.01;
    ctx.set_param(vdc_param, (vdc + delta).into());
    ctx.set_param(CircuitParam::TEMPERATURE, 300.15.into());

    let mut sim = circ.prepare_simulation(ctx.borrow(), &arena, SimConfig::default())?;
    let curr_delta = sim.dc_lead_current(vsrc1)?[0];

    ctx.set_param(vdc_param, vdc.into());
    sim.prepare_solver(ctx.borrow(), &arena)?;

    let curr = sim.dc_lead_current(vsrc1)?[0];
    assert_approx_eq!(curr, -0.0365);
    sim.set_omega(10.0 * std::f64::consts::TAU);
    let deriv = (curr_delta - curr) / delta;
    let curr = sim.ac_lead_current(vsrc1)?[0];

    assert_approx_eq!(deriv, curr.re);
    assert_approx_eq_cmplx!(curr, -0.1792 - j 5.09e-18);

    Ok(())
}

#[test]
fn bsimcmg() -> Result<()> {
    let mut arena = Arena::new();
    let mut circ = Circuit::new("test_circ".to_owned(), &mut arena);

    let gnd = circ.lookup_node("ground").expect("ground node");

    let node_s = circ.node("S".to_owned());
    let node_d = circ.node("D".to_owned());
    let node_g = circ.node("G".to_owned());


    let path = Utf8PathBuf::from_path_buf(project_root())
        .expect("only utf8 paths are supported")
        .join("integration_tests")
        .join("BSIMCMG")
        .join("bsimcmg.va");

    println!("BSIMCMG Verilog-A path: {}", path);

    circ.load_veriloga_file(path, &veriloga::Opts::default())?;
    let (mos_instance, mos_model) =
        circ.new_device_instance_by_name("mos".to_owned(), "bsimcmg_va", vec![node_d, node_g, node_s, gnd])?;

    let (vd, _) =
        circ.new_device_instance_by_name("vd".to_owned(), "vsource", vec![node_d, gnd])?;
    let (vd_param, vd_expr) = arena.def_param(circ.ctx, "vdc".to_owned())?;

    circ.set_instance_param(vd, "dc", vd_expr)?;
    circ.set_instance_param(vd, "mag", 1.0.into())?;

    let (vg, _) =
        circ.new_device_instance_by_name("vg".to_owned(), "vsource", vec![node_g, gnd])?;
    let (vg_param, vg_expr) = arena.def_param(circ.ctx, "vg".to_owned())?;
    circ.set_instance_param(vg, "dc", vg_expr)?;
    circ.set_instance_param(vg, "mag", 1.0.into())?;

    let (vs, _) =
        circ.new_device_instance_by_name("vs".to_owned(), "vsource", vec![node_s, gnd])?;
    let (vs_param, vs_expr) = arena.def_param(circ.ctx, "vs".to_owned())?;
    circ.set_instance_param(vs, "dc", vs_expr)?;
    circ.set_instance_param(vs, "mag", 1.0.into())?;



    let mut ctx = ExprEvalCtx::new(&arena);
    ctx.set_param(CircuitParam::TEMPERATURE, 300.15.into());

    // let vd_value: f64 = vd_expr.eval_num(ctx.borrow())?;
    // println!("Default vd_value: {}", vd_value);

    ctx.set_param(vd_param, 1.0.into());
    ctx.set_param(vs_param, 0.into());
    ctx.set_param(vg_param, 0.into());

    let mut sim = circ.prepare_simulation(ctx.borrow(), &arena, SimConfig::default())?;



    let currents = sim.dc_lead_current(mos_instance)?;
    println!("MOS currents: {:?}", currents);

    // ctx.set_param(vd_param, 1.5.into());
    // sim.prepare_solver(ctx.borrow(), &arena)?;

    // let currents = sim.dc_lead_current(mos_instance)?;
    // println!("MOS currents: {:?}", currents);


    println!("vg, id");
    for vg_val in (0..=150).map(|i| i as f64 * 0.01) {
        ctx.set_param(vg_param, vg_val.into());
        sim.prepare_solver(ctx.borrow(), &arena)?;
        let currents = sim.dc_lead_current(mos_instance)?;
        // println!("vg = {:.2}, MOS currents: {:?}", vg_val, currents);
        println!("{:?}, {:?}", vg_val, currents[0]);
    }

    

    Ok(())
}
