package site.nyaalex.paint.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp

@Immutable
data class AppTypography(
    val fontFamily: FontFamily,
    val monospaceFontFamily: FontFamily,
    val sizeM: TextUnit
) {
    companion object {
        fun default(): AppTypography = AppTypography(
            fontFamily = FontFamily.SansSerif,
            monospaceFontFamily = FontFamily.Monospace,
            sizeM = 14.sp
        )
    }
}
