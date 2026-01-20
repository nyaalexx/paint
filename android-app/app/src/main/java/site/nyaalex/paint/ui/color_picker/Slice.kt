package site.nyaalex.paint.ui.color_picker

sealed class Slice {
    class OkhsvHue(val hue: Float) : Slice()

    object OkhslHueVerticalGradient : Slice()
}