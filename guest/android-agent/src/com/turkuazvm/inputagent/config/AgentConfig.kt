// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/config/AgentConfig.kt
// # 📌 Amac: Android resource ve private TVGB key degerlerini typed Guest Agent runtime config'e donusturur
// # 📌 Modul - Kotlin
// # Version: 0.18.0
// # Aciklama: Protocol v2, loopback port, contact, clipboard limiti ve device-protected TVGB root key bilgisini saglar
// # Bagimli Oldugu Katman: Service | Tool | Config

package com.turkuazvm.inputagent.config

import android.content.Context
import com.turkuazvm.inputagent.R
import java.io.File

private const val TVGB_KEY_FILE = "tvgb-v2.key"
private const val TVGB_KEY_SIZE = 32

data class AgentConfig(
    val port: Int,
    val protocolVersion: Int,
    val maxContacts: Int,
    val gestureFrameMs: Long,
    val maxClipboardChars: Int,
    val rootKey: ByteArray,
) {
    companion object {
        fun load(context: Context): AgentConfig {
            val protectedContext = context.createDeviceProtectedStorageContext()
            val keyFile = File(protectedContext.filesDir, TVGB_KEY_FILE)
            val key = keyFile.readBytes()
            require(key.size == TVGB_KEY_SIZE) { "tvgb_root_key_invalid" }
            return AgentConfig(
                port = context.resources.getInteger(R.integer.guest_agent_port),
                protocolVersion = context.resources.getInteger(R.integer.guest_agent_protocol_version),
                maxContacts = context.resources.getInteger(R.integer.guest_agent_max_contacts),
                gestureFrameMs = context.resources.getInteger(R.integer.guest_agent_gesture_frame_ms).toLong(),
                maxClipboardChars = context.resources.getInteger(R.integer.guest_agent_max_clipboard_chars),
                rootKey = key,
            )
        }
    }
}
