package site.nyaalex.paint.core.color

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.colorspace.ColorSpaces

@Immutable
data class LinearSrgb(val r: Float, val g: Float, val b: Float) {
    fun toColor(): Color = Color(r, g, b, 1f, ColorSpaces.LinearSrgb)
}