package site.nyaalex.paint

import android.content.Context
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.viewmodel.compose.viewModel
import site.nyaalex.paint.rust.ColorPickerRenderer
import site.nyaalex.paint.rust.Surface

@Composable
fun ColorPicker(modifier: Modifier = Modifier, viewModel: PaintViewModel = viewModel()) {
    AndroidView(
        factory = { context ->
            ColorPickerView(context)
        },
        update = { view ->
            view.bind(viewModel)
        },
        modifier = modifier
    )
}

private class ColorPickerView(context: Context) : SurfaceView(context) {
    private var viewModel: PaintViewModel? = null

    fun bind(viewModel: PaintViewModel) {
        this.viewModel = viewModel
    }

    private var surface: Surface? = null
    private var renderer: ColorPickerRenderer? = null

    private var hue: Float = 0.0f

    init {
        holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                val vm = viewModel ?: return
                surface = Surface(vm.runtime, holder.surface)
                renderer = ColorPickerRenderer(vm.runtime)
                render()
            }

            override fun surfaceChanged(
                holder: SurfaceHolder,
                format: Int,
                width: Int,
                height: Int
            ) {
                surface?.resize(width, height)
                render()
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
                renderer?.close()
                renderer = null
                surface?.close()
                surface = null
            }
        })
    }

    private fun render() {
        val renderer = renderer ?: return
        val surface = surface ?: return
        renderer.renderOkhsvHueSlice(surface, hue)
    }
}