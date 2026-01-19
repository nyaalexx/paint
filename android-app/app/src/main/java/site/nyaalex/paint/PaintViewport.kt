package site.nyaalex.paint

import android.annotation.SuppressLint
import android.content.Context
import android.util.Log
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.viewmodel.compose.viewModel
import site.nyaalex.paint.rust.Behaviour
import site.nyaalex.paint.rust.Surface
import kotlin.math.absoluteValue
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.sin
import kotlin.math.sqrt

@Composable
fun PaintViewport(modifier: Modifier = Modifier, viewModel: PaintViewModel = viewModel()) {
    AndroidView(
        factory = { context ->
            PaintViewportView(context)
        },
        update = { view ->
            view.bind(viewModel)
        },
        modifier = modifier
    )
}

private class PaintViewportView(context: Context) : SurfaceView(context) {
    private var viewModel: PaintViewModel? = null

    fun bind(viewModel: PaintViewModel) {
        this.viewModel = viewModel
    }

    private val behaviour: Behaviour?
        get() = viewModel?.behaviour

    private var surface: Surface? = null

    init {
        holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                val vm = viewModel ?: return
                val newSurface = Surface(vm.runtime, holder.surface)
                vm.behaviour.attachViewportSurface(newSurface)
                surface = newSurface
            }

            override fun surfaceChanged(
                holder: SurfaceHolder,
                format: Int,
                width: Int,
                height: Int
            ) {
                surface?.resize(width, height)
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
                surface?.close()
                surface = null
            }
        })
    }

    override fun onHoverEvent(event: MotionEvent): Boolean {
        for (i in 0 until event.pointerCount) {
            if (event.getToolType(i) != MotionEvent.TOOL_TYPE_STYLUS) continue

            val pressure = event.getPressure(i)
            val tilt = event.getAxisValue(MotionEvent.AXIS_TILT, i)
            val distance = event.getAxisValue(MotionEvent.AXIS_DISTANCE, i)
            val orientation = event.getAxisValue(MotionEvent.AXIS_ORIENTATION, i)

            Log.d(
                "Stylus",
                "Hover $i: pressure=$pressure tilt=$tilt orientation=$orientation distanc=$distance"
            )
        }

        return true;
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (event.pointerCount == 1) {
            val type = event.getToolType(0)

            if (type == MotionEvent.TOOL_TYPE_STYLUS) {
                handleStylusEvent(event)
            } else {
                handleSinglePointerEvent(event)
            }
        }

        if (event.pointerCount == 2) {
            handleTwoPointerEvent(event)
        }

        behaviour?.setViewportTransform(transformScale, transformAngle, transformX, transformY)

        return true;
    }

    private fun handleStylusEvent(event: MotionEvent) {
        if (event.actionMasked == MotionEvent.ACTION_DOWN) {
            behaviour?.beginBrushStroke()
        }

        for (i in 0..event.historySize) {
            val (x, y, pressure) = if (i < event.historySize) {
                val x = event.getHistoricalX(0, i)
                val y = event.getHistoricalY(0, i)
                val pressure = event.getHistoricalPressure(0, i)
                Triple(x, y, pressure)
            } else {
                val x = event.getX(0)
                val y = event.getY(0)
                val pressure = event.getPressure(0)
                Triple(x, y, pressure)
            }

            val translatedX = x - transformX
            val translatedY = y - transformY

            val rotatedX = translatedX * cos(-transformAngle) - translatedY * sin(-transformAngle)
            val rotatedY = translatedX * sin(-transformAngle) + translatedY * cos(-transformAngle)

            val originalX = rotatedX / transformScale
            val originalY = rotatedY / transformScale

            behaviour?.updateBrushStroke(originalX, originalY, pressure)
        }

        if (event.actionMasked == MotionEvent.ACTION_UP) {
            behaviour?.endBrushStroke()
        }
    }

    private var transformScale: Float = 1f
    private var transformAngle: Float = 0f
    private var transformX: Float = 0f
    private var transformY: Float = 0f

    private var panX: Float = 0f
    private var panY: Float = 0f

    private fun handleSinglePointerEvent(event: MotionEvent) {
        val x = event.getX(0)
        val y = event.getY(0)

        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                panX = x
                panY = y
            }

            MotionEvent.ACTION_MOVE -> {
                val dx = x - panX
                val dy = y - panY

                transformX += dx
                transformY += dy

                panX = x
                panY = y
            }

            else -> {}
        }
    }

    private var lastAngle: Float = 0f
    private var lastDist: Float = 0f
    private var lastMidX: Float = 0f
    private var lastMidY: Float = 0f

    private val scaleSensitivity: Float = 0.8f

    private fun handleTwoPointerEvent(event: MotionEvent) {
        val aX = event.getX(0)
        val aY = event.getY(0)

        val bX = event.getX(1)
        val bY = event.getY(1)

        val abX = bX - aX
        val abY = bY - aY

        val midX = (aX + bX) * 0.5f
        val midY = (aY + bY) * 0.5f

        when (event.actionMasked) {
            MotionEvent.ACTION_POINTER_DOWN -> {
                lastAngle = atan2(abY, abX)
                lastDist = sqrt(abX * abX + abY * abY)
                lastMidX = midX
                lastMidY = midY
            }

            MotionEvent.ACTION_POINTER_UP -> {
                val remainingIndex = if (event.actionIndex == 0) 1 else 0
                panX = event.getX(remainingIndex)
                panY = event.getY(remainingIndex)
            }

            MotionEvent.ACTION_MOVE -> {
                // Translate
                val dX = midX - lastMidX
                val dY = midY - lastMidY
                transformX += dX
                transformY += dY
                lastMidX = midX
                lastMidY = midY

                // Scale
                val dist = sqrt(abX * abX + abY * abY)
                var scaleChange = 1 + ((dist - lastDist) / lastDist) * scaleSensitivity
                scaleChange = Math.clamp(scaleChange, 0.95f, 1.05f)
                if (scaleChange < 1e-5) scaleChange = 0f
                lastDist = dist

                transformX += (1 - scaleChange) * (midX - transformX)
                transformY += (1 - scaleChange) * (midY - transformY)
                transformScale *= scaleChange

                // Rotate
                val angle = atan2(abY, abX)
                var angleChange = angle - lastAngle
                val minAngle = 1f / dist
                if (angleChange.absoluteValue < minAngle) angleChange = 0f
                lastAngle = angle

                val dxRot = transformX - midX
                val dyRot = transformY - midY
                val cos = cos(angleChange)
                val sin = sin(angleChange)
                transformX = dxRot * cos - dyRot * sin + midX
                transformY = dxRot * sin + dyRot * cos + midY
                transformAngle += angleChange
            }

            else -> {}
        }
    }
}
