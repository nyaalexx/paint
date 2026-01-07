package site.nyaalex.paint

import android.content.Context
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.viewmodel.compose.viewModel
import site.nyaalex.paint.rust.Behaviour
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

    private val behaviour: Behaviour?
        get() = viewModel?.behaviour

    private var surface: Surface? = null

    init {
        holder.addCallback(object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                val vm = viewModel ?: return
                val newSurface = Surface(vm.gpu, holder.surface)
                vm.behaviour.attachColorPicker(newSurface)
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
}