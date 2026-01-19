package site.nyaalex.paint.core

import androidx.lifecycle.ViewModel
import site.nyaalex.paint.rust.Behaviour
import site.nyaalex.paint.rust.Runtime

class CoreViewModel : ViewModel() {
    val runtime: Runtime = Runtime()
    init { addCloseable { runtime }}

    val behaviour: Behaviour = Behaviour(runtime)
    init { addCloseable { behaviour }}
}