use prisma::FromColor;

fn main() {
    let rgb = prisma::Rgb::new(256.0, 3.0, 0.0);
    let hsv: prisma::Hsv<f32, angular_units::Deg<f32>> = prisma::Hsv::from_color(&rgb);
    let (angular_units::Deg(h), s, v) = (hsv.hue(), hsv.saturation() as f64, hsv.value() as f64);
    let h = h as f64;
    let v = v / 255.0;
    dbg!(h);
    dbg!(s);
    dbg!(v);
}
