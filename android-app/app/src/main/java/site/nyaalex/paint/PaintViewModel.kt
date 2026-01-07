package site.nyaalex.paint

import androidx.lifecycle.ViewModel
import site.nyaalex.paint.rust.Behaviour
import site.nyaalex.paint.rust.GpuContext

class PaintViewModel : ViewModel() {
    val gpu: GpuContext = GpuContext()
    val behaviour: Behaviour = Behaviour(gpu)

    init {
        addCloseable(behaviour)
        addCloseable(gpu)
    }
}