package site.nyaalex.paint.ui.color_picker

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

@Composable
fun VerticalSelector(
    slice: Slice,
    value: Float,
    onChange: (Float) -> Unit,
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
                                onChange(max(0f, min(1f, change.position.y / maxHeight.toPx())))
                                change.consume()
                            }
                        }
                    }
                }
        )

        Canvas(Modifier
            .fillMaxWidth()
            .height(0.dp)
            .offset(y = maxHeight * value)) {
            val path = Path().apply {
                val w = 10.dp.toPx()
                val h = w * sqrt(3f) / 3f

                moveTo(-w, -h)
                lineTo(-w, h)
                lineTo(0f, 0f)
                close()

                moveTo(size.width + w, -h)
                lineTo(size.width + w, h)
                lineTo(size.width, 0f)
                close()
            }

            drawPath(path, Color.White, style = Fill)
            drawPath(path, Color.Black, style = Stroke(width = 1.dp.toPx()))
        }
    }
}