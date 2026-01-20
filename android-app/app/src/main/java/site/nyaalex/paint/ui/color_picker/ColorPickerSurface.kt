package site.nyaalex.paint.ui.color_picker

import android.content.Context
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.viewmodel.compose.viewModel
import site.nyaalex.paint.core.CoreViewModel
import site.nyaalex.paint.rust.ColorPickerRenderer
import site.nyaalex.paint.rust.Surface

@Composable
fun ColorPickerSurface(slice: Slice, modifier: Modifier = Modifier) {
    val coreViewModel: CoreViewModel = viewModel()

    AndroidView(
        factory = { context ->
            ColorPickerSurfaceView(context)
        },
        update = { view ->
            view.update(coreViewModel, slice)
        },
        modifier = modifier
    )
}

private class ColorPickerSurfaceView(context: Context) : SurfaceView(context) {
    private var coreViewModel: CoreViewModel? = null
    private var slice: Slice? = null
    private var surface: Surface? = null
    private var renderer: ColorPickerRenderer? = null

    init {
        holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                val vm = coreViewModel ?: return
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

    fun update(viewModel: CoreViewModel, slice: Slice) {
        this.coreViewModel = viewModel
        this.slice = slice
        render()
    }

    private fun render() {
        val renderer = renderer ?: return
        val surface = surface ?: return
        val slice = slice ?: return

        when (slice) {
            is Slice.OkhsvHue ->
                renderer.renderOkhsvHueSlice(surface, slice.hue)
            is Slice.OkhslHueVerticalGradient ->
                renderer.renderOkhslHueVerticalGradient(surface)
        }
    }
}