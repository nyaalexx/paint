pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::{JFloatArray, JObject};
    use jni_fn::jni_fn;
    use paint_core::color::{Color, Okhsv};

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.ColorUtils$Native")]
    pub fn okhsvToLinearSrgb<'env>(
        env: JNIEnv<'env>,
        _this: JObject,
        h: f32,
        s: f32,
        v: f32,
    ) -> JFloatArray<'env> {
        let hsv = Okhsv::new(h, s, v);
        let rgb = hsv.to_linear_srgb_clamped();
        let array = env.new_float_array(3).unwrap();
        env.set_float_array_region(&array, 0, &[rgb.r, rgb.g, rgb.b])
            .unwrap();
        array
    }
}
