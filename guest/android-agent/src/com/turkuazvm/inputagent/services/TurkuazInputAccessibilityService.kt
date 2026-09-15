// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/services/TurkuazInputAccessibilityService.kt
// # 📌 Amac: Android Guest Agent lifecycle'ini AccessibilityService yasam dongusune baglar
// # 📌 Modul - Kotlin
// # Version: 0.18.0
// # Aciklama: TVGB v2 key fail-closed bootstrap, persistent touch, clipboard ve secure loopback server adapterini baslatir/durdurur
// # Bagimli Oldugu Katman: Service | Tool | Config

package com.turkuazvm.inputagent.services

import android.accessibilityservice.AccessibilityService
import android.view.accessibility.AccessibilityEvent
import com.turkuazvm.inputagent.config.AgentConfig
import com.turkuazvm.inputagent.tools.ClipboardTool
import com.turkuazvm.inputagent.tools.GuestAgentServerTool
import com.turkuazvm.inputagent.tools.TouchGestureTool

class TurkuazInputAccessibilityService : AccessibilityService() {
    private var server: GuestAgentServerTool? = null

    override fun onServiceConnected() {
        super.onServiceConnected()
        val config = runCatching { AgentConfig.load(this) }.getOrElse {
            disableSelf()
            return
        }
        val touchTool = TouchGestureTool(
            service = this,
            maxContacts = config.maxContacts,
            frameDurationMs = config.gestureFrameMs,
        )
        val clipboardTool = ClipboardTool(this, config.maxClipboardChars)
        server = GuestAgentServerTool(config, touchTool, clipboardTool).also { it.start() }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) = Unit

    override fun onInterrupt() = Unit

    override fun onDestroy() {
        server?.stop()
        server = null
        super.onDestroy()
    }
}
