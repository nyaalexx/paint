package site.nyaalex.paint.rust

import java.lang.AutoCloseable
import java.nio.file.Path

class Behaviour(runtime: Runtime) : AutoCloseable {
    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(runtimePtr: Long): Long

        external fun setViewportTransform(ptr: Long, scale: Float, angle: Float, x: Float, y: Float)

        external fun beginBrushStroke(ptr: Long)

        external fun updateBrushStroke(ptr: Long, x: Float, y: Float, pressure: Float)

        external fun endBrushStroke(ptr: Long)

        external fun attachViewportSurface(ptr: Long, surfacePtr: Long)

        external fun save(ptr: Long, path: String)

        external fun open(ptr: Long, path: String)

        external fun destroy(ptr: Long)
    }

    internal var ptr: Long = Native.create(runtime.ptr)
        private set

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

    fun attachViewportSurface(surface: Surface) {
        Native.attachViewportSurface(ptr, surface.ptr)
    }

    fun save(path: Path) {
        Native.save(ptr, path.toString())
    }

    fun open(path: Path) {
        Native.open(ptr, path.toString())
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}