package site.nyaalex.paint.rust

import java.io.Closeable

class ColorPickerRenderer(runtime: Runtime) : Closeable {
    internal var ptr: Long = Native.create(runtime.ptr)
        private set

    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(runtimePtr: Long): Long

        external fun renderOkhsvHueSlice(ptr: Long, surfacePtr: Long, hue: Float)

        external fun renderOkhslHueVerticalGradient(ptr: Long, surfacePtr: Long)

        external fun destroy(ptr: Long)
    }

    fun renderOkhsvHueSlice(surface: Surface, hue: Float) {
        Native.renderOkhsvHueSlice(ptr, surface.ptr, hue)
    }

    fun renderOkhslHueVerticalGradient(surface: Surface) {
        Native.renderOkhslHueVerticalGradient(ptr, surface.ptr)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}