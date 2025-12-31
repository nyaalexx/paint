package site.nyaalex.paint.rust

class ViewportRenderer(gpu: GpuContext) : Renderer(Native.new(gpu.ptr)) {
    private object Native {
        init {
            System.loadLibrary("paint_android")
        }

        external fun new(gpuPtr: Long): Long
    }

}