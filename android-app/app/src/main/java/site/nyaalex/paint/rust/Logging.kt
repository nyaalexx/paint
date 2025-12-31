package site.nyaalex.paint.rust

object Logging {
    init {
        System.loadLibrary("paint_android")
    }

    external fun init()
}