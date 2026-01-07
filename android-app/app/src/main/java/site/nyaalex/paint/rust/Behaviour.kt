package site.nyaalex.paint.rust

import java.io.Closeable

class Behaviour(gpu: GpuContext) : Closeable {
    internal var ptr: Long = Native.create(gpu.ptr)
        private set

    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(gpuPtr: Long): Long

        external fun setViewportTransform(ptr: Long, scale: Float, angle: Float, x: Float, y: Float)

        external fun beginBrushStroke(ptr: Long)

        external fun updateBrushStroke(ptr: Long, x: Float, y: Float, pressure: Float)

        external fun endBrushStroke(ptr: Long)

        external fun attachViewport(ptr: Long, surfacePtr: Long)

        external fun attachColorPicker(ptr: Long, surfacePtr: Long)

        external fun destroy(ptr: Long)
    }

    fun setViewportTransform(scale: Float, angle: Float, x: Float, y: Float) {
        Native.setViewportTransform(ptr, scale, angle, x, y)
    }

    fun beginBrushStroke() {
        Native.beginBrushStroke(ptr)
    }

    fun updateBrushStroke(x: Float, y: Float, pressure: Float) {
        Native.updateBrushStroke(ptr, x, y, pressure)
    }

    fun endBrushStroke() {
        Native.endBrushStroke(ptr)
    }

    fun attachViewport(surface: Surface) {
        Native.attachViewport(ptr, surface.ptr)
    }

    fun attachColorPicker(surface: Surface) {
        Native.attachColorPicker(ptr, surface.ptr)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}