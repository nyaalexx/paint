package site.nyaalex.paint.ui.color_picker

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import kotlin.math.max
import kotlin.math.min

@Composable
fun SquareSelector(
    slice: Slice,
    x: Float,
    y: Float,
    selectorBackground: Color,
    onChange: (Float, Float) -> Unit,
    modifier: Modifier = Modifier
) {
    BoxWithConstraints(modifier) {
        ColorPickerSurface(
            slice, modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 1.dp)
                .pointerInput(Unit) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            val change = event.changes.firstOrNull()
                            if (change != null && change.pressed) {
                                val x = max(0f, min(1f, change.position.x / maxWidth.toPx()))
                                val y = max(0f, min(1f, change.position.y / maxHeight.toPx()))
                                onChange(x, y)
                                change.consume()
                            }
                        }
                    }
                }
        )

        Canvas(
            Modifier
                .width(0.dp)
                .height(0.dp)
                .offset(x = maxWidth * x, y = maxHeight * y)
        ) {
            drawCircle(selectorBackground, radius = 6.dp.toPx(), style = Fill)
            drawCircle(Color.White, radius = 6.dp.toPx(), style = Stroke(width = 1.dp.toPx()))
        }
    }
}