package site.nyaalex.paint.core.color

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import site.nyaalex.paint.rust.ColorUtils

@Immutable
data class Okhsv(val h: Float, val s: Float, val v: Float) {
    fun toColor(): Color = ColorUtils.okhsvToLinearSrgb(this).toColor()
}
