package site.nyaalex.paint.rust

import site.nyaalex.paint.core.color.LinearSrgb
import site.nyaalex.paint.core.color.Okhsv

object ColorUtils {
    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun okhsvToLinearSrgb(h: Float, s: Float, v: Float): FloatArray
    }

    fun okhsvToLinearSrgb(okhsv: Okhsv): LinearSrgb {
        val arr = Native.okhsvToLinearSrgb(okhsv.h, okhsv.s, okhsv.v)
        return LinearSrgb(arr[0], arr[1], arr[2]);
    }
}