package site.nyaalex.paint.rust

import java.io.Closeable

sealed class Renderer(initPtr: Long) : Closeable {
    internal var ptr: Long = initPtr
        private set

    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun destroy(ptr: Long)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}