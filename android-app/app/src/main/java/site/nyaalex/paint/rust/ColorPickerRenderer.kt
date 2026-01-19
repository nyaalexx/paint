package site.nyaalex.paint.rust

import java.io.Closeable

class ColorPickerRenderer(gpu: GpuContext) : Closeable {
    internal var ptr: Long = Native.create(gpu.ptr)
        private set

    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(gpuPtr: Long): Long

        external fun renderOkhsvHueSlice(ptr: Long, surfacePtr: Long, hue: Float)

        external fun destroy(ptr: Long)
    }

    fun renderOkhsvHueSlice(surface: Surface, hue: Float) {
        Native.renderOkhsvHueSlice(ptr, surface.ptr, hue)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}