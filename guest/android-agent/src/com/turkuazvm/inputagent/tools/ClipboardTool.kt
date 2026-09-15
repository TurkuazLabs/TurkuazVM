// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/tools/ClipboardTool.kt
// # 📌 Amac: Android clipboard erisimini Guest Agent Service katmanindan ayiran adapteri saglar
// # 📌 Modul - Kotlin
// # Version: 0.18.0
// # Aciklama: Plain-text clipboard okuma/yazma ve merkezi karakter limiti dogrulamasini uygular
// # Bagimli Oldugu Katman: Tool | Service

package com.turkuazvm.inputagent.tools

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context

internal class ClipboardTool(
    private val context: Context,
    private val maxChars: Int,
) {
    private val clipboard = context.getSystemService(ClipboardManager::class.java)

    fun readText(): String {
        val clip = clipboard.primaryClip ?: return ""
        val item = if (clip.itemCount > 0) clip.getItemAt(0) else return ""
        return item.coerceToText(context)?.toString()?.take(maxChars) ?: ""
    }

    fun writeText(text: String) {
        require(text.length <= maxChars) { "clipboard_payload_too_large" }
        clipboard.setPrimaryClip(ClipData.newPlainText("TurkuazVM", text))
    }
}
