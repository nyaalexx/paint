package site.nyaalex.paint.rust

import android.view.Surface
import java.io.Closeable

class Surface(runtime: Runtime, androidSurface: Surface) : Closeable {
    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(runtimePtr: Long, androidSurface: Surface): Long

        external fun resize(ptr: Long, width: Int, height: Int)

        external fun destroy(ptr: Long)
    }

    internal var ptr: Long = Native.create(runtime.ptr, androidSurface)

    fun resize(width: Int, height: Int) {
        assert(ptr != 0L)
        Native.resize(ptr, width, height)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}