package com.ekushenext.keyboard

import android.content.Context
import android.view.inputmethod.InputMethodManager

/** Gboard, for typing English: is it on this phone, and is it turned on as a keyboard. */
object Gboard {
    const val PACKAGE = "com.google.android.inputmethod.latin"

    private fun imm(context: Context) = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager

    /** Gboard's keyboard id when it is turned on (switching needs that), else null */
    fun enabledId(context: Context): String? =
        imm(context).enabledInputMethodList.firstOrNull { it.packageName == PACKAGE }?.id

    fun installed(context: Context): Boolean =
        imm(context).inputMethodList.any { it.packageName == PACKAGE }
}
