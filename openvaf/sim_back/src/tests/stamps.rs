use ahash::AHashMap;
use expect_test::expect;
use float_cmp::assert_approx_eq;
use sourcegen::project_root;

#[test]
fn resistor() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file = project_root().join("integration_tests").join("RESISTOR").join("resistor.va");
    let (db, _module, mir, mut literals) = super::compile_to_mir(&root_file);

    //define parameters
    let r = 15.0;
    let zeta = 1.0;
    let tnom = 250.0;
    let va = 1.0;
    let vb = 0.0;

    // prepare inputs
    let temp = 298.5;
    let mut params = AHashMap::default();
    let mut node_voltages = AHashMap::default();

    params.insert("R", r.into());
    params.insert("zeta", zeta.into());
    params.insert("tnom", tnom.into());

    node_voltages.insert("A", va);
    node_voltages.insert("B", vb);

    // run the interpreter
    let result = mir.interpret(&db, &mut literals, &params, &node_voltages, temp);
    let stamps = mir.matrix.resistive_stamps(&db, &result);
    let rhs = mir.residual.resistive_entries(&db);

    // read the matrix entries
    let ia_va = stamps[("A", "A")];
    let ia_vb = stamps[("A", "B")];
    let ib_va = stamps[("B", "A")];
    let ib_vb = stamps[("B", "B")];

    let i_a = result.read(rhs["A"]);
    let i_b = result.read(rhs["B"]);

    // calculate the expected values for the stamps
    let vab = va - vb;
    let res = r * (temp / tnom).powf(zeta);
    let ir = vab / res;
    let g = ir / vab;

    // Resistor current flows from A into B, resistor voltage = Va-Vb
    let ia_va_expect = g;
    let ia_vb_expect = -g;
    let ib_vb_expect = g;
    let ib_va_expect = -g;

    // finally assert that the values are correct
    let epsilon = 1e-5;
    assert_approx_eq!(f64, ia_va, ia_va_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ia_vb, ia_vb_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ib_vb, ib_vb_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ib_va, ib_va_expect, epsilon = epsilon);

    assert_approx_eq!(f64, i_b, -ir);
    assert_approx_eq!(f64, i_a, ir);
}

#[test]
fn current_source() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file =
        project_root().join("integration_tests").join("CURRENT_SOURCE").join("current_source.va");
    let (db, _module, mir, mut literals) = super::compile_to_mir(&root_file);

    //define parameters
    let i = 1.0;
    let r = 1e3;

    // prepare inputs
    let temp = 298.5;
    let mut params = AHashMap::default();
    let mut node_voltages = AHashMap::default();

    params.insert("Io", i.into());
    params.insert("R", r.into());

    node_voltages.insert("Np", 1.0);
    node_voltages.insert("Nm", 0.0);

    // run the interpreter
    let result = mir.interpret(&db, &mut literals, &params, &node_voltages, temp);
    let stamps = mir.matrix.resistive_stamps(&db, &result);

    // read the matrix entries
    let ip_vp = stamps[("Np", "Np")];
    let ip_vm = stamps[("Np", "Nm")];
    let im_vp = stamps[("Nm", "Np")];
    let im_vm = stamps[("Nm", "Nm")];

    // calculate the expected values for the stamps
    let g = 1.0 / r;

    // Resistor current flows from A into B, resistor voltage = Va-Vb
    let ip_vp_expect = g;
    let ip_vm_expect = -g;
    let im_vm_expect = g;
    let im_vp_expect = -g;

    // finally assert that the values are correct
    let epsilon = 1e-5;
    assert_approx_eq!(f64, ip_vp, ip_vp_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ip_vm, ip_vm_expect, epsilon = epsilon);
    assert_approx_eq!(f64, im_vm, im_vm_expect, epsilon = epsilon);
    assert_approx_eq!(f64, im_vp, im_vp_expect, epsilon = epsilon);
}

#[test]
fn cccs() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file = project_root().join("integration_tests").join("CCCS").join("cccs.va");
    let (db, _module, mir, mut literals) = super::compile_to_mir(&root_file);

    //print assembly
    // println!("{}", mir.func.to_debug_string());

    //define parameters
    let rin = 1.0;
    let rout = 1e9;
    let g = 1e2;

    // prepare inputs
    let temp = 298.5;
    let mut params = AHashMap::default();
    let mut node_voltages = AHashMap::default();

    params.insert("G", g.into());
    params.insert("Rin", rin.into());
    params.insert("Rout", rout.into());

    node_voltages.insert("Inp", 1.0);
    node_voltages.insert("Inm", 0.0);
    node_voltages.insert("Outp", 1e-3);
    node_voltages.insert("Outm", 0.0);

    // run the interpreter
    let result = mir.interpret(&db, &mut literals, &params, &node_voltages, temp);
    let stamps = mir.matrix.resistive_stamps(&db, &result);

    // read the matrix entries
    let op_op = stamps[("Outp", "Outp")];
    let op_om = stamps[("Outp", "Outm")];
    let om_op = stamps[("Outm", "Outp")];
    let om_om = stamps[("Outm", "Outm")];
    // let ip_ip: f64 = result.read(stamps[&("Inp", "Inp")]);
    // let ip_im: f64 = result.read(stamps[&("Inp", "Inm")]);
    // let im_ip: f64 = result.read(stamps[&("Inm", "Inp")]);
    // let im_im: f64 = result.read(stamps[&("Inm", "Inm")]);

    // gin is the derivative of the input BRANCH by the input voltages not the input kirchoff laws
    // calculate the expected values for the stamps
    // let gin = 1.0 / rin;
    let gout = 1.0 / rout;

    // let ip_ip_expect = gin;
    // let ip_im_expect = -gin;
    // let im_ip_expect = -gin;
    // let im_im_expect = gin;
    let op_op_expect = gout;
    let op_om_expect = -gout;
    let om_op_expect = -gout;
    let om_om_expect = gout;

    // finally assert that the values are correct
    let epsilon = 1e-5;
    // assert_approx_eq!(f64, ip_ip, ip_ip_expect, epsilon = epsilon);
    // assert_approx_eq!(f64, ip_im, ip_im_expect, epsilon = epsilon);
    // assert_approx_eq!(f64, im_ip, im_ip_expect, epsilon = epsilon);
    // assert_approx_eq!(f64, im_im, im_im_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_op, op_op_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_om, op_om_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_op, om_op_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_om, om_om_expect, epsilon = epsilon);
}

#[test]
fn vccs() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file = project_root().join("integration_tests").join("VCCS").join("vccs.va");
    let (db, _module, mir, mut literals) = super::compile_to_mir(&root_file);

    //print assembly
    // println!("{}", mir.func.to_debug_string());

    //define parameters
    let rin = 1.0;
    let rout = 1e9;
    let g = 1e3;

    // prepare inputs
    let temp = 298.5;
    let mut params = AHashMap::default();
    let mut node_voltages = AHashMap::default();

    params.insert("G", g.into());
    params.insert("Rin", rin.into());
    params.insert("Rout", rout.into());

    node_voltages.insert("Inp", 1.0);
    node_voltages.insert("Inm", 0.0);
    node_voltages.insert("Outp", 1e-3);
    node_voltages.insert("Outm", 0.0);

    // run the interpreter
    let result = mir.interpret(&db, &mut literals, &params, &node_voltages, temp);
    let stamps = mir.matrix.resistive_stamps(&db, &result);

    // read the matrix entries
    let ip_ip = stamps[("Inp", "Inp")];
    let ip_im = stamps[("Inp", "Inm")];
    let im_ip = stamps[("Inm", "Inp")];
    let im_im = stamps[("Inm", "Inm")];

    let op_op = stamps[("Outp", "Outp")];
    let op_om = stamps[("Outp", "Outm")];
    let om_op = stamps[("Outm", "Outp")];
    let om_om = stamps[("Outm", "Outm")];

    let op_ip = stamps[("Outp", "Inp")];
    let op_im = stamps[("Outp", "Inm")];
    let om_ip = stamps[("Outm", "Inp")];
    let om_im = stamps[("Outm", "Inm")];

    // calculate the expected values for the stamps
    let gin = 1.0 / rin;
    let gout = 1.0 / rout;

    // input resistor
    let ip_ip_expect = gin;
    let ip_im_expect = -gin;
    let im_ip_expect = -gin;
    let im_im_expect = gin;

    // output resistor
    let op_op_expect = gout;
    let op_om_expect = -gout;
    let om_op_expect = -gout;
    let om_om_expect = gout;

    // voltage controlled output current
    let op_ip_expect = g;
    let op_im_expect = -g;
    let om_ip_expect = -g;
    let om_im_expect = g;

    // finally assert that the values are correct
    let epsilon = 1e-5;
    assert_approx_eq!(f64, ip_ip, ip_ip_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ip_im, ip_im_expect, epsilon = epsilon);
    assert_approx_eq!(f64, im_ip, im_ip_expect, epsilon = epsilon);
    assert_approx_eq!(f64, im_im, im_im_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_op, op_op_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_om, op_om_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_op, om_op_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_om, om_om_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_ip, op_ip_expect, epsilon = epsilon);
    assert_approx_eq!(f64, op_im, op_im_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_ip, om_ip_expect, epsilon = epsilon);
    assert_approx_eq!(f64, om_im, om_im_expect, epsilon = epsilon);
}

#[test]
fn diode() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file = project_root().join("integration_tests").join("DIODE").join("diode.va");
    let (db, module, mir, mut literals) = super::compile_to_mir(&root_file);

    //define parameters
    let is = 1e-9;
    let rs = 1.0;
    let zetars = 5.0;
    let zetarth = 2.0;
    let n = 1.0;
    let cj0 = 1e-12;
    let vj = 1.0;
    let m = 0.5;
    let rth = 1e3;
    let va = 1.0;
    let vci = 0.5;
    let vc = 0.0;
    let vdtj = 10.0;
    let tnom = 350.0;
    let zetais = 2.0;
    let ea = 0.0;

    // prepare inputs
    let temp = 298.5;
    let mut params = AHashMap::default();
    let mut node_voltages = AHashMap::default();

    params.insert("Is", is.into());
    params.insert("Rs", rs.into());
    params.insert("N", n.into());
    params.insert("Cj0", cj0.into());
    params.insert("Vj", vj.into());
    params.insert("M", m.into());
    params.insert("Rth", rth.into());
    params.insert("Tnom", tnom.into());
    params.insert("zetars", zetars.into());
    params.insert("zetarth", zetarth.into());
    params.insert("zetais", zetais.into());
    params.insert("ea", ea.into());
    params.insert("minr", 1e-3.into());

    node_voltages.insert("A", va);
    node_voltages.insert("CI", vci);
    node_voltages.insert("C", vc);
    node_voltages.insert("dT", vdtj);

    // run the interpreter
    let result = mir.interpret(&db, &mut literals, &params, &node_voltages, temp);
    // let vars =
    // TODO check reactive component
    // TODO check RHS
    // let _stamps_react = mir.matrix.reactive_stamps(&db, &result);
    // let _rhs = mir.residual.resistive_entries(&db);
    // let _rhs_react = mir.residual.reactive_entries(&db);

    let matrix_res = expect![[r#"
        (A, dT) = v372
        (A, A) = v373
        (A, CI) = v386
        (CI, dT) = v374
        (CI, A) = v375
        (CI, CI) = v388
        (CI, C) = v389
        (C, dT) = v377
        (C, CI) = v378
        (C, C) = v390
        (dT, dT) = v379
        (dT, A) = v380
        (dT, CI) = v392
        (dT, C) = v393
    "#]];

    let matrix_react = expect![[r#"
        (A, dT) = v382
        (A, A) = v383
        (A, CI) = v394
        (CI, dT) = v384
        (CI, A) = v385
        (CI, CI) = v395
    "#]];

    matrix_res.assert_eq(&mir.matrix.print_resistive_stamps(&db));
    matrix_react.assert_eq(&mir.matrix.print_reactive_stamps(&db));

    let stamps = mir.matrix.resistive_stamps(&db, &result);

    // read the matrix entries
    let ia_va = stamps[("A", "A")];
    let ia_vci = stamps[("A", "CI")];
    let ia_dtj = stamps[("A", "dT")];
    let ici_va = stamps[("CI", "A")];
    let ici_vci = stamps[("CI", "CI")];
    let ici_vc = stamps[("CI", "C")];
    let ici_dtj = stamps[("CI", "dT")];
    let ic_vci = stamps[("C", "CI")];
    let ic_vc = stamps[("C", "C")];
    let itj_dtj = stamps[("dT", "dT")];
    let itj_va = stamps[("dT", "A")];
    let itj_vci = stamps[("dT", "CI")];
    let itj_vc = stamps[("dT", "C")];

    // calculate the expected values for the stamps:
    // first some basic pre-calculations
    let pk = 1.3806503e-23;
    let pq = 1.602176462e-19;
    let t_dev = temp + vdtj;
    let tdev_tnom = t_dev / tnom;
    let rs_t = rs * tdev_tnom.powf(zetars);
    let rth_t = rth * tdev_tnom.powf(zetarth);
    let is_t = is * tdev_tnom.powf(zetais / n);
    let rs_dt = zetars * rs * tdev_tnom.powf(zetars - 1.0) / tnom;
    let rth_dt = zetarth * rth * tdev_tnom.powf(zetarth - 1.0) / tnom;
    let is_dt = zetais * is * tdev_tnom.powf(zetais - 1.0) / tnom;
    let vt = t_dev * pk / pq;
    let vt_tj = pk / pq;
    let vaci = va - vci;
    let vcic = vci - vc;

    let id = is_t * ((vaci / (n * vt)).exp() - 1.0);
    let gd = is_t / vt * (vaci / (n * vt)).exp();
    let gdt = -is_t * (vaci / (n * vt)).exp() * vaci / n / vt / vt * vt_tj
        + 1.0 * ((vaci / (n * vt)).exp() - 1.0) * is_dt;

    let irs = vcic / rs_t;
    let g = 1.0 / rs_t;
    let grt = -irs / rs_t * rs_dt;

    let irth = vdtj / rth_t;
    let gt = 1.0 / rth_t - irth / rth_t * rth_dt;

    // let ith = id * vaci + vcic.powf(2.0) / rs_t;
    let ith_vtj = gdt * vaci - vcic.powf(2.0) / rs_t / rs_t * rs_dt;
    let ith_vc = 0.0 - 2.0 * vcic / rs_t;
    let ith_va = gd * vaci + id;
    let ith_vci = -gd * vaci - id + 2.0 * vcic / rs_t;

    // Diode current flows from Ci into A, diode voltage = Va-Vci
    // Resistor current flows from C into Ci, resistor voltage = Vci-Vc

    // stamp diode
    let ia_va_expect = gd;
    let ia_vci_expect = -gd;
    let ici_va_expect = -gd;
    let ici_vci_expect = gd;

    //diode thermal
    let ia_vtj_expect = gdt;
    let ici_vtj_expect = -gdt;

    // stamp resistor
    let ici_vci_expect = ici_vci_expect + g;
    let ici_vc_expect = -g;
    let ic_vci_expect = -g;
    let ic_vc_expect = g;

    // resistor thermal
    let ici_vtj_expect = ici_vtj_expect + grt;
    // let ic_vtj_expect = -grt;

    //stamp rth flowing into T node dTj/rth
    let it_vt_expect = gt;

    //stamp ith flowing out of T node
    let it_vt_expect = ith_vtj - it_vt_expect;
    let it_vci_expect = ith_vci;
    let it_vc_expect = ith_vc;
    let it_va_expect = ith_va;

    // finally assert that the values are correct
    let epsilon = 1e-5;
    assert_approx_eq!(f64, ia_va, ia_va_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ia_vci, ia_vci_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ia_dtj, ia_vtj_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ici_va, ici_va_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ici_vci, ici_vci_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ici_vc, ici_vc_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ici_dtj, ici_vtj_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ic_vc, ic_vc_expect, epsilon = epsilon);
    assert_approx_eq!(f64, ic_vci, ic_vci_expect, epsilon = epsilon);
    assert_approx_eq!(f64, itj_dtj, it_vt_expect, epsilon = epsilon);
    assert_approx_eq!(f64, itj_va, it_va_expect, epsilon = epsilon);
    assert_approx_eq!(f64, itj_vc, it_vc_expect, epsilon = epsilon);
    assert_approx_eq!(f64, itj_vci, it_vci_expect, epsilon = epsilon);

    let opvars = mir.opvars(&module, &result);
    assert_approx_eq!(f64, opvars["gd"], gd, epsilon = epsilon);
}

#[test]
fn hicum() {
    // compile model and obtain:
    // a data base that contains all info about the model available to the compiler
    // the optimized MIR that represents the actual compiled code (including matrix entries)
    // the interned string literals (unintersting)
    let root_file = project_root().join("integration_tests").join("HICUML2").join("hicuml2.va");
    let (db, _, mir, _) = super::compile_to_mir(&root_file);

    let matrix_res = expect![[r#"
        (bi, bi) = v47464
        (bi, ei) = v47462
        (bi, ci) = v47461
        (bi, bp) = v47463
        (bi, tnode) = v46253
        (ei, bi) = v47466
        (ei, ei) = v47472
        (ei, ci) = v47471
        (ei, bp) = v46257
        (ei, tnode) = v46258
        (ei, e) = v47470
        (ei, xf2) = v46260
        (ci, bi) = v47474
        (ci, ei) = v47481
        (ci, ci) = v47480
        (ci, bp) = v46264
        (ci, si) = v46265
        (ci, tnode) = v46266
        (ci, c) = v47479
        (ci, xf2) = v46268
        (b, tnode) = v46270
        (b, b) = v46271
        (b, bp) = v47482
        (bp, bi) = v47491
        (bp, ei) = v47486
        (bp, ci) = v47489
        (bp, bp) = v47492
        (bp, si) = v46276
        (bp, tnode) = v46277
        (bp, b) = v46279
        (si, bp) = v46280
        (si, ci) = v47494
        (si, si) = v47495
        (si, tnode) = v46282
        (si, s) = v47496
        (c, tnode) = v46284
        (c, ci) = v46285
        (c, c) = v47497
        (e, tnode) = v46286
        (e, ei) = v46287
        (e, e) = v47498
        (s, si) = v46288
        (s, s) = v47499
        (tnode, bi) = v47508
        (tnode, ei) = v47509
        (tnode, ci) = v47511
        (tnode, bp) = v47513
        (tnode, si) = v46293
        (tnode, tnode) = v46294
        (tnode, e) = v47510
        (tnode, c) = v47512
        (tnode, b) = v46298
        (xf1, bi) = v47515
        (xf1, ei) = v47514
        (xf1, ci) = v47516
        (xf1, tnode) = v46301
        (xf1, xf1) = v46302
        (xf1, xf2) = v46303
        (xf2, bi) = v47518
        (xf2, ei) = v47517
        (xf2, ci) = v47519
        (xf2, tnode) = v46306
        (xf2, xf1) = v46307
        (xf2, xf2) = v46308
        (xf, bi) = v47521
        (xf, ei) = v47520
        (xf, ci) = v47522
        (xf, tnode) = v46311
        (xf, xf) = v46312
    "#]];

    let matrix_react = expect![[r#"
        (bi, bi) = v47526
        (bi, ei) = v47523
        (bi, ci) = v47525
        (bi, tnode) = v46315
        (bi, bp) = v46316
        (bi, xf) = v46268
        (ei, bi) = v47528
        (ei, ei) = v47530
        (ei, ci) = v47529
        (ei, bp) = v46320
        (ei, tnode) = v46321
        (ei, xf) = v46322
        (ci, bi) = v47532
        (ci, ei) = v47531
        (ci, ci) = v47536
        (ci, bp) = v46325
        (ci, b) = v46326
        (ci, si) = v46327
        (ci, tnode) = v46328
        (b, b) = v47538
        (b, ci) = v47537
        (b, tnode) = v46330
        (b, e) = v47539
        (bp, bi) = v47547
        (bp, ei) = v47543
        (bp, ci) = v47545
        (bp, bp) = v47548
        (bp, tnode) = v46336
        (bp, e) = v47549
        (e, b) = v46339
        (e, e) = v47551
        (e, bp) = v46340
        (si, si) = v47553
        (si, ci) = v47552
        (si, tnode) = v46342
        (si, s) = v47554
        (s, s) = v47556
        (s, c) = v47555
        (s, tnode) = v46345
        (s, si) = v46346
        (c, s) = v46347
        (c, c) = v47557
        (c, tnode) = v46348
        (tnode, tnode) = v46349
        (xf1, xf1) = v46350
        (xf2, xf2) = v46351
        (xf, xf) = v46352
    "#]];

    matrix_res.assert_eq(&mir.matrix.print_resistive_stamps(&db));
    matrix_react.assert_eq(&mir.matrix.print_reactive_stamps(&db));
}
