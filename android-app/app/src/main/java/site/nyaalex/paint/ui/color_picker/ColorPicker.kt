package site.nyaalex.paint.ui.color_picker

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import site.nyaalex.paint.ui.color_picker.mode.OkhsvColorPicker
import site.nyaalex.paint.ui.theme.AppTheme

@Composable
fun ColorPicker(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier
            .background(color = AppTheme.colors.popupBackground.copy(alpha = 0.5f), shape = RoundedCornerShape(16.dp))
            .border(
                width = 1.dp,
                color = AppTheme.colors.popupBorder,
                shape = RoundedCornerShape(16.dp)
            )
            .padding(16.dp)
            .shadow(elevation = 16.dp)
    ) {
        val textStyle = TextStyle(
            color = AppTheme.colors.text,
            fontFamily = AppTheme.typography.fontFamily,
            fontSize = AppTheme.typography.sizeM
        )

        Box(Modifier.padding(16.dp)) {
            BasicText("Okhsv", style = textStyle)
        }

        Box(Modifier.padding(16.dp)) {
            OkhsvColorPicker()
        }
    }
}