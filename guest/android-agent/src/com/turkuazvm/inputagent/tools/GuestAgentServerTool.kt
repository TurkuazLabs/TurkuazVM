// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/tools/GuestAgentServerTool.kt
// # 📌 Amac: Host Guest Agent Protocol v2 isteklerini authenticated TVGB TCP uzerinden servis eder
// # 📌 Modul - Kotlin
// # Version: 0.18.0
// # Aciklama: Ping, capability, touch, reset ve clipboard komutlarini TVGB v2 session/replay korumasi altinda yonlendirir
// # Bagimli Oldugu Katman: Service | Tool | Config

package com.turkuazvm.inputagent.tools

import com.turkuazvm.inputagent.config.AgentConfig
import org.json.JSONArray
import org.json.JSONObject
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.atomic.AtomicBoolean

private const val ACTION_PING = "ping"
private const val ACTION_CAPABILITIES = "capabilities"
private const val ACTION_APPLY_TOUCH_FRAME = "apply_touch_frame"
private const val ACTION_RESET_INPUT = "reset_input"
private const val ACTION_CLIPBOARD_GET = "clipboard_get"
private const val ACTION_CLIPBOARD_SET = "clipboard_set"
private const val RESPONSE_PONG = "pong"
private const val RESPONSE_CAPABILITIES = "capabilities"
private const val RESPONSE_ACK = "ack"
private const val RESPONSE_CLIPBOARD_TEXT = "clipboard_text"
private const val INPUT_BACKEND = "accessibility_continuation"

class GuestAgentServerTool(
    private val config: AgentConfig,
    private val touchTool: TouchGestureTool,
    private val clipboardTool: ClipboardTool,
) {
    private val running = AtomicBoolean(false)
    private var serverSocket: ServerSocket? = null
    private var worker: Thread? = null

    fun start() {
        if (!running.compareAndSet(false, true)) return
        worker = Thread({ runServer() }, "TurkuazInputAgentServer").apply { start() }
    }

    fun stop() {
        running.set(false)
        runCatching { serverSocket?.close() }
        worker?.interrupt()
        worker = null
        serverSocket = null
        touchTool.reset()
        config.rootKey.fill(0)
    }

    private fun runServer() {
        try {
            ServerSocket(config.port, 8, InetAddress.getLoopbackAddress()).use { server ->
                serverSocket = server
                while (running.get()) {
                    val socket = runCatching { server.accept() }.getOrNull() ?: break
                    handleClient(socket)
                }
            }
        } finally {
            running.set(false)
            serverSocket = null
        }
    }

    private fun handleClient(socket: Socket) {
        socket.use { client ->
            val channel = runCatching { TvgbSecureChannel.accept(client, config.rootKey) }.getOrNull() ?: return
            channel.use { secure ->
                while (running.get() && !client.isClosed) {
                    val frame = runCatching { secure.readRequest() }.getOrNull() ?: break
                    val response = runCatching { handleRequest(JSONObject(String(frame.payload, Charsets.UTF_8))) }
                        .getOrElse { errorResponse(frame.requestId, it.message ?: "guest_agent_failure") }
                    val payload = response.toString().toByteArray(Charsets.UTF_8)
                    if (runCatching { secure.writeResponse(frame.requestId, payload) }.isFailure) break
                }
            }
        }
    }

    private fun handleRequest(request: JSONObject): JSONObject {
        val requestId = request.getLong("request_id")
        val protocolVersion = request.getInt("protocol_version")
        if (protocolVersion != config.protocolVersion) {
            return errorResponse(requestId, "protocol_version_mismatch")
        }
        return when (request.getString("action")) {
            ACTION_PING -> successResponse(requestId, JSONObject().put("kind", RESPONSE_PONG))
            ACTION_CAPABILITIES -> successResponse(
                requestId,
                JSONObject()
                    .put("kind", RESPONSE_CAPABILITIES)
                    .put("persistent_multi_touch", true)
                    .put("max_contacts", config.maxContacts)
                    .put("continuation_api", true)
                    .put("input_backend", INPUT_BACKEND)
                    .put("secure_transport_v2", true)
                    .put("clipboard", true),
            )
            ACTION_APPLY_TOUCH_FRAME -> {
                val contacts = parseContacts(request.optJSONArray("contacts") ?: JSONArray())
                touchTool.applyFrame(contacts).fold(
                    onSuccess = { successResponse(requestId, JSONObject().put("kind", RESPONSE_ACK)) },
                    onFailure = { errorResponse(requestId, it.message ?: "touch_frame_failed") },
                )
            }
            ACTION_RESET_INPUT -> touchTool.reset().fold(
                onSuccess = { successResponse(requestId, JSONObject().put("kind", RESPONSE_ACK)) },
                onFailure = { errorResponse(requestId, it.message ?: "reset_failed") },
            )
            ACTION_CLIPBOARD_GET -> successResponse(
                requestId,
                JSONObject()
                    .put("kind", RESPONSE_CLIPBOARD_TEXT)
                    .put("text", clipboardTool.readText()),
            )
            ACTION_CLIPBOARD_SET -> runCatching {
                clipboardTool.writeText(request.getString("text"))
            }.fold(
                onSuccess = { successResponse(requestId, JSONObject().put("kind", RESPONSE_ACK)) },
                onFailure = { errorResponse(requestId, it.message ?: "clipboard_write_failed") },
            )
            else -> errorResponse(requestId, "unsupported_action")
        }
    }

    private fun parseContacts(values: JSONArray): List<AgentTouchContact> {
        val contacts = mutableListOf<AgentTouchContact>()
        for (index in 0 until values.length()) {
            val value = values.getJSONObject(index)
            contacts.add(
                AgentTouchContact(
                    pointerId = value.getInt("pointer_id"),
                    phase = value.getString("phase"),
                    x = value.getInt("x"),
                    y = value.getInt("y"),
                ),
            )
        }
        return contacts
    }

    private fun successResponse(requestId: Long, data: JSONObject): JSONObject = JSONObject()
        .put("request_id", requestId)
        .put("protocol_version", config.protocolVersion)
        .put("ok", true)
        .put("data", data)
        .put("error", JSONObject.NULL)

    private fun errorResponse(requestId: Long, message: String): JSONObject = JSONObject()
        .put("request_id", requestId)
        .put("protocol_version", config.protocolVersion)
        .put("ok", false)
        .put("data", JSONObject.NULL)
        .put("error", message)
}
