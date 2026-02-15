package site.nyaalex.paint.brush

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class BrushSettings(val settings: List<BrushSetting>)

@Serializable
data class BrushSetting(val name: String, val kind: BrushSettingKind)

@Serializable
sealed class BrushSettingKind {
    @Serializable
    @SerialName("LinearSrgbaColor")
    object LinearSrgbaColor : BrushSettingKind()

    @Serializable
    @SerialName("F32")
    data class F32(val min: Float, val max: Float) : BrushSettingKind()
}
