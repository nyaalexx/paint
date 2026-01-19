package site.nyaalex.paint.rust

import java.io.Closeable

class Runtime : Closeable {
    internal var ptr: Long = Native.create()
        private set

    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun create(): Long

        external fun destroy(ptr: Long)
    }

    override fun close() {
        if (ptr == 0L) return
        Native.destroy(ptr)
        ptr = 0L
    }
}