package site.nyaalex.paint.ui.color_picker.mode

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import site.nyaalex.paint.core.color.Okhsv
import site.nyaalex.paint.ui.color_picker.Slice
import site.nyaalex.paint.ui.color_picker.SquareSelector
import site.nyaalex.paint.ui.color_picker.VerticalSelector
import site.nyaalex.paint.ui.theme.AppTheme
import java.lang.Math.toDegrees
import kotlin.math.PI

@Composable
fun OkhsvColorPicker(modifier: Modifier = Modifier) {
    var color by remember { mutableStateOf(Okhsv(0f, 0f, 0f)) }

    Column(modifier, verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Row {
            SquareSelector(
                slice = Slice.OkhsvHue(color.h),
                x = color.s,
                y = 1f - color.v,
                selectorBackground = color.toColor(),
                onChange = { x, y ->
                    color = color.copy(s = x, v = 1f - y)
                },
                modifier = Modifier
                    .height(256.dp)
                    .aspectRatio(1f)
            )

            Spacer(modifier = Modifier.width(32.dp))

            VerticalSelector(
                Slice.OkhslHueVerticalGradient,
                value = color.h / 2f / PI.toFloat(),
                onChange = { color = color.copy(h = it * 2f * PI.toFloat()) },
                modifier = Modifier
                    .height(256.dp)
                    .width(32.dp)
            )
        }

        Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            val textStyle = TextStyle(
                color = AppTheme.colors.text,
                fontFamily = AppTheme.typography.monospaceFontFamily,
                fontSize = AppTheme.typography.sizeM
            )

            val hueStr = "H: %3.0f°".format(toDegrees(color.h.toDouble()))
            BasicText(hueStr, style = textStyle)

            val saturationStr = "S: %3.0f%%".format(color.s * 100)
            BasicText(saturationStr, style = textStyle)

            val valueStr = "V: %3.0f%%".format(color.v * 100)
            BasicText(valueStr, style = textStyle)
        }
    }
}