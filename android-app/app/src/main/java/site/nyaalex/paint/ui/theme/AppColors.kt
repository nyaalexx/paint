package site.nyaalex.paint.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

@Immutable
data class AppColors(
    val background: Color,
    val text: Color,
    val popupBackground: Color,
    val popupBorder: Color,
) {
    companion object {
        fun dark(): AppColors = AppColors(
            background = Color.Black,
            text = Color.White,
            popupBackground = Color(0xFF050509),
            popupBorder = Color(0xFF8A8AA4)
        )
    }
}

